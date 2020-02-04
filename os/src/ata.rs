use crate::io;
use crate::println;
use crate::print;
use crate::timer;
use bit_field::BitField;
use alloc::vec::Vec;
use alloc::string::String;

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


pub fn init() {
    unsafe {
        let mut drives = 0;

        io::outb(DRIVESEL, 0xE0);
        io::outb(SECTOR_COUNT, 0);
        io::outb(LBAL, 0);
        io::outb(LBAM, 0);
        io::outb(LBAH, 0);
        io::outb(COMMAND, ATACommand::IdentifyDevice as u8);

        if io::inb(STATUS) == 0 || io::inb(STATUS) == 0xFF {
            println!("ATA: master not found.");
        } else {
            println!("ATA: master found. STATUS = {}", io::inb(STATUS));
            drives += 1;
        }

        io::outb(DRIVESEL, 0xF0);
        io::outb(SECTOR_COUNT, 0);
        io::outb(LBAL, 0);
        io::outb(LBAM, 0);
        io::outb(LBAH, 0);
        io::outb(COMMAND, ATACommand::IdentifyDevice as u8);

        if io::inb(STATUS) == 0 || io::inb(STATUS) == 0xFF {
            println!("ATA: slave not found.");
        } else {
            println!("ATA: slave found. STATUS = {}", io::inb(STATUS));
            drives += 1;
        }

        if drives < 1 {
            println!("ATA: no drives found. Aborting.\n");
            return;
        }

        while io::inb(STATUS).get_bit(BSY) { crate::hlt_loop(); }

        let mut raw: [u16; 256] = [0; 256];
        if io::inb(STATUS).get_bit(DRQ) && !io::inb(STATUS).get_bit(ERR) {
            for i in raw.iter_mut() {
                *i = io::inw(DATA);
            }
        } else {
            println!("ATA: read error");
            return;
        }
        println!();

        let total_sectors_lba28 = {
            let (lobytes, hibytes) = (raw[60].to_le_bytes(), raw[61].to_le_bytes());
            u32::from_le_bytes([lobytes[0], lobytes[1], hibytes[0], hibytes[1]])
        };
        println!("ATA: total LBA28 sectors: {}", total_sectors_lba28);

		let total_sectors_lba48 = {
		    let (lobytes, hibytes) = (
		        [raw[100].to_le_bytes(), raw[101].to_le_bytes()],
		        [raw[102].to_le_bytes(), raw[103].to_le_bytes()],
		    );
		    u64::from_le_bytes([
		        lobytes[0][0],
		        lobytes[0][1],
		        lobytes[1][0],
		        lobytes[1][1],
		        hibytes[0][0],
		        hibytes[0][1],
		        hibytes[1][0],
		        hibytes[1][1],
		    ])
		};
		println!("ATA: total LBA48 sectors: {}", total_sectors_lba48);
		
	    let model_number = {
		    let mut bytes: Vec<u8> = Vec::new();
		    for i in 27..47 {
		        let part = raw[i].to_le_bytes();
		        bytes.push(part[0]);
		        bytes.push(part[1]);
		    }
		    // Swap the bytes
		    for i in (0..bytes.len()).step_by(2) {
		        let tmp = bytes[i];
		        bytes[i] = bytes[i + 1];
		        bytes[i + 1] = tmp;
		    }
		    String::from_utf8(bytes).unwrap()
    	};
		println!("ATA: model number: {}", model_number);

    	let current_media_sn = {
		    let mut bytes: Vec<u8> = Vec::new();
		    for i in 176..206 {
		        let part = raw[i].to_le_bytes();
		        bytes.push(part[0]);
		        bytes.push(part[1]);
		    }
		    // Swap the bytes
		    for i in (0..bytes.len()).step_by(2) {
		        let tmp = bytes[i];
		        bytes[i] = bytes[i + 1];
		        bytes[i + 1] = tmp;
		    }
		    String::from_utf8(bytes).unwrap()
    	};
		println!("ATA: serial number: {}", current_media_sn);

    }
    println!();
}
