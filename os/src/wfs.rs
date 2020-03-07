//WFS: A shit filesystem
//Spec can be found at ../wfs_spec.txt

use crate::struct_tools;
use crate::timer;
use crate::vga_buffer;
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
use core::result::Result;
use alloc::string::ToString;

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
#[repr(C, packed)]
pub struct FileEntry {
    signature: [u8; 4],
    name: [char; 64],
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
            name: [' '; 64],
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

    let root_arr: [u8; 512] = [0; 512];
    let root_attributes: u8 = *0.set_bit(0, true).set_bit(1, true).set_bit(2, true);
    let root = FileEntry {
        name: vfs::nfs(String::from("")),
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

    let r = vfs::create_node(0, String::from("A:"), *0.set_bit(vfs::ATTR_DIR, true).set_bit(vfs::ATTR_SYS, true), 0, 0).unwrap();

    let mut s = entry_from_sector(ata::pio28_read(ata::ATA_HANDLER.lock().master, 1, 1));
    s.next_entry = 3;
    ata::pio28_write(ata::ATA_HANDLER.lock().master, 1, 1, sector_from_entry(s));
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

    vfs::install_device(String::from("A:"), vfs::System::WFS);
}

// VFS functions

pub fn find_node(parent_id: u64, name: String, dev_id: usize) -> Result<vfs::FsNode, vfs::Error> {
    match find_entry_by_name(parent_id, name.to_string()) {
        Ok(e) => {
            let entry = e;
            let node = vfs::FsNode {
                name: vfs::nfs(name.to_string()), 
                device: dev_id,
                parent_id: parent_id,
                id: entry.id,
                attributes: entry.attributes,
                t_creation: entry.t_creation,
                t_edit: entry.t_edit,
                owner: entry.owner,
                size: entry.size,
                open: false,
            };
            return Ok(node);
        },
        Err(e) => return Err(e),
    }
}

pub fn create_node(parent_id: u64, name: String, attributes: u8, owner: u8, dev_id: usize) -> Result<vfs::FsNode, vfs::Error> {
    match find_entry(parent_id) {
        Ok(mut parent) => {
            if !parent.attributes.get_bit(vfs::ATTR_DIR) {
                return Err(vfs::Error::ParentNotDirectory);
            }
            let entry = create_entry(name.to_string(), parent_id, attributes, owner);
            let node = vfs::FsNode {
                name: vfs::nfs(name.to_string()),
                device: dev_id,
                parent_id: parent_id,
                id: entry.id,
                attributes: entry.attributes,
                t_creation: entry.t_creation,
                t_edit: entry.t_edit,
                owner: entry.owner,
                size: entry.size,
                open: false,
            };

            //parent.size += 1;
            append_entry(parent, entry.location.to_le_bytes().to_vec());
            //ata::pio28_write(ata::ATA_HANDLER.lock().master, parent.location as usize, 1, sector_from_entry(parent));

            return Ok(node);
        },
        Err(e) => return Err(e),
    }

}

pub fn read_node(parent_id: u64, name: String) -> Result<Vec<u8>, vfs::Error> {
    match find_entry_by_name(parent_id, name) {
        Ok(e) => {
            return read_entry(e);
        },
        Err(e)=> return Err(e),
    }
}

pub fn write_node(parent_id: u64, name: String, buf: Vec<u8>) -> Result<(), vfs::Error> {
    match find_entry_by_name(parent_id, name) {
        Ok(e) => return write_entry(e, buf),
        Err(e) => return Err(e),
    }
}

pub fn append_node(parent_id: u64, name: String, buf:Vec<u8>) -> Result<(), vfs::Error> {
    match find_entry_by_name(parent_id, name) {
        Ok(e) => return append_entry(e, buf),
        Err(e) => return Err(e),
    }
}

pub fn delete_node(parent_id: u64, name: String) -> Result<(), vfs::Error> {
    match find_entry_by_name(parent_id, name) {
        Ok(e) => return delete_entry(e),
        Err(e) => return Err(e),
    }
}

pub fn get_root(dev_id: usize) -> Result<vfs::FsNode, vfs::Error> {
    match find_entry(1) {
        Ok(e) => {
            let node = vfs::FsNode {
                name: e.name, 
                device: dev_id,
                parent_id: e.parent_id,
                id: e.id,
                attributes: e.attributes,
                t_creation: e.t_creation,
                t_edit: e.t_edit,
                owner: e.owner,
                size: e.size,
                open: false,
            };
            return Ok(node);
        },
        Err(e) => return Err(e),
    }
}

pub fn find_node_by_id(id: u64, dev_id: usize) -> Result<vfs::FsNode, vfs::Error> {
    match find_entry(id) {
        Ok(e) => {
            let node = vfs::FsNode {
                name: e.name, 
                device: dev_id,
                parent_id: e.parent_id,
                id: e.id,
                attributes: e.attributes,
                t_creation: e.t_creation,
                t_edit: e.t_edit,
                owner: e.owner,
                size: e.size,
                open: false,
            };
            return Ok(node);
        },
        Err(e) => return Err(e),
    }
}

pub fn get_children(parent_id: u64, name: String, dev_id: usize) -> Result<Vec<vfs::FsNode>, vfs::Error> {
    match find_entry_by_name(parent_id, name) {
        Ok(e) => {
            let entries = get_entry_children(e)?;
            let mut ret: Vec<vfs::FsNode> = Vec::with_capacity(entries.len());

            for entry in entries {
                ret.push(vfs::FsNode {
                    name: entry.name, 
                    device: dev_id,
                    parent_id: e.id,
                    id: entry.id,
                    attributes: entry.attributes,
                    t_creation: entry.t_creation,
                    t_edit: entry.t_edit,
                    owner: entry.owner,
                    size: entry.size,
                    open: false,
                });
            }
            
            return Ok(ret);
        },
        Err(e) => return Err(e),
    }
}

pub fn get_parent(id: u64, dev_id: usize) -> Result<vfs::FsNode, vfs::Error> {
    if id != 1 {
        let mut e = find_entry(id)?;
        println!("ASDF");
        e = find_entry(e.parent_id)?;
        let node = vfs::FsNode {
            name: e.name, 
            device: dev_id,
            parent_id: e.parent_id,
            id: e.id,
            attributes: e.attributes,
            t_creation: e.t_creation,
            t_edit: e.t_edit,
            owner: e.owner,
            size: e.size,
            open: false,
        };
        return Ok(node);
    } else {
        let e = find_entry(id)?;
        let node = vfs::FsNode {
            name: e.name, 
            device: dev_id,
            parent_id: e.parent_id,
            id: e.id,
            attributes: e.attributes,
            t_creation: e.t_creation,
            t_edit: e.t_edit,
            owner: e.owner,
            size: e.size,
            open: false,
        };
        return Ok(node);
    }

}

//WFS specific functions

fn read_entry(entry: FileEntry) -> Result<Vec<u8>, vfs::Error> {
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

        let next = u64::from_le_bytes(raw[4..12].try_into().expect(""));

        if next == FREE || next == RESERVED {
            return Err(vfs::Error::ReadError);
        }
        
        for b in &raw[12..512] {
            if written >= entry.size as usize {
                break;
            }
            ret.push(*b);
            written += 1;
        }    

        if next == END_OF_CHAIN {
            break;
        }

        lba = next as usize;
    }

    return Ok(ret);
}

fn delete_entry(entry: FileEntry) -> Result<(), vfs::Error> {
    ata::pio28_write(ata::ATA_HANDLER.lock().master, entry.location as usize, 1, [0; 512]);
    WFS_INFO.lock().blocks_in_use -= 1;
    update_info();

    let mut parent = find_entry(entry.parent_id)?;
    let mut buf = read_entry(parent)?;
    
    let mut j = 0;
    loop {
        if u64::from_le_bytes(buf[j*8..j*8+8].try_into().expect("")) == entry.location {
            for i in j*8..j*8+8 {
                buf.remove(j*8);
            }
            write_entry(parent, buf)?;
            break;
        }
        j += 1;
    }

    let mut prev = entry_from_sector(ata::pio28_read(ata::ATA_HANDLER.lock().master, entry.prev_entry as usize, 1));
    prev.next_entry = entry.next_entry;
    ata::pio28_write(ata::ATA_HANDLER.lock().master, prev.location as usize, 1, sector_from_entry(prev));

    if entry.size == 0 || entry.start_sec == 0 || entry.start_sec == END_OF_CHAIN {
        return Ok(());
    }

    let mut lba = entry.start_sec as usize;
    loop {
        let raw = ata::pio28_read(ata::ATA_HANDLER.lock().master, lba, 1);
        let next = u64::from_le_bytes(raw[4..12].try_into().expect(""));
        if next == END_OF_CHAIN {
            break;
        }
        //TODO: Not make this fucking stupid
        for i in 0..1000 {}
        ata::pio28_write(ata::ATA_HANDLER.lock().master, next as usize, 1, [0; 512]); 
        for i in 0..1000 {}
        WFS_INFO.lock().blocks_in_use -= 1;
        lba = next as usize;
    }
    update_info();

    if entry.attributes.get_bit(vfs::ATTR_DIR) {
        for e in get_entry_children(entry)? {
            delete_entry(e);
        }
    }

    if entry.attributes.get_bit(vfs::ATTR_DIR) {
        for c in get_entry_children(entry)?.iter() {
            delete_entry(*c)?;
        }
    }

    Ok(())
} 

fn create_entry(name: String, parent_id: u64, attributes: u8, owner: u8) -> FileEntry {
    WFS_INFO.lock().files += 1;
    WFS_INFO.lock().blocks_in_use += 1;
    update_info();

    let block = find_empty_blocks(1)[0];

    let f = WFS_INFO.lock().files;

    let entry = FileEntry {
        signature: DATA_SIG,
        name: vfs::nfs(name),
        parent_id: parent_id,
        id: f,
        attributes: attributes,
        t_creation: 0,      //TODO: implement global time fmt fitting in u64
        t_edit: 0,
        owner: owner,
        size: 0,
        start_sec: END_OF_CHAIN,
        next_entry: END_OF_CHAIN,
        prev_entry: WFS_INFO.lock().final_entry,
        location: block as u64,
    };
    let arr = sector_from_entry(entry);
    ata::pio28_write(ata::ATA_HANDLER.lock().master, entry.location as usize, 1, arr); 

    let mut prev = entry_from_sector(ata::pio28_read(ata::ATA_HANDLER.lock().master, WFS_INFO.lock().final_entry as usize, 1));

    for i in 0..1000 {};

    prev.next_entry = entry.location;
    ata::pio28_write(ata::ATA_HANDLER.lock().master, prev.location as usize, 1, sector_from_entry(prev));


    WFS_INFO.lock().final_entry = entry.location;
    update_info();

    return entry;
}

fn write_entry(e: FileEntry, buf: Vec<u8>) -> Result<(), vfs::Error> {
    let mut entry = e;

    if entry.size > 0 {
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

    let mut sec_count = 0;

    if buf.len() % 500 == 0 {
        sec_count = buf.len() / 500;
    } else {
        sec_count = (buf.len() - (buf.len() % 500)) / 500 + 1;
    }
    let mut data: Vec<[u8; 500]> = Vec::with_capacity(sec_count);

    let mut j = 0;
    for i in 0..sec_count {
        let mut sec: [u8; 500] = [0; 500];

        let mut k = 0;
        for l in j..j + 500 {
            if l >= buf.len() {
                break;
            }
            sec[k] = buf[l];
            k += 1;
            j += 1;
        }

        data.push(sec);
    }

    let fblock = find_empty_blocks(1)[0];

    WFS_INFO.lock().blocks_in_use += 1;
    update_info();

    entry.start_sec = fblock as u64;
    entry.size = buf.len() as u64;
    ata::pio28_write(ata::ATA_HANDLER.lock().master, entry.location as usize, 1, sector_from_entry(entry));

    let mut block = fblock;
    let mut next = block;
    for i in 0..data.len() {
        let mut sec = [0u8; 512];
        
        for j in 0..4 {
            sec[j] = DATA_SIG[j];
            //print!("{} ", sec[j]);
        }
        
        for j in 12..512 {
            sec[j] = data[i][j - 12];
            
        }

        ata::pio28_write(ata::ATA_HANDLER.lock().master, block, 1, sec);
        WFS_INFO.lock().blocks_in_use += 1;

        next = find_empty_blocks(1)[0];
        WFS_INFO.lock().blocks_in_use += 1;
        update_info();

        let mut j = 4;
        if i == data.len() - 1 {
            for b in &END_OF_CHAIN.to_le_bytes() {
                sec[j] = *b;
                j += 1;
            }
        } else {
            for b in &next.to_le_bytes() {
                sec[j] = *b;
                j += 1;
            }
        }
        ata::pio28_write(ata::ATA_HANDLER.lock().master, block, 1, sec);

        let buffer = ata::pio28_read(true, block, 1);
        let sig: [u8; 4] = buffer[0..4].try_into().expect("");
        let bn = u64::from_le_bytes(buffer[4..12].try_into().expect(""));

        block = next;
    }

    Ok(())
}

fn append_entry(e: FileEntry, mut b: Vec<u8>) -> Result<(), vfs::Error> {
    if e.size == 0 {
        return write_entry(e, b);
    }

    let mut buf = b;
    let finsize = e.size + buf.len() as u64;
    let mut entry = e;
    entry.size = finsize;

    let offset = (e.size as usize) % 500 + 12;


    let mut sec: [u8; 512] = [0; 512];

    let mut next = entry.start_sec;
    loop {
        let raw = ata::pio28_read(ata::ATA_HANDLER.lock().master, next as usize, 1);
        let n = u64::from_le_bytes(raw[4..12].try_into().expect(""));

        if n == END_OF_CHAIN {
            sec = raw;
            break;
        }

        next = n;
    }

    let mut j = 0;

    if buf.len() <= 512 - offset {
        for i in offset..offset + buf.len() {
            sec[i] = buf[j];
            j += 1;
        }
    } else {
        for i in offset..512 {
            sec[i] = buf[j];
            j += 1;
        }
    }


    ata::pio28_write(ata::ATA_HANDLER.lock().master, entry.location as usize, 1, sector_from_entry(entry));

    if offset + buf.len() - 12 <= 512 {
        ata::pio28_write(ata::ATA_HANDLER.lock().master, next as usize, 1, sec);
        return Ok(());
    }

    if buf.len() <= 512 - offset {
        for i in offset..offset + buf.len() - 1 {
            buf.remove(i);
        }
    } else {
        for i in 0..512 - offset {
            buf.remove(i);
        }
    }

    let sec_count = buf.len() - (buf.len() % 500) + 1;
    let mut blocks = find_empty_blocks(sec_count);

    for i in 0..4 {
        sec[i] = DATA_SIG[i];
    }

    let mut j = 4;
    for b in &blocks[0].to_le_bytes() {
        sec[j] = *b;
        j += 1;
    }
    
    ata::pio28_write(ata::ATA_HANDLER.lock().master, next as usize, 1, sec);
    let mut l = 0;
    for i in 0..sec_count {
        let mut j = 4;

        if i == sec_count - 1 {
            for b in &END_OF_CHAIN.to_le_bytes() {
                sec[j] = *b;
                j += 1;
            }
        } else {
            for b in &blocks[i + 1].to_le_bytes() {
                sec[j] = *b;
                j += 1;
            }
        }
        for k in 12..512 {
            if l >= buf.len() {
                break;
            }
            sec[k] = buf[l];
            l += 1;
        }   
        ata::pio28_write(ata::ATA_HANDLER.lock().master, blocks[i], 1, sec);
    }

    return Ok(());
}

fn find_entry(id: u64) -> Result<FileEntry, vfs::Error> {
    let m = ata::ATA_HANDLER.lock().master;

    let mut temp = entry_from_sector(ata::pio28_read(m, 1, 1));
    loop {
        if temp.id == id {
            return Ok(temp);
        }

        if temp.next_entry == END_OF_CHAIN  {
            return Err(vfs::Error::FileNotFound);
        }
        temp = entry_from_sector(ata::pio28_read(m, temp.next_entry as usize, 1));
    }
}

fn find_entry_by_name(parent_id: u64, name: String) -> Result<FileEntry, vfs::Error> {
    let m = ata::ATA_HANDLER.lock().master;

    match find_entry(parent_id) {
        Ok(parent) => {
            let locations = read_entry(parent)?; 
            
            for i in 0..locations.len() / 8 {
                if i * 8 + 8 > locations.len() {
                    return Err(vfs::Error::FileNotFound);
                }
                let e = entry_from_sector(ata::pio28_read(ata::ATA_HANDLER.lock().master, u64::from_le_bytes(locations[i*8..i*8+8].try_into().expect("")) as usize, 1));

                if vfs::sfn(e.name) == name {
                    return Ok(e);
                }
            }
        
            return Err(vfs::Error::FileNotFound);
        },
        Err(e) => return Err(e),
    }
}

fn get_entry_children(e: FileEntry) -> Result<Vec<FileEntry>, vfs::Error> {
    if !e.attributes.get_bit(vfs::ATTR_DIR) {
        return Err(vfs::Error::IllegalOperation);
    }

    let locations = read_entry(e)?;
    let mut res: Vec<FileEntry> = Vec::with_capacity(e.size as usize / 8);

    for i in 0..locations.len() / 8 {
        if i * 8 + 8 > locations.len() {
            break;
        }
        let e = entry_from_sector(ata::pio28_read(ata::ATA_HANDLER.lock().master, u64::from_le_bytes(locations[i*8..i*8+8].try_into().expect("")) as usize, 1));
        res.push(e);
    }

    Ok(res)
}


fn find_empty_blocks(n: usize) -> Vec<usize> {
    let mut res: Vec<usize> = Vec::with_capacity(n);

    let mut off = 1;

    for _ in 0..n {
        if res.contains(&((WFS_INFO.lock().blocks_in_use + off) as usize)) {
            off += 1;
        } else {
            let first = ata::pio28_read(ata::ATA_HANDLER.lock().master, (WFS_INFO.lock().blocks_in_use + off) as usize, 1);
            if String::from_utf8_lossy(&first[0..=3]) != String::from("DATA") {
                res.push((WFS_INFO.lock().blocks_in_use + off) as usize);
                off += 1;
                continue;
            }
        }

        for i in 1..WFS_INFO.lock().blocks as usize {
            if res.contains(&i) {
                continue;
            }

            let raw = ata::pio28_read(ata::ATA_HANDLER.lock().master, i, 1);
            if String::from_utf8_lossy(&raw[0..=3]) != String::from("DATA") {
                res.push(i);
                break;
            }    
        }
    }

    return res;
}


fn sector_from_entry(f: FileEntry) -> [u8; 512] {
    let mut res: [u8; 512] = [0; 512];
    /*let mut i = 0;
    for b in f.signature.iter() {
        res[i] = *b;
        i += 1;
    }
    for b in f.name.iter() {
        res[i] = *b as u8;
        i += 1;
    }
    for b in &f.parent_id.to_le_bytes() {
        res[i] = *b;
        i += 1;
    }
    for b in &f.id.to_le_bytes() {
        res[i] = *b;
        i += 1;
    }
    res[i] = f.attributes;
    i += 1;
    for b in &f.t_creation.to_le_bytes() {
        res[i] = *b;
        i += 1;
    }
    for b in &f.t_edit.to_le_bytes() {
        res[i] = *b;
        i += 1;
    }
    res[i] = f.owner;
    i += 1;
    for b in &f.size.to_le_bytes() {
        res[i] = *b;
        i += 1;
    }
    for b in &f.start_sec.to_le_bytes() {
        res[i] = *b;
        i += 1;
    }
    for b in &f.next_entry.to_le_bytes() {
        res[i] = *b;
        i += 1;
    }
    for b in &f.prev_entry.to_le_bytes() {
        res[i] = *b;
        i += 1;
    }
    for b in &f.location.to_le_bytes() {
        res[i] = *b;
        i += 1;
    }*/

    unsafe {
        let slice = struct_tools::to_slice(&f);
        for i in 0..slice.len() {
            res[i] = slice[i];
        }

        return res;
    }
}

fn entry_from_sector(sec: [u8; 512]) -> FileEntry {
    /*let res = FileEntry {
        signature: sec[0..4].try_into().expect("sig"),
        name: name_from_slice(&sec[4..132]),
        parent_id: u64::from_le_bytes(sec[132..140].try_into().expect("pid")),
        id: u64::from_le_bytes(sec[140..148].try_into().expect("id")),
        attributes: sec[148],
        t_creation: u64::from_le_bytes(sec[150..158].try_into().expect("tc")),
        t_edit: u64::from_le_bytes(sec[158..166].try_into().expect("te")),
        owner: sec[166],
        size: u64::from_le_bytes(sec[166..174].try_into().expect("sz")),
        start_sec: u64::from_le_bytes(sec[174..182].try_into().expect("ss")),
        next_entry: u64::from_le_bytes(sec[182..190].try_into().expect("ne")),
        prev_entry: u64::from_le_bytes(sec[190..198].try_into().expect("pe")),
        location: u64::from_le_bytes(sec[198..206].try_into().expect("l")),
    };*/

    let buf = &sec;
    let ptr: *const FileEntry = unsafe { mem::transmute(buf.as_ptr()) };
    let res: FileEntry = unsafe { *ptr };

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

fn name_from_slice(slice: &[u8]) -> [char; 64] {
    let mut res: [char; 64] = [' '; 64];
    let mut i = 0;
    for b in slice {
        if i >= 64 || *b as char == ' ' { break; }
        res[i] = *b as char;
        i += 1;
    }
    return res;
}

pub fn demo() {
    println!("[Demo] Creating and opening file 'test'...");
    let mut n = vfs::create_node(0, String::from("test"), 0, 0, 0).unwrap();
    n.open();

    println!("[Demo] Writing to file...");
    n.write(b"Hello, world!\n".to_vec()).unwrap();

    println!("[Demo] Reading file...\n");
    let buffer = n.read().unwrap();

    vga_buffer::set_color(vga_buffer::Color::LightBlue, vga_buffer::Color::Black);
    for b in buffer {
        print!("{}", b as char);
    } 
    vga_buffer::set_color(vga_buffer::Color::White, vga_buffer::Color::Black);

    println!("\n[Demo] Appending data...");
    n.append(b"This is another line!\n".to_vec()).unwrap();

    println!("[Demo] Reading again...\n");
    let buffer = n.read().unwrap();

    vga_buffer::set_color(vga_buffer::Color::LightBlue, vga_buffer::Color::Black);
    for b in buffer {
        print!("{}", b as char);   
    }
    vga_buffer::set_color(vga_buffer::Color::White, vga_buffer::Color::Black);

    println!("\n[Demo] Closing file...");
    n.close();
}

