use crate::io;
use crate::println;
use bit_field::BitField;
use alloc::vec::Vec;
use alloc::string::String;
use spin::Mutex;
use lazy_static::lazy_static;
use crate::timer;

#[repr(u8)]
pub enum ATACommand {
    CfaEraseSectors = 0xC0,
    CfaRequestExtendedErrorCode = 0x03,
    CfaTranslateSector = 0x87,
    CfaWriteMultipleWithoutErase = 0xCD,
    CfaWriteSectorsWithoutErase = 0x38,
    CheckMediaCardType = 0xD1,
    CheckPowerMode = 0xE5,
    ConfigureStream = 0x51,
    DeviceConfigure = 0xB1,
    DeviceReset = 0x08,
    DownloadMicrocode = 0x92,
    ExecuteDeviceDiagnostic = 0x90,
    FlushCache = 0xE7,
    FlushCacheExt = 0xEA,
    IdentifyDevice = 0xEC,
    IdentifyPacketDevice = 0xA1,
    Idle = 0xE3,
    IdleImmediate = 0xE1,
    Nop = 0x00,
    NvCache = 0xB6,
    Packet = 0xA0,
    ReadBuffer = 0xE4,
    ReadDma = 0xC8,
    ReadDmaExt = 0x25,
    ReadDmaQueued = 0xC7,
    ReadDmaQueuedExt = 0x26,
    ReadFpdmaQueued = 0x60,
    ReadLogExt = 0x2F,
    ReadLogDmaExt = 0x47,
    ReadMultiple = 0xC4,
    ReadMultipleExt = 0x29,
    ReadNativeMaxAddress = 0xF8,
    ReadNativeMaxAddressExt = 0x27,
    ReadSectors = 0x20,
    ReadSectorsExt = 0x24,
    ReadStreamDmaExt = 0x2A,
    ReadStreamExt = 0x2B,
    ReadVerifySectors = 0x40,
    ReadVerifySectorsExt = 0x42,
    SecurityDisablePassword = 0xF6,
    SecurityErasePrepare = 0xF3,
    SecurityEraseUnit = 0xF4,
    SecurityFrezeLock = 0xF5,
    SecuritySetPassword = 0xF1,
    SecurityUnlock = 0xF2,
    Service = 0xA2,
    SetFeatures = 0xEF,
    SetMax = 0xF9,
    SetMaxAddressExt = 0x37,
    SetMultipleMode = 0xC6,
    Sleep = 0xE6,
    Smart = 0xB0,
    Standby = 0xE2,
    StandbyImmediate = 0xE0,
    TrustedNonData = 0x5B,
    TrustedReceive = 0x5C,
    TrustedReceiveDma = 0x5D,
    TrustedSend = 0x5E,
    TrustedSendDma = 0x5F,
    WriteBuffer = 0xE8,
    WriteDma = 0xCA,
    WriteDmaExt = 0x35,
    WriteDmaFuaExt = 0x3D,
    WriteDmaQueued = 0xCC,
    WriteDmaQueuedExt = 0x36,
    WriteDmaQueuedFuaExt = 0x3E,
    WriteFpdmaQueued = 0x61,
    WriteLogExt = 0x3F,
    WriteLogDmaExt = 0x57,
    WriteMultiple = 0xC5,
    WriteMultipleExt = 0x39,
    WriteMultipleFuaExt = 0xCE,
    WriteSectors = 0x30,
    WriteSectorsExt = 0x34,
    WriteStreamDmaExt = 0x3A,
    WriteStreamExt = 0x3B,
    WriteUncorrectableExt = 0x45,
}

const DATA: u16 = 0x1F0;
const ERROR: u16 = 0x1F1;
const FEATURES: u16 = 0x1F1;
const SECTOR_COUNT: u16 = 0x1F2;
const LBAL: u16 = 0x1F3;
const LBAM: u16 = 0x1F4;
const LBAH: u16 = 0x1F5;
const DRIVESEL: u16 = 0x1F6;
const STATUS: u16 = 0x1F7;
const COMMAND: u16 = 0x1F7;
const ALTSTATUS: u16 = 0x3F6;
const DEVCTL: u16 = 0x3F6;
const DRIVE_ADDR: u16 = 0x3F7;
const DATA2: u16 = 0x170;
const ERROR2: u16 = 0x171;
const FEATURES2: u16 = 0x171;
const SECTOR_COUNT2: u16 = 0x172;
const LBAL2: u16 = 0x173;
const LBAM2: u16 = 0x174;
const LBAH2: u16 = 0x175;
const DRIVESEL2: u16 = 0x176;
const STATUS2: u16 = 0x177;
const COMMAND2: u16 = 0x177;
const ALTSTATUS2: u16 = 0x376;
const DEVCTL2: u16 = 0x376;
const DRIVE_ADDR2: u16 = 0x377;
const AMNF: usize = 0;
const TKZNF: usize = 1;
const ABRT: usize = 2;
const MCR: usize = 3;
const IDNF: usize = 4;
const MC: usize = 5;
const UNC: usize = 6;
const BBK: usize = 7;
const DRV: usize = 4;
const LBA: usize = 6;
const ERR: usize = 0;
const DRQ: usize = 3;
const SRV: usize = 4;
const DF: usize = 5;
const RDY: usize = 6;
const BSY: usize = 7;
const NEIN: usize = 1;
const SRST: usize = 2;
const HOB: usize = 7;
const DS0: usize = 0;
const DS1: usize = 1;
const HS0: usize = 2;
const HS1: usize = 3;
const HS2: usize = 4;
const HS3: usize = 5;
const WTG: usize = 6;

pub struct AtaHandler {
    pub detected: bool,
    pub total_sectors: usize,
    pub master: bool,
}

lazy_static! {
    pub static ref ATA_HANDLER: Mutex<AtaHandler> = Mutex::new(AtaHandler {
        detected: false,
        total_sectors: 0,
        master: true,
    });
}

pub fn init() {
    unsafe {
        let mut drives = 0;

        io::outb(DRIVESEL, 0xE0);
        io::outb(SECTOR_COUNT, 0);
        io::outb(LBAL, 0);
        io::outb(LBAM, 0);
        io::outb(LBAH, 0);
        io::outb(COMMAND, ATACommand::IdentifyDevice as u8);

        delay();
        while io::inb(STATUS).get_bit(BSY) { crate::hlt_loop(); }

        if io::inb(STATUS) == 0 || io::inb(STATUS) == 0xFF {
            println!("[ATA] master not found.");
        } else {
            println!("[ATA] master found");
            drives += 1;
        }

        io::outb(DRIVESEL, 0xF0);
        io::outb(SECTOR_COUNT, 0);
        io::outb(LBAL, 0);
        io::outb(LBAM, 0);
        io::outb(LBAH, 0);
        io::outb(COMMAND, ATACommand::IdentifyDevice as u8);

        delay();
        while io::inb(STATUS).get_bit(BSY) { crate::hlt_loop(); }

        if io::inb(STATUS) == 0 || io::inb(STATUS) == 0xFF {
            println!("[ATA] slave not found.");
        } else {
            println!("[ATA] slave found");
            drives += 1;
        }

        if drives < 1 {
            println!("[ATA] no drives found. Aborting.\n");
            return;
        }

        io::outb(DRIVESEL, 0xE0);

        delay();
        while io::inb(STATUS).get_bit(BSY) { crate::hlt_loop(); }

        identify_drive();
    }
}

pub fn identify_drive() {
    unsafe {
        while io::inb(STATUS).get_bit(BSY) { crate::hlt_loop(); }

        let mut raw: [u16; 256] = [0; 256];
        if io::inb(STATUS).get_bit(DRQ) && !io::inb(STATUS).get_bit(ERR) {
            for i in raw.iter_mut() {
                *i = io::inw(DATA);
            }
        } else {
            println!("[ATA] read error");
            return;
        }

        let total_sectors_lba28 = {
            let (lobytes, hibytes) = (raw[60].to_le_bytes(), raw[61].to_le_bytes());
            u32::from_le_bytes([lobytes[0], lobytes[1], hibytes[0], hibytes[1]])
        };
        ATA_HANDLER.lock().total_sectors = total_sectors_lba28 as usize;

	    let model_number = {
		    let mut bytes: Vec<u8> = Vec::new();
		    for i in 27..47 {
		        let part = raw[i].to_le_bytes();
		        bytes.push(part[0]);
		        bytes.push(part[1]);
		    }
		    for i in (0..bytes.len()).step_by(2) {
		        let tmp = bytes[i];
		        bytes[i] = bytes[i + 1];
		        bytes[i + 1] = tmp;
		    }
		    String::from_utf8(bytes).unwrap()
    	};
		println!("[ATA] Model: {}", model_number);

        ATA_HANDLER.lock().detected = true;
    }
}

pub fn pio28_read(master: bool, lba: usize, count: u8) -> [u8; 512] {
    unsafe {
        io::outb(FEATURES, 0x00);
        io::outb(SECTOR_COUNT, count);
        io::outb(LBAL, lba.get_bits(24..32) as u8);
        io::outb(LBAM, lba.get_bits(32..40) as u8);
        io::outb(LBAH, lba.get_bits(40..48) as u8);

        io::outb(SECTOR_COUNT, count.get_bits(0..8) as u8);
        io::outb(LBAL, lba.get_bits(0..8) as u8);
        io::outb(LBAM, lba.get_bits(8..16) as u8);
        io::outb(LBAH, lba.get_bits(16..24) as u8);

        io::outb(COMMAND, ATACommand::ReadSectors as u8);

        delay();
        while io::inb(STATUS).get_bit(BSY) { crate::hlt_loop(); }

        let mut sector: [u8; 512] = [0; 512];

        let mut j = 0;
        for i in 0..256 {
            let rawbytes = io::inw(DATA).to_le_bytes();
            sector[j] = rawbytes[0];
            sector[j + 1] = rawbytes[1];

            j += 2;
        }

        return sector;
    }
}
    
pub fn pio28_write(master: bool, lba: usize, count: u8, sec: [u8; 512]) {
    unsafe {
        let mut buf: [u16; 256] = [0; 256];

        let mut j = 0;
        for i in 0..256 {
            buf[i] = u16::from_le_bytes([sec[j], sec[j+1]]);
            j += 2;
        }

        io::outb(FEATURES, 0x00);
        io::outb(SECTOR_COUNT, count);
        io::outb(LBAL, lba.get_bits(24..32) as u8);
        io::outb(LBAM, lba.get_bits(32..40) as u8);
        io::outb(LBAH, lba.get_bits(40..48) as u8);

        io::outb(SECTOR_COUNT, count.get_bits(0..8) as u8);
        io::outb(LBAL, lba.get_bits(0..8) as u8);
        io::outb(LBAM, lba.get_bits(8..16) as u8);
        io::outb(LBAH, lba.get_bits(16..24) as u8);
    
        delay();
        while io::inb(STATUS).get_bit(BSY) { crate::hlt_loop(); }

        io::outb(COMMAND, ATACommand::WriteSectors as u8);
           
        flush_cache();

        for i in 0..256 {
            io::outw(DATA, buf[i]);
            flush_cache();
        }

    }
}

#[no_mangle]
fn delay() {
    for _ in 0..200 {}
/*
    unsafe {
        io::inb(STATUS);
        io::inb(STATUS);
        io::inb(STATUS);
        io::inb(STATUS);
    }
//    timer::wait(1); */
}

fn flush_cache() {
    unsafe {
        io::outb(COMMAND, ATACommand::FlushCache as u8);
    }
}
