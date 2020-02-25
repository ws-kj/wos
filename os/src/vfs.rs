use lazy_static::lazy_static;
use spin::Mutex;
use alloc::string::{ToString, String};
use alloc::vec::Vec;
use core::ptr;
use crate::wfs;

pub const ATTR_RO: usize = 0x00;
pub const ATTR_SYS: usize = 0x01;
pub const ATTR_DIR: usize = 0x02;
pub const ATTR_HDN: usize = 0x03;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Error {
    FileNotFound,
    IllegalOperation,
    PermissionDenied,
    OperationNotSupported,
    DeviceNotFound,
    DuplicateDevice,
    ReadError,
}

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

impl FsNode {
    pub fn read(&mut self) -> Result<Vec<u8>, Error> {
        match DEVICES.lock().get(self.device) {
            Some(d) => {
                match d.system {
                    System::WFS => return wfs::read_node(self.parent_id, sfn(self.name)),
                    _ => return Err(Error::OperationNotSupported),
                }
            }
            None => return Err(Error::DeviceNotFound),
        }
    }

    pub fn write(&mut self, buf: Vec<u8>) -> Result<(), Error> {
        match DEVICES.lock().get(self.device) {
            Some(d) => {
                match d.system {
                    System::WFS => {
                        let len = buf.len() as u64;
                        match wfs::write_node(self.parent_id, sfn(self.name), buf) {
                            Ok(_) => {
                                self.size = len;
                                Ok(())
                            },
                            Err(s) => Err(s),
                        }
                    },
                    _ => return Err(Error::OperationNotSupported),
                }
            }
            None => return Err(Error::DeviceNotFound),
        }
    } 

    pub fn append(&mut self, buf: Vec<u8>) -> Result<(), Error> {
        match DEVICES.lock().get(self.device) {
            Some(d) => {
                match d.system {
                    System::WFS => {
                        let len = self.size + (buf.len() as u64);
                        match wfs::append_node(self.parent_id, sfn(self.name), buf) {
                            Ok(_) => {
                                self.size = len;
                                Ok(())
                            },
                            Err(s) => Err(s),
                        }
                    },
                    _ => return Err(Error::OperationNotSupported),
                }
            },
            None => return Err(Error::DeviceNotFound),
        }
    }

    pub fn delete(&mut self) -> Result<(), Error> {
        match DEVICES.lock().get(self.device) {
            Some(d) => {
                match d.system {
                    System::WFS => {
                        match wfs::delete_node(self.parent_id, sfn(self.name)) {
                            Ok(_) => {
                                Ok(())
                            },
                            Err(s) => Err(s),
                        }
                    },
                    _ => return Err(Error::OperationNotSupported),
                }
            },
            None => return Err(Error::DeviceNotFound),
        }
    }
}

lazy_static! {
    pub static ref DEVICES: Mutex<Vec<Device>> = Mutex::new(Vec::new());
}

pub fn install_device(name: String, system: System) -> Result<usize, Error> {
    for d in DEVICES.lock().iter() {
        if name == sfn(d.name) {
            return Err(Error::DuplicateDevice);
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


pub fn find_node(parent_id: u64, name: String, dev_id: usize) -> Result<FsNode, Error> {
    match DEVICES.lock().get(dev_id) {
        Some(d) => {
            match d.system {
                System::WFS => return wfs::find_node(parent_id, name, dev_id),
                _ => return Err(Error::OperationNotSupported),
            }
        },
        None => return Err(Error::DeviceNotFound),
    }
}

pub fn create_node(parent_id: u64, filename: String, attributes: u8, owner: u8, dev_id: usize) -> Result<FsNode, Error> {
    match DEVICES.lock().get(dev_id) {
        Some(d) => {
            match d.system {
                System::WFS => return wfs::create_node(parent_id, filename, attributes, owner, dev_id),
                _ => return Err(Error::OperationNotSupported),
            }
        }
        None => return Err(Error::DeviceNotFound),
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

pub fn sfn(filename: [char; 128]) -> String {
    let mut res = String::from("");
    for c in filename.iter() {
        if *c == ' ' { break; }
        res.push(*c);
    }
    return res;
}

