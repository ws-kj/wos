use lazy_static::lazy_static;
use spin::Mutex;
use alloc::string::{ToString, String};
use alloc::vec::Vec;
use core::ptr;
use crate::wfs;

pub const FS_DIR: usize = 0x02;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum System {
    Initrd,
    WFS,
}

#[derive(Copy, Clone)]
pub struct Device {
    pub name: [char; 128],
    pub system: System,
    pub index: usize,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct FsNode {
    pub name:   [char; 128],
    pub device: usize,
    pub parent_id: u64,
    pub id: u64,
    pub attributes: u8,
    pub t_creation: u64,
    pub t_edit: u64,
    pub owner: u8,
    pub size: u64,
}
unsafe impl Send for FsNode {}

lazy_static! {
    pub static ref DEVICES: Mutex<Vec<Device>> = Mutex::new(Vec::new());
}

pub fn install_device(name: String, system: System) -> Result<usize, &'static str> {
    for d in DEVICES.lock().iter() {
        if name == string_from_filename(d.name) {
            return Err("device with name already exists");
        }
    }

    let s = DEVICES.lock().len();
    DEVICES.lock().push(Device {
        name: nfs(name),
        system: system,
        index: s,
    });

    Ok(s)
}

pub fn find_node(parent_id: u64, name: &'static str, dev_id: usize) -> Result<FsNode, &'static str> {
    match DEVICES.lock().get(dev_id) {
        Some(d) => {
            match d.system {
                System::WFS => return wfs::find_node(parent_id, name, dev_id),
                _ => return Err("operation not supported by filesystem"),
            }
        },
        None => return Err("file not found"),
    }
}

pub fn nfs(name: String) -> [char; 128] {
    filename_from_slice(name.as_bytes())
}

pub fn filename_from_slice(slice: &[u8]) -> [char; 128] {
    let mut res: [char; 128] = [' '; 128];
    let mut i = 0;
    for b in slice {
        if i >= 128 || *b as char == ' ' { break; }
        res[i] = *b as char;
        i += 1;
    }
    return res;
}

pub fn string_from_filename(filename: [char; 128]) -> String {
    let mut res = String::from("");
    for c in filename.iter() {
        if *c == ' ' { break; }
        res.push(*c);
    }
    return res;
}

