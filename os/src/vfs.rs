use lazy_static::lazy_static;
use spin::Mutex;
use crate::initrd;

pub const FS_FILE: u32      = 0x01;
pub const FS_DIR: u32       = 0x02;
pub const FS_CHARDEV: u32   = 0x03;
pub const FS_BLOCKDEV: u32  = 0x04;
pub const FS_PIPE: u32      = 0x05;
pub const FS_SYMLINK: u32   = 0x06;
pub const FS_MNTPOINT: u32  = 0x08;

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum System {
    Initrd,
}

lazy_static! {
    pub static ref FS_ROOT: Mutex<FsNode> = Mutex::new(FsNode {
        name: "",
        system: System::Initrd,
        mask: 0,
        flags: 0,
        inode: 0,
        length: 0,
        impln: 0,
        ptr: None,
    });
}

#[derive(Clone)]
#[repr(C)]
pub struct Dirent {
    pub name: &'static str,
    pub ino:  u32,
}

#[derive(Eq, PartialEq, Clone, Copy)]
#[repr(C)]
pub struct FsNode {
    pub name:   &'static str,
    pub system: System,
    pub mask:   u32,
    pub flags:  u32,
    pub inode:  u32,
    pub length: u32,
    pub impln:  u32,

    pub ptr: Option<&'static FsNode>,
}

pub fn read_fs(node: FsNode, offset: u32, size: u32, buffer:  &mut u8) -> u32 {
    match node.system {
        System::Initrd => initrd::INITRD.lock().read(node, offset, size, buffer),
    }
}

/*TODO: Implement write, open, and close for InitRD
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

pub fn readdir_fs(node: FsNode, index: u32) -> Option<Dirent> {
    if (node.flags&0x7) == FS_DIR {
        match node.system {
            System::Initrd => {
                match initrd::INITRD.lock().readdir(node, index) {
                    Some(d) => return Some(d),
                    None => return None,
                }
            },
        }
    } else {
        None
    }
}

pub fn finddir_fs(node: FsNode, name: &'static str) -> Option<FsNode> {
    if (node.flags&0x7) == FS_DIR {
        match node.system {
            System::Initrd => {
                match initrd::INITRD.lock().finddir(node, name) {
                    Some(n) => return Some(n),
                    None => return None,
                }
            },
        }
    } else {
        None
    }
}

