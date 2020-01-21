use crate::initrd_img;
use crate::vfs;
use spin::Mutex;
use lazy_static::lazy_static;
use crate::{print, println};
use alloc::vec::Vec;
use core::mem;
use alloc::string::String;
use core::str;

#[repr(C)]
pub struct Initrd {
    pub nfiles: u8,
    pub file_headers: Vec<FileHeader>,
    pub root: vfs::FsNode,
    pub dev: vfs::FsNode,
    pub root_nodes: Vec<vfs::FsNode>,
    pub nroot_nodes: u32,
    pub dirent: vfs::Dirent,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FileHeader {
    name: [char; 30],
    size: u32,
    offset: u32,
}

lazy_static! {
    pub static ref INITRD: Mutex<Initrd> = Mutex::new(Initrd { 
        nfiles: 0,
        file_headers: Vec::new(),
        root: vfs::FsNode {
            name: String::from("initrd"),
            system: vfs::System::Initrd,
            mask: 0,
            flags: 0,
            inode: 0,
            length: 0,
            impln: 0,
            ptr: None,
        },
        dev: vfs::FsNode {
            name: String::from("dev"),
            system: vfs::System::Initrd,
            mask: 0,
            flags: vfs::FS_DIR,
            inode: 0,
            length: 0,
            impln: 0,
            ptr: None,
        },
        root_nodes: Vec::new(),
        nroot_nodes: 0,
        dirent: vfs::Dirent {
            name: String::from(""),
            ino: 0,
        },
    });
}

pub fn read(node: vfs::FsNode) -> &'static [u8] {
    let header = INITRD.lock().file_headers[node.inode as usize];
    let mut buf = &initrd_img::IMG[(header.offset as usize)..header.offset as usize + header.size  as usize];
    buf
}

pub fn readdir(node: &vfs::FsNode, index: u32) -> Option<vfs::Dirent> {
    unsafe { INITRD.force_unlock() }
    if *node == INITRD.lock().root && index == 0 {
        return Some(vfs::Dirent {
            name: String::from("dev"),
            ino: 0,
        });
    }
    if index >= INITRD.lock().nroot_nodes { 
        return None; 
    }
    
    let t = INITRD.lock().root_nodes[index as usize].name.clone();
    INITRD.lock().dirent.name = t;

    let t2 = INITRD.lock().root_nodes[index as usize].inode;
    INITRD.lock().dirent.ino = t2;

    return Some(INITRD.lock().dirent.clone());
}


pub fn finddir(node: &vfs::FsNode, name: String) -> Option<vfs::FsNode> {
    unsafe { INITRD.force_unlock() }
    if *node == INITRD.lock().root && name != String::from("dev") {
        return Some(INITRD.lock().dev.clone());
    }
    if INITRD.lock().root_nodes.len() > 0 {
        unsafe { INITRD.force_unlock() }
        for i in 0..INITRD.lock().nroot_nodes {
            unsafe { INITRD.force_unlock() }
            if name == INITRD.lock().root_nodes[i as usize].name {
                unsafe { INITRD.force_unlock() }
                return Some(INITRD.lock().root_nodes[i as usize].clone());
            }
        }
    }
    return None;
}

pub fn init() {
    INITRD.lock().nfiles = initrd_img::IMG[0];
 
    INITRD.lock().nroot_nodes = initrd_img::IMG[0] as u32;

    let mut offset = 1;
    for i in 0..INITRD.lock().nfiles {
        let header_size = mem::size_of::<FileHeader>();
        let mut buffer = &initrd_img::IMG[offset..offset + header_size];
        
        let ptr: *const FileHeader = unsafe { mem::transmute(buffer.as_ptr()) };
        let header: FileHeader = unsafe { *ptr };

        unsafe { INITRD.force_unlock() };
        INITRD.lock().file_headers.push(header);

        INITRD.lock().root_nodes.push(vfs::FsNode {
            name: String::from(&osfn(header.name)),
            system: vfs::System::Initrd,
            mask: 0,
            flags: vfs::FS_FILE,
            inode: i as u32,
            length: header.size,
            impln: 0,
            ptr: None,
        });

        offset += header_size;
    }
    //println!("{}", str::from_utf8(&initrd_img::IMG[offset..initrd_img::IMG.len()]).unwrap());
}

pub fn osfn(name: [char; 30]) -> String {
    let mut res = String::new();
    for i in 0..30 {
        if name[i] == '\0' {
            break;
        } else {
            res.push(name[i]);
        }
    }
    res
}

