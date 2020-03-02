use lazy_static::lazy_static;
use spin::Mutex;
use alloc::string::{ToString, String};
use alloc::vec::Vec;
use core::ptr;
use crate::wfs;
use crate::println;


pub const ATTR_RO: usize = 0x00;
pub const ATTR_SYS: usize = 0x01;
pub const ATTR_DIR: usize = 0x02;
pub const ATTR_HDN: usize = 0x03;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Error {
    FileNotFound,
    IllegalOperation,
    PermissionDenied,
    Closed,
    AlreadyOpened,
    OperationNotSupported,
    ParentNotDirectory,
    DeviceNotFound,
    DuplicateDevice,
    ReadError,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum System {
    Initrd,
    WFS,
}

pub struct Device {
    pub name: [char; 128],
    pub system: System,
    pub index: usize,
    pub opened: Vec<u64>,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct FsNode {
    pub open: bool,
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
    pub fn open(&mut self) -> Result<(), Error> {
        if self.open { return Err(Error::AlreadyOpened); }

        match DEVICES.lock().get_mut(self.device) {
            Some(d) => {
                if d.opened.contains(&self.id) { 
                    self.open = true;
                    return Err(Error::AlreadyOpened); 
                }
                
                d.opened.push(self.id);
                self.open = true;
            },
            None => return Err(Error::DeviceNotFound),
        }
        Ok(())
    }

    pub fn close(&mut self) -> Result<(), Error> {
        if !self.open { return Err(Error::Closed); }

        match DEVICES.lock().get_mut(self.device) {
            Some(d) => {
                if d.opened.contains(&self.id) {
                    d.opened.remove(d.opened.iter().position(|&r| r == self.id).unwrap());
                }

                self.open = false;
            },
            None => return Err(Error::DeviceNotFound),
        }

        Ok(())
    }

    pub fn read(&mut self) -> Result<Vec<u8>, Error> {
        if !self.open { return Err(Error::Closed); }

        match DEVICES.lock().get_mut(self.device) {
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
        if !self.open { return Err(Error::Closed); }

        match DEVICES.lock().get_mut(self.device) {
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
        if !self.open { return Err(Error::Closed); }

        match DEVICES.lock().get_mut(self.device) {
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
        if !self.open { return Err(Error::Closed) };

        match DEVICES.lock().get_mut(self.device) {
            Some(d) => {
                match d.system {
                    System::WFS => return wfs::delete_node(self.parent_id, sfn(self.name)),
                    _ => return Err(Error::OperationNotSupported),
                }
            },
            None => return Err(Error::DeviceNotFound),
        }
    }

    pub fn get_children(&mut self) -> Result<Vec<FsNode>, Error> {
        match DEVICES.lock().get_mut(self.device) {
            Some(d) => {
                match d.system {
                    System::WFS => return wfs::get_children(self.parent_id, sfn(self.name), self.device),
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
        opened: Vec::new(),
    });

    Ok(s)
}

pub fn find_node_by_id(id: u64, dev_id: usize) -> Result<FsNode, Error> {
    match DEVICES.lock().get_mut(dev_id) {
        Some(d) => {
            match d.system {
                System::WFS => return wfs::find_node_by_id(id, dev_id),
                _ => return Err(Error::OperationNotSupported),
            }
        },
        None => return Err(Error::DeviceNotFound),
    }
}

pub fn find_node(parent_id: u64, name: String, dev_id: usize) -> Result<FsNode, Error> {
    match DEVICES.lock().get_mut(dev_id) {
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
    match DEVICES.lock().get_mut(dev_id) {
        Some(d) => {
            match d.system {
                System::WFS => return wfs::create_node(parent_id, filename, attributes, owner, dev_id),
                _ => return Err(Error::OperationNotSupported),
            }
        },
        None => return Err(Error::DeviceNotFound),
    }
} 

pub fn get_root(dev_id: usize) -> Result<FsNode, Error> {
    match DEVICES.lock().get(dev_id) {
        Some(d) => {
            match d.system {
                System::WFS => return wfs::get_root(dev_id),
                _ => return Err(Error::OperationNotSupported),
            }
        },
        None => return Err(Error::DeviceNotFound),
    }
}

pub fn get_parent(id: u64, dev_id: usize) -> Result<FsNode, Error> {
    match DEVICES.lock().get(dev_id) {
        Some(d) => {
            match d.system {
                System::WFS => return wfs::get_parent(id, dev_id),
                _ => return Err(Error::OperationNotSupported),
            }
        },
        None => return Err(Error::DeviceNotFound),
    }
}

pub fn node_from_local_path(p: &FsNode, path: String) -> Result<FsNode, Error> {
    let mut names: Vec<&str> = path.split("/").collect();
    let dev_id = (*p).device;

    let goal = names[names.len() - 1];
    let mut parent = *p;

    let mut i = 0;
    loop {
        if i >= names.len() {
            break;
        }

        if names[i] == ".." {
            parent = get_parent(parent.id, dev_id)?;
            if i == names.len() - 2 {
                return Ok(parent);
            } 

            i += 1;
            continue;
        }

        let node = find_node(parent.id, names[i].to_string(), dev_id)?;

        if &sfn(node.name) == goal {
            return Ok(node);
        }
        
        parent = node;
        i += 1;
    }
    return Err(Error::FileNotFound);
}

pub fn node_from_path(path: String) -> Result<FsNode, Error> {
    let mut names: Vec<&str> = path.split("/").collect();
    let mut dev_id = 0;
   
    for d in DEVICES.lock().iter() {
        if sfn(d.name) == names[0] {
            dev_id = d.index;
        }
    }
    names.remove(0);

    let goal = names[names.len() - 1];
    let mut parent = get_root(dev_id)?;
    
    let mut i = 0;
    loop {
        if i >= names.len() {
            break;
        }

        let node = find_node(parent.id, names[i].to_string(), dev_id)?;
        
        if &sfn(node.name) == goal {
            return Ok(node);
        }
        
        parent = node;
        i += 1;
    }

    return Err(Error::FileNotFound);
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

