use crate::vfs;
use crate::drivers::ata;
use spin::Mutex;
use lazy_static::lazy_static;
use bit_field::BitField;
use crate::println;
use alloc::string::String;
use alloc::vec::Vec;
use core::convert::TryInto;

#[repr(C)]
pub struct InfoBlock {
    reserved: u8,
    signature: [u8; 8],
    blocks: u64,
    blocks_in_use: u64,
    files: u64,
    bytes_per_block: u64,
}

#[repr(C)]
pub struct FileEntry {
    parent_id: u64,
    filename: [char; 77],
    id: u64,
    attributes: u8,
    t_creation: u64,
    t_edit: u64,
    owner: u16,
    size: u64,
    start_sec: u64,
}

#[repr(C)]
pub struct DataSector {
    signature: [u8; 4],
    next_sec: u64,
    data: [u8; 500],
}

lazy_static! {
    pub static ref WFS_INFO: Mutex<InfoBlock> = Mutex::new(InfoBlock {
        reserved: 0,
        signature: [0; 8],
        blocks: 0,
        blocks_in_use: 0,
        files: 0,
        bytes_per_block: 0,
    });
}

pub fn init() {

    if !ata::ATA_HANDLER.lock().detected {
        println!("[WFS] ATA init failed. Aborting.");
        return;
    }

    let block0 = ata::pio28_read(true, 0, 1);
    let sig = String::from_utf8_lossy(&block0[1..9]);
    if sig == String::from("_WFS_SIG") {
        println!("[WFS] Valid InfoBlock found.");
        init_fs();
    } else {
        println!("[WFS] No valid InfoBlock found.");
        install_ata();
    }
}

pub fn install_ata() {
    println!("[WFS] Installing wFS on ATA drive.");
    
    WFS_INFO.lock().signature = [b'_', b'W', b'F', b'S', b'_', b'S', b'I', b'G'];
    WFS_INFO.lock().blocks = ata::ATA_HANDLER.lock().total_sectors as u64;
    WFS_INFO.lock().blocks_in_use = 1;
    WFS_INFO.lock().files = 0;
    WFS_INFO.lock().bytes_per_block = 512;

    let mut bufv: Vec<u8> = Vec::new();
    
    bufv.push(WFS_INFO.lock().reserved);
    for b in &WFS_INFO.lock().signature {
        bufv.push(*b);
    }
    for b in &WFS_INFO.lock().blocks.to_le_bytes() {
        bufv.push(*b);
    }
    for b in &WFS_INFO.lock().blocks_in_use.to_le_bytes() {
        bufv.push(*b);
    }
    for b in &WFS_INFO.lock().files.to_le_bytes() {
        bufv.push(*b);
    }
    for b in &WFS_INFO.lock().bytes_per_block.to_le_bytes() {
        bufv.push(*b);
    }

    let mut info: [u8; 512] = [0; 512];
    for i in 0..bufv.len() {
        info[i] = bufv[i];
    }

    println!("[WFS] Writing InfoBlock to ATA drive.");
    ata::pio28_write(ata::ATA_HANDLER.lock().master, 0, 1, info);

    init_fs();
}

pub fn init_fs() {
    let info_block = ata::pio28_read(ata::ATA_HANDLER.lock().master, 0, 1);

    WFS_INFO.lock().reserved = info_block[0];
    WFS_INFO.lock().signature = info_block[1..=8].try_into().expect("");
    WFS_INFO.lock().blocks = u64::from_le_bytes(info_block[9..=16].try_into().expect(""));
    WFS_INFO.lock().blocks_in_use = u64::from_le_bytes(info_block[17..=24].try_into().expect(""));
    WFS_INFO.lock().files = u64::from_le_bytes(info_block[25..=32].try_into().expect(""));
    WFS_INFO.lock().bytes_per_block = u64::from_le_bytes(info_block[33..=40].try_into().expect(""));
}
