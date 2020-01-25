use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloc::slice::SliceConcatExt;
use spin::Mutex;
use lazy_static::lazy_static;
use alloc::string::String;
use crate::vga_buffer;
use crate::println;
use crate::print;
use crate::cmos;
use crate::vfs;
use core::str;

pub struct Command {
    com_name: String,
    desc: String,
    func: fn(args: Vec<String>),
}

lazy_static! {
    pub static ref COMMANDS: Mutex<BTreeMap<String, Command>> = 
        Mutex::new(BTreeMap::new()); 
}

pub fn init() {
    let clear = Command {
        com_name: String::from("clear"),
        desc: String::from("clear the screen"),
        func: clear_fn,
    };
    init_command(String::from("clear"), clear);

    let echo = Command {
        com_name: String::from("echo"),
        desc: String::from("write a string to the screen"),
        func: echo_fn,
    };
    init_command(String::from("echo"), echo);

    let help = Command {
        com_name: String::from("help"),
        desc: String::from("list commands and descriptions"),
        func: help_fn,
    };
    init_command(String::from("help"), help);

    let time = Command {
        com_name: String::from("time"),
        desc: String::from("get the current time and date"),
        func: time_fn,
    };
    init_command(String::from("time"), time);

    let ls = Command {
        com_name: String::from("ls"),
        desc: String::from("list files"),
        func: ls_fn,
    };
    init_command(String::from("ls"), ls);

    let read = Command {
        com_name: String::from("read"),
        desc: String::from("get contents of a file"),
        func: read_fn,
    };
    init_command(String::from("read"), read);

    let info = Command {
        com_name: String::from("info"),
        desc: String::from("get info about file(s)"),
        func: info_fn,
    };
    init_command(String::from("info"), info);
}

pub fn init_command(n: String, c: Command) {
    COMMANDS.lock().insert(String::from(n), c);
}

pub fn get_command(name: String, args: Vec<String>) {
    match COMMANDS.lock().get(&name) {
        Some(com) => (com.func)(args),
        None => println!("command not found: {}", name),
    }
}

pub fn clear_fn(args: Vec<String>) {
    vga_buffer::WRITER.lock().clear_screen();
}

pub fn echo_fn(args: Vec<String>) {
    let mut res = args;
    res.remove_item(&String::from("echo"));
    let text = res.join(" ");
    println!("{}", text);
}

pub fn help_fn(args: Vec<String>) {
    unsafe { COMMANDS.force_unlock(); }
    if args.len() > 1 {
        match COMMANDS.lock().get(&args[1]) {
            Some(com) => {
                println!("{} - {}", com.com_name, com.desc);
            },
            None => println!("help: command not found: {}", args[0]),
        }
    } else { 
        for (n, com) in COMMANDS.lock().iter() {
            println!("{} - {}", com.com_name, com.desc);
        }
    }
            
}

pub fn time_fn(args: Vec<String>) {
    println!("{}", cmos::RTC.lock().get_datetime());
}

pub fn ls_fn(args: Vec<String>) {
    for c in vfs::FS_ROOT.lock().node.children.iter() {
        unsafe {
            print!("{}", (*(*c)).name);
            if (*(*c)).flags&0x7 == vfs::FS_DIR {
                println!("/");
            } else {
                println!();
            }
        }
    }
}

pub fn read_fn(args: Vec<String>) {
    if args.len() <= 1 {
        println!("please specify a file");
        return ();
    }

    //let node = vfs::finddir_fs(&initrd::INITRD.lock().dev, args[1].clone());
    
    let node = vfs::get_child(&vfs::FS_ROOT.lock().node, args[1].clone());

    match node {
        Some(n) => {
            if (n.flags&0x7 == vfs::FS_DIR) && n.length == 0 {
                println!("{} is a directory", n.name);
            } else {
                println!("{}", str::from_utf8(&vfs::read_fs(n)).unwrap());
            }
        },
        None => println!("file not found: {}", &args[1]),
    }
}

pub fn info_fn(args: Vec<String>) {

    //println!("{}", vfs::get_nth_child(&initrd::INITRD.lock().root, 0).unwrap().name);
    for i in 1..args.len() {
        let node = vfs::get_child(&vfs::FS_ROOT.lock().node, args[i].clone());
        match node {
            Some(n) => {
                println!("file: {}", n.name);
                println!("    flags: {}", n.flags);
                println!("    length: {}B", n.length);
            },
            None => println!("file not found: {}", &args[i]),
        }
        println!(); 
    }
}

