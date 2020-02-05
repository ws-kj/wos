use lazy_static::lazy_static;
use spin::Mutex;
use crate::initrd;
use alloc::string::{ToString, String};
use alloc::vec::Vec;
use core::ptr;

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

lazy_static! {
    pub static ref FS_ROOT: Mutex<FsRoot> = Mutex::new(FsRoot { node: FsNode {
        name: String::from("/"),
        system: System::Initrd,
        flags: 2,
        inode: 0,
        length: 0,
        children: Vec::new(),
        parent: ptr::null_mut(),
    }});
}


#[derive(Debug, Eq, PartialEq, Clone)]
#[repr(C)]
pub struct FsNode {
    pub name:   String,
    pub system: System,
    pub flags:  u32,
    pub inode:  u32,
    pub length: u32,
    pub children: Vec<*mut FsNode>,
    pub parent: *mut FsNode,
}
unsafe impl Send for FsNode {}

pub fn get_node_from_path(p: String) -> Option<*mut FsNode> {
    let mut path = p;
    if path == String::from("/") { return Some(&mut FS_ROOT.lock().node); }
    if path.chars().nth(&path.chars().count() - 1).unwrap() == '/' {
        let t = String::from(&path[0..&path.chars().count()-1]);
        path = t.clone();
    }
    let mut args: Vec<String> = path.split("/").map(|s| s.to_string()).collect();
    if args.len() == 0 { return None; }

    for i in 0..args.len() - 1 {
        if args[i] == "" {
            args.remove(i);
        }
    }


    let mut i = 0;
    let mut node: *mut FsNode = &mut FS_ROOT.lock().node;
    loop { unsafe {
        match get_child(&(*node), args[i].clone()) {
            Some(n) => {
                node = n;
                i += 1;
                if i == args.len() {
                    return Some(node);
                }
            },
            None => break,
        }
    }}
    None
}
pub fn get_node(node: &mut FsNode, p: String) -> Option<*mut FsNode> {
    let mut path = p;

    if path == String::from("/") { return Some(&mut FS_ROOT.lock().node); }
    if path.chars().nth(&path.chars().count() - 1).unwrap() == '/' {
        let t = String::from(&path[0..&path.chars().count()-1]);
        path = t.clone();
    }
    let mut args: Vec<String> = path.split("/").map(|s| s.to_string()).collect();
    if args.len() == 0 { return None; }

    for i in 0..args.len() - 1 {
        if args[i] == "" {
            args.remove(i);
        }
    }
    let mut i = 0;
    let mut n: *mut FsNode = node;
    if path.chars().nth(0).unwrap() == '/' {
        match get_node_from_path(path) {
            Some(no) => Some(no),
            None => None,
        };
    }

    loop { unsafe {
        if args[i] == ".." {
            FS_ROOT.force_unlock();
            if n == &mut FS_ROOT.lock().node {
                args.remove(i);
                continue;
            }
            let g = (*n).parent;
            n = g;
            i += 1;
        }
        
        if i >= args.len() {
            return Some(n);
        }

        match get_child(&(*n), args[i].clone()) {
            Some(no) => {
                i += 1;
                if i == args.len() {
                    return Some(no.clone());
                } else {
                    n = no.clone();
                }
            },
            None => break,
        }

    }}

    None
}

pub fn read(node: &FsNode) -> &'static [u8] {
    match node.system {
        System::Initrd => initrd::read(node),
    }
}

pub fn get_child(node: &FsNode, name: String) -> Option<*mut FsNode> {
    if node.children.len() > 0 {
       for i in 0..node.children.len() {
           unsafe {
               if (*node.children[i]).name == name {
                   return Some(node.children[i]);
                }
            }
        }
        return None;
    } else {
        return None;
    }
}

pub fn get_nth_child(node: &FsNode, index: usize) -> Option<*mut FsNode> {
    if node.children.len() != 0 && node.children.len() >= index + 1 {
        Some(node.children[index]) 
    } else {
        None
    }
}

pub fn add_child(parent: &mut FsNode, node: &mut FsNode) {
    node.parent = parent as *mut FsNode;
    parent.children.push(node as *mut FsNode);
}

pub fn reparent(node: *mut FsNode, np: *mut FsNode) {
    unsafe {
        let mut i = 0;
        for n in (*(*node).parent).children.iter() {
            if &(*(*n)).name == &(*node).name {
                (*(*node).parent).children.remove(i);

                (*node).parent = np;
                (*np).children.push(node);
            }
            i += 1;
        }
    }
}


/*
TODO: Implement write, open, and close for InitRD
pub fn write_fs(node: FsNode, offset: u32, size: u32, buffer: u8) -> u32 {
    match node.system {
        System::Initrd => initrd::INITRD.lock().write(node, offset, size, buffer),
    }
}



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
