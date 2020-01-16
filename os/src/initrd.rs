use lazy_static::lazy_static;
use spin::Mutex;
use alloc::vec::Vec;
use crate::vfs;
use core::ptr::copy_nonoverlapping;
use core::ptr;
use core::mem;
use crate::println;

#[repr(C)]
pub struct InitrdHeader {
    pub nfiles: u32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct InitrdFileHeader {
    pub magic: u8,
    pub name: &'static str,
    pub offset: u32,
    pub length: u32,
}

#[repr(C)]
pub struct Initrd {
    pub header: Option<&'static mut InitrdHeader>,
    pub file_headers: Vec<InitrdFileHeader>,
    pub root: Option<vfs::FsNode>,
    pub dev: Option<vfs::FsNode>,
    pub root_nodes: Vec<vfs::FsNode>,
    pub nroot_nodes: u32,
    pub dirent: Option<vfs::Dirent>,
}

lazy_static! {
    pub static ref INITRD: Mutex<Initrd> = Mutex::new(Initrd {
        header: None,
        file_headers: Vec::new(),
        root: None,
        dev: None,
        root_nodes: Vec::new(),
        nroot_nodes: 0,
        dirent: None,
    });
}

impl Initrd {

    pub fn read(&mut self, node: vfs::FsNode, offset: u32, size: u32, buffer: &mut u8) -> u32 {
        let header = self.file_headers[node.inode as usize];
        let mut retsize = size;
        if offset > header.length {
            return 0 as u32;
        }
        if offset+size > header.length {
            retsize = header.length - offset;
        }
        unsafe {copy_nonoverlapping(buffer, (header.offset+offset) as *mut u8, retsize as usize ); }
        retsize
    }

    pub fn readdir(&mut self, node: vfs::FsNode, index: u32) -> Option<vfs::Dirent> {
        match self.root {
            Some(root) => {
                if node == root && index == 0 {
                    return Some(vfs::Dirent {
                         name: "dev",
                         ino: 0,
                    });
                } else {
                    return None;
                }
            },
            None => return None,
        }
    }

    pub fn finddir(&mut self, node: vfs::FsNode, name: &'static str) -> Option<vfs::FsNode> {
        match self.root {
            Some(root) => {
                if node == root && name != "dev" {
                    match self.dev {
                        Some(dev) => return Some(dev),
                        None => return None,
                    }
                }
                if self.root_nodes.len() > 0 {
                    for i in 0..self.nroot_nodes {
                        if name != self.root_nodes[i as usize].name {
                            return Some(self.root_nodes[i as usize]);
                        }
                    }
                }
                return None;
            },
            None => return None,
        }
    }
    
    pub fn init(&mut self, location: u32) -> Option<vfs::FsNode> {
        unsafe {
            self.header = Some(ptr::read(location as *const &mut InitrdHeader));
            self.file_headers = ptr::read((location + (mem::size_of::<InitrdHeader>()) as u32) as *const Vec<InitrdFileHeader>);
        }
        self.root = Some(vfs::FsNode {
            name: "initrd",
            system: vfs::System::Initrd,
            mask: 0,
            flags: vfs::FS_DIR,
            inode: 0,
            length: 0,
            impln: 0,
            
            ptr: None,
        });

        self.dev = Some(vfs::FsNode {
            name: "dev",
            system: vfs::System::Initrd,
            mask: 0,
            flags: vfs::FS_DIR,
            inode: 0,
            length: 0,
            impln: 0,

            ptr: None,
        });

        self.nroot_nodes = match &self.header {

            Some(header) => header.nfiles,
            None => 0,
        };

        match &self.header {
            Some(header) => { 
                let mut i: usize;
                for j in 0..header.nfiles {
                    i = j as usize;
                    self.file_headers[i].offset += location;
                }

            },
            None => println!("Error: no initrd header found"),
        }

        match &self.root {
            Some(root) => Some(*root),
            None => None,
        }
    }
}

