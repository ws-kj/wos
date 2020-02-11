//WFS: A shit filesystem
//Spec can be found at ../wfs_spec.txt

use crate::vfs;
use crate::timer;
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
use core::convert::AsMut;

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
    final_entry: u64,
}

#[derive(Clone, Copy)]
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
    prev_entry: u64,
    location: u64,
}
impl Default for FileEntry {
    fn default() -> FileEntry {
        FileEntry {
            signature: [0; 4],
            filename: [' '; 128],
            parent_id: 0,
            id: 0,
            attributes: 0,
            t_creation: 0,
            t_edit: 0,
            owner: 0,
            size: 0,
            start_sec: 0,
            next_entry: 0,
            prev_entry: 0,
            location: 0,
        }
    }
}

#[repr(C)]
struct DataSector {
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
        final_entry: 0,
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
    WFS_INFO.lock().final_entry = 1;

    println!("[WFS] Writing InfoBlock to ATA drive.");
    update_info();

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
        prev_entry: END_OF_CHAIN,
        location: 1,
    };
    let root_arr = sector_from_entry(root);
    println!("[WFS] Writing Root file entry to ATA drive.");
    ata::pio28_write(ata::ATA_HANDLER.lock().master, 1, 1, root_arr);
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
    WFS_INFO.lock().final_entry = u64::from_le_bytes(info_block[41..49].try_into().expect(""));
    
}

pub fn read_file(entry: FileEntry) -> Option<Vec<u8>> {
    let mut sec_count = 0;

    if entry.size % 500 == 0 {
        sec_count = entry.size / 500;
    } else {
        sec_count = (entry.size - (entry.size % 500)) + 1;
    }
  
    let mut ret: Vec<u8> = Vec::new();
    let mut lba = entry.start_sec as usize;

    let mut written: usize = 0;
    for i in 0..sec_count {
        let raw = ata::pio28_read(ata::ATA_HANDLER.lock().master, lba, 1);

        if raw[0..4] != DATA_SIG {
            return None;
        } 

        let next = u64::from_le_bytes(raw[4..12].try_into().expect(""));

        if next == FREE || next == RESERVED {
            return None;
        }
        
        if next == END_OF_CHAIN {
            break;
        }

        for b in &raw[13..=512] {
            if written >= entry.size as usize - 1 {
                break;
            }
            ret.push(*b);
            written += 1;
        }    

        lba = next as usize;
    }

    return Some(ret);
}

pub fn delete_file(id: u64) {
    let mut entry: FileEntry = Default::default(); 
    match find_entry(id) {
        Some(e) => entry = e,
        None => return,
    }
    ata::pio28_write(ata::ATA_HANDLER.lock().master, entry.location as usize, 1, [0; 512]);

    let mut prev = entry_from_sector(ata::pio28_read(ata::ATA_HANDLER.lock().master, entry.prev_entry as usize, 1));
    prev.next_entry = entry.next_entry;
    ata::pio28_write(ata::ATA_HANDLER.lock().master, prev.location as usize, 1, sector_from_entry(prev));

    if entry.size == 0 || entry.start_sec == 0 || entry.start_sec == END_OF_CHAIN {
        return;
    }

    let mut lba = entry.start_sec as usize;
    loop {
        let raw = ata::pio28_read(ata::ATA_HANDLER.lock().master, lba, 1);
        let next = u64::from_le_bytes(raw[4..12].try_into().expect(""));
        if next == END_OF_CHAIN {
            break;
        }
        //TODO: Not make this fucking stupid
        ata::pio28_write(ata::ATA_HANDLER.lock().master, next as usize, 1, [0; 512]); 

        lba = next as usize;
    }

} 

pub fn create_entry(filename: String, parent_id: u64, attributes: u8, owner: u8) -> FileEntry {
    WFS_INFO.lock().files += 1;
    WFS_INFO.lock().blocks_in_use += 1;
    update_info();

    let block = find_empty_block();

    let f = WFS_INFO.lock().files;

    let entry = FileEntry {
        signature: DATA_SIG,
        filename: filename_from_string(filename),
        parent_id: parent_id,
        id: f,
        attributes: attributes,
        t_creation: 0,      //TODO: implement global time format fitting in u64
        t_edit: 0,
        owner: owner,
        size: 0,
        start_sec: END_OF_CHAIN,
        next_entry: END_OF_CHAIN,
        prev_entry: WFS_INFO.lock().final_entry,
        location: block as u64,
    };
    let arr = sector_from_entry(entry);
    
    let mut prev = entry_from_sector(ata::pio28_read(ata::ATA_HANDLER.lock().master, WFS_INFO.lock().final_entry as usize, 1));
    prev.next_entry = entry.location;
    let prev_arr = sector_from_entry(prev);
    ata::pio28_write(ata::ATA_HANDLER.lock().master, prev.location as usize, 1, prev_arr);
    
    WFS_INFO.lock().final_entry = entry.location;
    update_info();

    ata::pio28_write(ata::ATA_HANDLER.lock().master, entry.location as usize, 1, arr); 

    return entry;
}

pub fn write_file(id: u64, buf: Vec<u8>) {
    let mut entry: FileEntry = Default::default();
    match find_entry(id) {
        Some(e) => entry = e,
        None => return,
    }

    let mut sec_count = 0;

    if buf.len() % 500 == 0 {
        sec_count = buf.len() / 500;
    } else {
        sec_count = (buf.len() - (buf.len() % 500)) + 1;
    }

    let mut data: Vec<[u8; 500]> = Vec::with_capacity(sec_count);

    let mut j = 0;
    for i in 0..sec_count {
        let mut sec: [u8; 500] = [0; 500];

        if j + 500 > buf.len() {
            for l in j..buf.len() {
                sec[l] = buf[l];
            }    
        } else {
            for l in j..j + 500 {
                sec[l] = buf[l];
            }
        }

        data.push(sec);
        j += 500;
    }

    let fblock = find_empty_block();
    entry.start_sec = fblock as u64;
    ata::pio28_write(ata::ATA_HANDLER.lock().master, entry.location as usize, 1, sector_from_entry(entry));

    let mut block = fblock; 
    for i in 0..data.len() {
        let mut sec = [0u8; 512];
        
        for j in 0..4 {
            sec[j] = DATA_SIG[j];
        }
        
        for j in 12..512 {
            sec[j] = data[i][j - 12];
        }

        ata::pio28_write(ata::ATA_HANDLER.lock().master, block, 1, sec);

        let next = find_empty_block();

        let mut j = 4;
        for b in &next.to_le_bytes() {
            sec[j] = *b;
            j += 1;
        }

        ata::pio28_write(ata::ATA_HANDLER.lock().master, block, 1, sec);

        block = next;
    }

}

fn find_entry(id: u64) -> Option<FileEntry> {
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

fn find_entry_by_name(parent_id: u64, name: String) -> Option<FileEntry> {
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

fn find_empty_block() -> usize {
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
    for b in &f.prev_entry.to_le_bytes() {
        v.push(*b);
    }
    for b in &f.location.to_le_bytes() {
        v.push(*b); 
    }
    for i in 0..v.len() {
        res[i] = v[i];
    }

    return res;
}

fn entry_from_sector(sec: [u8; 512]) -> FileEntry {
    let res = FileEntry {
        signature: sec[0..4].try_into().expect("sig"),
        filename: filename_from_slice(&sec[4..132]),
        parent_id: u64::from_le_bytes(sec[132..140].try_into().expect("pid")),
        id: u64::from_le_bytes(sec[140..148].try_into().expect("id")),
        attributes: sec[149],
        t_creation: u64::from_le_bytes(sec[150..158].try_into().expect("tc")),
        t_edit: u64::from_le_bytes(sec[158..166].try_into().expect("te")),
        owner: sec[166],
        size: u64::from_le_bytes(sec[166..174].try_into().expect("sz")),
        start_sec: u64::from_le_bytes(sec[174..182].try_into().expect("ss")),
        next_entry: u64::from_le_bytes(sec[182..190].try_into().expect("ne")),
        prev_entry: u64::from_le_bytes(sec[190..198].try_into().expect("pe")),
        location: u64::from_le_bytes(sec[198..206].try_into().expect("l")),
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

fn update_info() {
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
    for b in &WFS_INFO.lock().final_entry.to_le_bytes() {
        bufv.push(*b);
    }

    let mut info: [u8; 512] = [0; 512];
    for i in 0..bufv.len() {
        info[i] = bufv[i];
    }

    ata::pio28_write(ata::ATA_HANDLER.lock().master, 0, 1, info);
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

