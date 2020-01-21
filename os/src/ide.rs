use crate::io;
use lazy_static::lazy_static;
use spin::Mutex;
use crate::{print, println};
use crate::timer;

//status
const ATA_SR_BSY:  u8 = 0x80; //busy
const ATA_SR_DRDY: u8 = 0x40; //drive ready
const ATA_SR_DF:   u8 = 0x20; //drive ready fault
const ATA_SR_DSC:  u8 = 0x10; //drive seek complete
const ATA_SR_DRQ:  u8 = 0x08; //data request ready
const ATA_SR_CORR: u8 = 0x04; //corrected data
const ATA_SR_IDX:  u8 = 0x02; //index
const ATA_SR_ERR:   u8 = 0x01; //error

//errors
const ATA_ER_BKK:   u8 = 0x80; //bad block
const ATA_ER_UNC:   u8 = 0x40; //uncorrectable data
const ATA_ER_MC:    u8 = 0x20; //media changed
const ATA_ER_IDNF:  u8 = 0x10; //id mark not found
const ATA_ER_MCR:   u8 = 0x08; //media change request
const ATA_ER_ABRT:  u8 = 0x04; //command aborted
const ATA_ER_TK0NF: u8 = 0x02; //track 0 not found
const ATA_ER_AMNF:  u8 = 0x01; // no address mark

//commands
const ATA_CMD_READ_PIO:         u8 = 0x20;
const ATA_CMD_READ_PIO_EXT:     u8 = 0x24;
const ATA_CMD_READ_DMA:         u8 = 0xC8;
const ATA_CMD_READ_DMA_EXT:     u8 = 0x25;
const ATA_CMD_WRITE_PIO:        u8 = 0x30;
const ATA_CMD_WRITE_PIO_EXT:    u8 = 0x34;
const ATA_CMD_WRITE_DMA:        u8 = 0xCA;
const ATA_CMD_WRITE_DMA_EXT:    u8 = 0x35;
const ATA_CMD_CACHE_FLUSH:      u8 = 0xE7;
const ATA_CMD_CACHE_FLUSH_EXT:  u8 = 0xEA;
const ATA_CMD_PACKET:           u8 = 0xA0;
const ATA_CMD_IDENTIFY_PACKET:  u8 = 0xA1;
const ATA_CMD_IDENTIFY:         u8 = 0xEC;
const ATA_CMD_READ:             u8 = 0xA8;
const ATA_CMD_EJECT:            u8 = 0x1B;
 
//identification space
const ATA_IDENT_DEVICE_TYPE:    u8 = 0;
const ATA_IDENT_CYLINDERS:      u8 = 2;
const ATA_IDENT_HEADS:          u8 = 6;
const ATA_IDENT_SECTORS:        u8 = 12;
const ATA_IDENT_SERIAL:         u8 = 20;
const ATA_IDENT_MODEL:          u8 = 54;
const ATA_IDENT_CAPABILITIES:   u8 = 98;
const ATA_IDENT_FIELD_VALID:    u8 = 106;
const ATA_IDENT_MAXLBA:         u8 = 120;
const ATA_IDENT_COMMAND_SETS:   u8 = 164;
const ATA_IDENT_MAX_LBA_EXT:    u8 = 200;

//interface type
const IDE_ATA:   u8 = 0x00;
const IDE_ATAPI: u8 = 0x01;
 
const ATA_MASTER: u8 = 0x00;
const ATA_SLAVE:  u8 = 0x01;

//registers
const ATA_REG_DATA:       u8 = 0x00;
const ATA_REG_ERROR:      u8 = 0x01;
const ATA_REG_FEATURES:   u8 = 0x01;
const ATA_REG_SECCOUNT0:  u8 = 0x02;
const ATA_REG_LBA0:       u8 = 0x03;
const ATA_REG_LBA1:       u8 = 0x04;
const ATA_REG_LBA2:       u8 = 0x05;
const ATA_REG_HDDEVSEL:   u8 = 0x06;
const ATA_REG_COMMAND:    u8 = 0x07;
const ATA_REG_STATUS:     u8 = 0x07;
const ATA_REG_SECCOUNT1:  u8 = 0x08;
const ATA_REG_LBA3:       u8 = 0x09;
const ATA_REG_LBA4:       u8 = 0x0A;
const ATA_REG_LBA5:       u8 = 0x0B;
const ATA_REG_CONTROL:    u8 = 0x0C;
const ATA_REG_ALTSTATUS:  u8 = 0x0C;
const ATA_REG_DEVADDRESS: u8 = 0x0D; 

//channels
const ATA_PRIMARY:   u8 = 0x00;
const ATA_SECONDARY: u8 = 0x01; 
 
//registers
const ATA_READ:  u8 = 0x00;
const ATA_WRITE: u8 = 0x01;

#[repr(C)]
#[derive(Copy, Clone)]
struct IdeChannelRegisters {
    base: u16,
    ctrl: u16,
    bmide: u16,
    n_ien: u8,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct IdeDevice {
    reserved: u8,
    channel: u8,
    drive: u8,
    drive_type: u16,
    signature: u16,
    capabilities: u16,
    command_sets: u32,
    size: u32,
    model: [char; 41],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct IdeHandler {
    channels: [IdeChannelRegisters; 2],
    ide_buf: [u8; 2048],
    atapi_packet: [u8; 12],
    ide_devices: [IdeDevice; 4],
}

lazy_static! {
    pub static ref IDE_HANDLER: Mutex<IdeHandler> = Mutex::new(IdeHandler {
        channels: [IdeChannelRegisters {
            base: 0,
            ctrl: 0,
            bmide: 0,
            n_ien: 0,
        }; 2],
        ide_buf: [0; 2048],
        atapi_packet: [0xA8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        ide_devices: [IdeDevice {
            reserved: 0,
            channel: 0,
            drive: 0,
            drive_type: 0,
            signature: 0,
            capabilities: 0,
            command_sets: 0,
            size: 0,
            model: [' '; 41],
        }; 4],
    });
}

fn ide_read(channel: usize, reg: u8) -> u8 {
    let mut res: u8 = 0;

    if reg > 0x07 && reg < 0x0C {
        ide_write(channel, ATA_REG_CONTROL, 0x80 | IDE_HANDLER.lock().channels[channel].n_ien);
    }
    unsafe {
        if reg < 0x08 {
            res = io::inb(IDE_HANDLER.lock().channels[channel].base + (reg as u16) - 0x00);
        } else if reg < 0x0C {
            res = io::inb(IDE_HANDLER.lock().channels[channel].base + (reg as u16) - 0x06);
        } else if reg < 0x0E {
           res = io::inb(IDE_HANDLER.lock().channels[channel].ctrl + (reg as u16) - 0x0A);
        } else if reg < 0x16 {
            res = io::inb(IDE_HANDLER.lock().channels[channel].bmide + (reg as u16) - 0x0E);
        }
    }
    if reg > 0x07 && reg < 0x0C {
        ide_write(channel, ATA_REG_CONTROL, IDE_HANDLER.lock().channels[channel].n_ien);
    }
    res
}

fn ide_write(channel: usize, reg: u8, data: u8) { 
    if reg < 0x07 && reg < 0x0C {
        ide_write(channel, ATA_REG_CONTROL, 0x80 | IDE_HANDLER.lock().channels[channel].n_ien);
    }
    unsafe {
        if reg < 0x08 {
            io::outb(IDE_HANDLER.lock().channels[channel].base + (reg as u16) - 0x00, data);
        } else if reg < 0x0C {
            io::outb(IDE_HANDLER.lock().channels[channel].base + (reg as u16) - 0x06, data);
        } else if reg < 0x0E {
            io::outb(IDE_HANDLER.lock().channels[channel].ctrl + (reg as u16) - 0x0A, data);
        } else if reg < 0x16 {
            io::outb(IDE_HANDLER.lock().channels[channel].bmide + (reg as u16) - 0x0E, data);
        }
    }
    if reg < 0x07 && reg < 0x0C {
        ide_write(channel, ATA_REG_CONTROL, IDE_HANDLER.lock().channels[channel].n_ien);
    }
}

fn ide_read_buffer(channel: usize, reg: u8, buffer: u32, quads: u32) {
    if reg > 0x07 && reg < 0x0C {
        ide_write(channel, ATA_REG_CONTROL, 0x80 | IDE_HANDLER.lock().channels[channel].n_ien);
    }
    unsafe {
        asm!("pushw %es; pushw %ax; movw %ds, %ax; movw %ax, %es; popw %ax");

        if reg < 0x08 {
            io::insl(IDE_HANDLER.lock().channels[channel].base + (reg as u16) - 0x00, buffer, quads);
        } else if reg < 0x0C {
            io::insl(IDE_HANDLER.lock().channels[channel].base + (reg as u16) - 0x06, buffer, quads);
        } else if reg < 0x0E {
            io::insl(IDE_HANDLER.lock().channels[channel].ctrl + (reg as u16) - 0x0A, buffer, quads);
        } else if reg < 0x16 {
            io::insl(IDE_HANDLER.lock().channels[channel].bmide + (reg as u16) - 0x0E, buffer, quads);
        }

        asm!("popw %es;");
        
        if reg > 0x07 && reg < 0x0C {
            ide_write(channel, ATA_REG_CONTROL, IDE_HANDLER.lock().channels[channel].n_ien);
        }
    }
}

fn ide_polling(channel: usize, advanced_check: u32) -> u8 {
    for i in 0..4 {
        ide_read(channel, ATA_REG_ALTSTATUS);
    }

    while ide_read(channel, ATA_REG_STATUS) & ATA_SR_BSY != 0 {}

    if advanced_check != 0 {
        let state = ide_read(channel, ATA_REG_STATUS);

        if (state & ATA_SR_ERR) != 0 {
            return 2;
        }

        if (state & ATA_SR_DF) != 0 {
            return 1;
        }

       if (state & ATA_SR_DRQ) == 0 {
            return 3;
        }
    }
    return 0;
}

fn ide_print_error(drive: usize, err: u8) -> u8 {
    let mut merr = err;
    if merr == 0 {
        return merr;
    }

    print!("IDE: ");
    if merr == 1 {
        print!("- Device Fault"); 
        merr = 19;
    } else if merr == 2 {
        let st = ide_read(IDE_HANDLER.lock().ide_devices[drive].channel as usize, ATA_REG_ERROR);
        if st & ATA_ER_AMNF != 0 { print!("- No Address Mark Found "); merr = 7; }
        if st & ATA_ER_TK0NF != 0 { print!("- No Media or Media Error"); merr = 3; }
        if st & ATA_ER_ABRT != 0 { print!("- Command Aborted"); merr = 20; }
        if st & ATA_ER_MCR != 0 { print!("- No Media or Media Error"); merr = 3; }
        if st & ATA_ER_IDNF != 0 { print!("- ID Mark Not Found"); merr = 21; }
        if st & ATA_ER_MC != 0 { print!("- No Media or Media Error"); merr = 3; }
        if st & ATA_ER_UNC != 0 { print!("- Uncorrectable Data Error"); merr = 22; }
        if st & ATA_ER_BKK != 0 { print!("- Bad Sectors"); merr = 13; }
    } else if merr == 3 { 
        print!("- Reads Nothing"); 
        merr = 23;
    } else if merr == 28 {
        print!("- Write Protected"); 
        merr = 8;
    }

    print!(" - [{} {}]",
        IDE_HANDLER.lock().ide_devices[drive].channel,
        IDE_HANDLER.lock().ide_devices[drive].drive,
    );
    for i in IDE_HANDLER.lock().ide_devices[drive].model.iter() {
        print!("{}", i);
    }
    println!();

    return merr;
}

pub fn ide_initialize(bar0: u32, bar1: u32, bar2: u32, bar3: u32, bar4: u32) {
    let mut count = 0;

    IDE_HANDLER.lock().channels[ATA_PRIMARY   as usize].base  = ((bar0 & 0xFFFFFFFC) + 0x1F0 * (!bar0)) as u16;
    IDE_HANDLER.lock().channels[ATA_PRIMARY   as usize].ctrl  = ((bar1 & 0xFFFFFFFC) + 0x3F6 * (!bar1)) as u16;
    IDE_HANDLER.lock().channels[ATA_SECONDARY as usize].base  = ((bar2 & 0xFFFFFFFC) + 0x170 * (!bar2)) as u16;
    IDE_HANDLER.lock().channels[ATA_SECONDARY as usize].ctrl  = ((bar3 & 0xFFFFFFFC) + 0x376 * (!bar3)) as u16;
    IDE_HANDLER.lock().channels[ATA_PRIMARY   as usize].bmide = ((bar4 & 0xFFFFFFFC) + 0) as u16;
    IDE_HANDLER.lock().channels[ATA_SECONDARY as usize].bmide = ((bar4 & 0xFFFFFFFC) + 8) as u16;

    ide_write(ATA_PRIMARY as usize, ATA_REG_CONTROL, 2);
    ide_write(ATA_SECONDARY as usize, ATA_REG_CONTROL, 2);

    for i in 0..2 {
        for j in 0..2 {
            let mut err: u8 = 0;
            let mut ide_type = IDE_ATA;
            let mut status: u8 = 0;
            IDE_HANDLER.lock().ide_devices[count].reserved = 0;

            ide_write(i, ATA_REG_HDDEVSEL, 0xA0 | (j << 4));
            timer::wait(2);

            ide_write(i, ATA_REG_COMMAND, ATA_CMD_IDENTIFY);
            timer::wait(2);

            if ide_read(i, ATA_REG_STATUS) == 0 { continue; }

            loop {
                status = ide_read(i, ATA_REG_STATUS);
                if status & ATA_SR_ERR != 0 { err = 1; break; }
                if !(status & ATA_SR_BSY != 0) && (status & ATA_SR_DRQ != 0) { break; }
            }

            if err != 0 {
                let cl = ide_read(i, ATA_REG_LBA1);
                let ch = ide_read(i, ATA_REG_LBA2);

                if cl == 0x14 && ch == 0xEB {
                    ide_type = IDE_ATAPI;
                } else if cl == 0x69 && ch == 0x96 {
                    ide_type = IDE_ATAPI;
                } else {
                    continue;
                }

                ide_write(i, ATA_REG_COMMAND, ATA_CMD_IDENTIFY_PACKET);
                timer::wait(2);
            }

            ide_read_buffer(i, ATA_REG_DATA, IDE_HANDLER.lock().ide_buf as u32, 128);

            IDE_HANDLER.lock().ide_devices[count].reserved = 1;
            IDE_HANDLER.lock().ide_devices[count].drive_type = ide_type;
            IDE_HANDLER.lock().ide_devices[count].channel = i;
            IDE_HANDLER.lock().ide_devices[count].drive = 0;
            IDE_HANDLER.lock().ide_devices[count].signature = IDE_HANDLER.lock().ide_buf + ATA_IDENT_DEVICE_TYPE as u16; 
            IDE_HANDLER.lock().ide_devices[count].capabilities = IDE_HANDLER.lock().ide_buf + ATA_IDENT_CAPABILITIES as u16;
            IDE_HANDLER.lock().ide_devices[count].command_sets = IDE_HANDLER.lock().ide_buf + ATA_IDENT_COMMAND_SETS;
            

        }
    }
}


