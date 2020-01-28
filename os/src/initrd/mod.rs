use crate::vfs;
use spin::Mutex;
use lazy_static::lazy_static;
use alloc::vec::Vec;
use core::mem;
use alloc::string::String;
use core::ptr;

pub mod initrd_img;

#[repr(C)]
pub struct Initrd {
    pub nfiles: u8,
    pub file_headers: Vec<FileHeader>,
    pub root: vfs::FsNode,
    pub root_nodes: Vec<vfs::FsNode>,
    pub nroot_nodes: u32,
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
            name: String::from("Init"),
            system: vfs::System::Initrd,
            flags: vfs::FS_DIR,
            inode: 0,
            length: 0,
            children: Vec::new(),
            parent: ptr::null_mut(),
        },
        root_nodes: Vec::new(),
        nroot_nodes: 0,
    });
}

pub fn read(node: vfs::FsNode) -> &'static [u8] {
    let header = INITRD.lock().file_headers[node.inode as usize];
    let buf = &initrd_img::IMG[(header.offset as usize)..header.offset as usize + header.size  as usize];
    buf
}

pub fn init() {
    INITRD.lock().nfiles = initrd_img::IMG[0];
 
    INITRD.lock().nroot_nodes = initrd_img::IMG[0] as u32;

    vfs::add_child(&mut vfs::FS_ROOT.lock().node, &mut INITRD.lock().root);

    let mut offset = 1;
    for i in 0..INITRD.lock().nroot_nodes {
        let header_size = mem::size_of::<FileHeader>();
        let buffer = &initrd_img::IMG[offset..offset + header_size];
        
        let ptr: *const FileHeader = unsafe { mem::transmute(buffer.as_ptr()) };
        let header: FileHeader = unsafe { *ptr };

        unsafe { INITRD.force_unlock() };
        INITRD.lock().file_headers.push(header);


        let node = vfs::FsNode {
            name: String::from(&osfn(header.name)),
            system: vfs::System::Initrd,
            flags: vfs::FS_FILE,
            inode: i as u32,
            length: header.size,
            children: Vec::new(),
            parent: ptr::null_mut(),
        };
        INITRD.lock().root_nodes.push(node);
        offset += header_size;
    }

    for i in 0..INITRD.lock().root_nodes.len() {
        unsafe { INITRD.force_unlock() }
        let n = &mut INITRD.lock().root_nodes[i as usize];
        vfs::add_child(&mut vfs::FS_ROOT.lock().node, n);
    }
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

