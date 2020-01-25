use lazy_static::lazy_static;
use spin::Mutex;
use crate::initrd;
use alloc::string::String;
use crate::println;
use alloc::vec::Vec;

pub const FS_FILE: u32      = 0x01;
pub const FS_DIR: u32       = 0x02;
pub const FS_CHARDEV: u32   = 0x03;
pub const FS_BLOCKDEV: u32  = 0x04;
pub const FS_PIPE: u32      = 0x05;
pub const FS_SYMLINK: u32   = 0x06;
pub const FS_MNTPOINT: u32  = 0x08;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum System {
    Initrd,
}

pub struct FsRoot {
    pub node: FsNode,
}

pub struct CDir {
    pub dirent: Dirent,
}

lazy_static! {
    pub static ref FS_ROOT: Mutex<FsRoot> = Mutex::new(FsRoot { node: FsNode {
        name: String::from("/"),
        system: System::Initrd,
        mask: 0,
        flags: 2,
        inode: 0,
        length: 0,
        children: Vec::new(),
    }});

    pub static ref CDIR: Mutex<CDir> = Mutex::new(CDir { dirent: Dirent {
        name: FS_ROOT.lock().node.name.clone(),
        ino: FS_ROOT.lock().node.inode,
    }});
}

#[derive(Clone)]
#[repr(C)]
pub struct Dirent {
    pub name: String,
    pub ino:  u32,
}

#[derive(Debug, Eq, PartialEq, Clone)]
#[repr(C)]
pub struct FsNode {
    pub name:   String,
    pub system: System,
    pub mask:   u32,
    pub flags:  u32,
    pub inode:  u32,
    pub length: u32,
    pub children: Vec<*const FsNode>,
}

unsafe impl Send for FsNode {}

pub fn read_fs(node: FsNode) -> &'static [u8] {
    match node.system {
        System::Initrd => initrd::read(node),
    }
}

pub fn get_child(node: &FsNode, name: String) -> Option<FsNode> {
    if node.children.len() > 0 {
       for i in 0..node.children.len() {
           unsafe {
               if (*node.children[i]).name == name {
                   return Some((*node.children[i]).clone());
                }
            }
        }
       return None;
    } else {
        return None;
    }
}

pub fn get_nth_child(node: &FsNode, index: usize) -> Option<FsNode> {
    if node.children.len() != 0 && node.children.len() >= index + 1 {
        unsafe { Some((*node.children[index]).clone()) }
    } else {
        None
    }
}

/*
TODO: Implement write, open, and close for InitRD
pub fn write_fs(node: FsNode, offset: u32, size: u32, buffer: u8) -> u32 {
    match node.system {
        System::Initrd => initrd::INITRD.lock().write(node, offset, size, buffer),
    }
}


pub fn open_fs(node: FsNode, read: u8, write: u8) {
    match node.system {
        System::Initrd => initrd::INITRD.lock().open(node, read, write),
        None => (),
    }
}

pub fn close_fs(node: FsNode) {
    match node.system {
        System::Initrd => initrd::INITRD.lock().close(node),
        None => (),
    }
}
*/

pub fn readdir_fs(node: &FsNode, index: u32) -> Option<Dirent> {
    if (node.flags&0x7) == FS_DIR {
        match node.system {
            System::Initrd => {
                match initrd::readdir(node, index) {
                    Some(d) => return Some(d),
                    None => return None,
                }
            },
        }
    } else {
        None
    }
}

pub fn finddir_fs(node: &FsNode, name: String) -> Option<FsNode> {
    if (node.flags&0x7) == FS_DIR {
        match node.system {
            System::Initrd => {
                match initrd::finddir(node, name) {
                    Some(n) => return Some(n),
                    None => return None,
                }
            },
        }
    } else {
        None
    }
}

pub fn get_cdir_dirent() -> Dirent {
    CDIR.lock().dirent.name = initrd::INITRD.lock().dev.name.clone();
    CDIR.lock().dirent.ino = initrd::INITRD.lock().dev.inode;
    return CDIR.lock().dirent.clone();
}

pub fn get_cdir_path() -> String {
    CDIR.lock().dirent.name = initrd::INITRD.lock().dev.name.clone();
    CDIR.lock().dirent.ino = initrd::INITRD.lock().dev.inode;
    let mut res = String::from("/");
    res.push_str(&CDIR.lock().dirent.name);
    res.push_str("/");
    res
}
