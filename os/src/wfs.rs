//WFS: A shit filesystem
//Spec can be found at ../wfs_spec.txt

use crate::vfs;
use crate::drivers::ata;
use spin::Mutex;
use lazy_static::lazy_static;
use bit_field::BitField;
use crate::println;
use crate::print;
use alloc::string::String;
use alloc::vec::Vec;
use core::convert::TryInto;
use core::mem;

const FREE: u64 = 0x00000000_00000000;
const RESERVED: u64 = 0xFFFFFFFF_FFFFFFF0;
const END_OF_CHAIN: u64 = 0xFFFFFFFF_FFFFFFFF;

const DATA_SIG: [u8; 4] = [b'D', b'A', b'T', b'A'];
const WFS_SIG: [u8; 8] = [b'_', b'W', b'F', b'S', b'_', b'S', b'I', b'G'];

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
    signature: [u8; 4],
    filename: [char; 128],
    parent_id: u64,
    id: u64,
    attributes: u8,
    t_creation: u64,
    t_edit: u64,
    owner: u8,
    size: u64,
    start_sec: u64,
    next_entry: u64,
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
    
    //Create WFS InfoBlock and write to first sector of disk.
    WFS_INFO.lock().signature = WFS_SIG;
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

    let mut root_arr: [u8; 512] = [0; 512];
    let root_attributes: u8 = *0.set_bit(0, true).set_bit(1, true).set_bit(2, true);
    let root = FileEntry {
        filename: filename_from_string(String::from("ATA0")),
        signature: DATA_SIG,
        parent_id: 0,
        id: 0,
        attributes: root_attributes,
        t_creation: 0,
        t_edit: 0,
        owner: 0,
        size: 0,
        start_sec: END_OF_CHAIN,
        next_entry: END_OF_CHAIN,
    };
    let root_arr = sector_from_entry(root);
    println!("[WFS] Writing Root file entry to ATA drive.");
    ata::pio28_write(ata::ATA_HANDLER.lock().master, 1, 1, root_arr);

    let t = find_file(0).unwrap();
    println!("{:x} - {:x} - {:x}", t.start_sec, t.next_entry, END_OF_CHAIN);
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

pub fn read_file(file: FileEntry) -> Option<Vec<u8>> {
    let mut sec_count = 0;

    if file.size % 500 == 0 {
        sec_count = file.size / 500;
    } else {
        sec_count = (file.size - (file.size % 500)) + 1;
    }
  
    let mut ret: Vec<u8> = Vec::new();
    let mut lba = file.start_sec as usize;

    let mut written: usize = 0;
    for i in 0..sec_count {
        let raw = ata::pio28_read(ata::ATA_HANDLER.lock().master, lba, 1);

        if String::from_utf8_lossy(&raw[0..=3]) != String::from("DATA") {
            return None;
        } 

        let next = u64::from_le_bytes(raw[4..=11].try_into().expect(""));

        if next == FREE || next == RESERVED {
            return None;
        }
        
        if next == END_OF_CHAIN {
            break;
        }

        for b in &raw[13..=512] {
            if written >= file.size as usize - 1 {
                break;
            }
            ret.push(*b);
            written += 1;
        }    

        lba = next as usize;
    }

    return Some(ret);
}

pub fn find_file(id: u64) -> Option<FileEntry> {
    let m = ata::ATA_HANDLER.lock().master;

    let mut temp = entry_from_sector(ata::pio28_read(m, 1, 1));
    loop {
        if temp.id == id {
            return Some(temp);
        }

        if temp.next_entry == END_OF_CHAIN  {
            return None;
        }
        temp = entry_from_sector(ata::pio28_read(m, temp.next_entry as usize, 1));
    }
}

pub fn find_file_by_name(parent_id: u64, name: String) -> Option<FileEntry> {
    let m = ata::ATA_HANDLER.lock().master;

    let mut temp = entry_from_sector(ata::pio28_read(m, 1, 1));
    loop {
        if string_from_filename(temp.filename) == name && temp.parent_id == parent_id {
            return Some(temp);
        }

        if temp.next_entry == END_OF_CHAIN {
            return None;
        }

        temp = entry_from_sector(ata::pio28_read(m, temp.next_entry as usize, 1));
    }
}

pub fn get_empty_block() -> usize {
    let first = ata::pio28_read(ata::ATA_HANDLER.lock().master, WFS_INFO.lock().blocks_in_use as usize, 1);
    if String::from_utf8_lossy(&first[0..=3]) != String::from("DATA") {
        return WFS_INFO.lock().blocks_in_use as usize;
    }

    for i in 1..WFS_INFO.lock().blocks as usize {
        let raw = ata::pio28_read(ata::ATA_HANDLER.lock().master, i, 1);
        if String::from_utf8_lossy(&raw[0..=3]) != String::from("DATA") {
            return i;
        }    
    }

    panic!("[WFS] No free block found.");
}


fn sector_from_entry(f: FileEntry) -> [u8; 512] {
    let mut res: [u8; 512] = [0; 512];

    let mut v: Vec<u8> = Vec::with_capacity(256);

    for b in f.signature.iter() {
        v.push(*b);
    }
    for b in f.filename.iter() {
        v.push(*b as u8);
    }
    for b in &f.parent_id.to_le_bytes() {
        v.push(*b);
    }
    for b in &f.id.to_le_bytes() {
        v.push(*b);
    }
    v.push(f.attributes);
    for b in &f.t_creation.to_le_bytes() {
        v.push(*b);
    }
    for b in &f.t_edit.to_le_bytes() {
        v.push(*b);
    }
    v.push(f.owner);
    for b in &f.size.to_le_bytes() {
        v.push(*b);
    }
    for b in &f.start_sec.to_le_bytes() {
        v.push(*b);
    }
    for b in &f.next_entry.to_le_bytes() {
        v.push(*b);
    }

    for i in 0..v.len() {
        res[i] = v[i];
    }

    return res;
}

fn entry_from_sector(sec: [u8; 512]) -> FileEntry {
    let res = FileEntry {
        signature: sec[0..4].try_into().expect("a"),
        filename: filename_from_slice(&sec[4..132]),
        parent_id: u64::from_le_bytes(sec[132..140].try_into().expect("b")),
        id: u64::from_le_bytes(sec[140..148].try_into().expect("c")),
        attributes: sec[149],
        t_creation: u64::from_le_bytes(sec[150..158].try_into().expect("d")),
        t_edit: u64::from_le_bytes(sec[158..166].try_into().expect("e")),
        owner: sec[166],
        size: u64::from_le_bytes(sec[166..174].try_into().expect("g")),
        start_sec: u64::from_le_bytes(sec[174..182].try_into().expect("h")),
        next_entry: u64::from_le_bytes(sec[182..190].try_into().expect("i")),
    };
    return res;
}

fn filename_from_string(s: String) -> [char; 128] {
    let mut res: [char; 128] = [' '; 128];

    let mut i = 0;
    for c in s.chars() {
        res[i] = c;
        i += 1;
    }
    return res;
}

fn filename_from_slice(slice: &[u8]) -> [char; 128] {
    let mut res: [char; 128] = [' '; 128];
    let mut i = 0;
    for b in slice {
        if i >= 128 || *b as char == ' ' { break; }
        res[i] = *b as char;
        i += 1;
    }
    return res;
}

fn string_from_filename(filename: [char; 128]) -> String {
    let mut res = String::from("");
    for c in filename.iter() {
        if *c == ' ' { break; }
        res.push(*c);
    }
    return res;
}

unsafe fn as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::core::slice::from_raw_parts(
        (p as *const T) as *const u8,
        mem::size_of::<T>(),
    )
}
