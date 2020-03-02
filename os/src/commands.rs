use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloc::slice::SliceConcatExt;
use spin::Mutex;
use lazy_static::lazy_static;
use alloc::string::String;
use crate::vga_buffer;
use crate::println;
use crate::drivers::cmos;
use crate::vfs;
use crate::console;
use bit_field::BitField;
use crate::print;

pub struct Command {
    name: String,
    desc: String,
    func: fn(args: Vec<String>),
}

lazy_static! {
    pub static ref COMMANDS: Mutex<BTreeMap<String, Command>> = 
        Mutex::new(BTreeMap::new()); 
}

pub fn init() {
    let clear = Command {
        name: String::from("clear"),
        desc: String::from("clear the screen"),
        func: clear_fn,
    };
    init_command(String::from("clear"), clear);

    let echo = Command {
        name: String::from("echo"),
        desc: String::from("write a string to the screen"),
        func: echo_fn,
    };
    init_command(String::from("echo"), echo);

    let help = Command {
        name: String::from("help"),
        desc: String::from("list commands and descriptions"),
        func: help_fn,
    };
    init_command(String::from("help"), help);

    let time = Command {
        name: String::from("time"),
        desc: String::from("get the current time and date"),
        func: time_fn,
    };
    init_command(String::from("time"), time);

    let ls = Command {
        name: String::from("ls"),
        desc: String::from("list files"),
        func: ls_fn,
    };
    init_command(String::from("ls"), ls);

    let read = Command {
        name: String::from("read"),
        desc: String::from("get contents of a file"),
        func: read_fn,
    };
    init_command(String::from("read"), read);

    let info = Command {
        name: String::from("info"),
        desc: String::from("get info about file(s)"),
        func: info_fn,
    };
    init_command(String::from("info"), info);

    let pcd = Command {
        name: String::from("pcd"),
        desc: String::from("print current directory"),
        func: pcd_fn,
    };
    init_command(String::from("pcd"), pcd);

    let mkf = Command {
        name: String::from("mkf"),
        desc: String::from("create file"),
        func: mkf_fn,
    };
    init_command(String::from("mkf"), mkf);

    let write = Command {
        name: String::from("write"),
        desc: String::from("write to file"),
        func: write_fn,
    };
    init_command(String::from("write"), write);

    let del = Command {
        name: String::from("del"),
        desc: String::from("delete file"),
        func: del_fn,
    };
    init_command(String::from("del"), del);
/*
    let cd = Command {
        name: String::from("cd"),
        desc: String::from("change current directory"),
        func: cd_fn,
    };
    init_command(String::from("cd"), cd);

    let mv = Command {
        name: String::from("mv"),
        desc: String::from("move file"),
        func: mv_fn,
    };
    init_command(String::from("mv"), mv);*/
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
                println!("{} - {}", com.name, com.desc);
            },
            None => println!("help: command not found: {}", args[0]),
        }
    } else { 
        for (n, com) in COMMANDS.lock().iter() {
            println!("{} - {}", com.name, com.desc);
        }
    }
            
}

pub fn time_fn(args: Vec<String>) {
    println!("{}", cmos::RTC.lock().get_datetime());
}

pub fn ls_fn(args: Vec<String>) {
    let mut node = console::get_cdir();

    if args.len() > 1 {
        match vfs::node_from_local_path(&console::get_cdir(), args[1].clone()) {
            Ok(n) => node = n,
            Err(e) => {
                println!("file not found: {}", &args[1]);
                return;
            },
        }
    }
    
    let mut children: Vec<vfs::FsNode> = Vec::new();

    match node.get_children() {
        Ok(v) => children = v,
        Err(e) => println!("could not get children: {}", vfs::sfn(node.name)),
    }

    for c in children.iter() {
        if c.attributes.get_bit(vfs::ATTR_DIR) {
            println!("{}/", vfs::sfn(c.name));
        } else {
            println!("{}", vfs::sfn(c.name));
        }
    }
}

pub fn read_fn(args: Vec<String>) {
    if args.len() <= 1 {
        println!("please specify a file");
        return;
    }

    match vfs::find_node(console::get_cdir().id, args[1].clone(), 0) {
        Ok(mut n) => {
            match n.open() {
                Ok(()) => {},
                Err(e) => {
                    println!("could not open file: {}", &args[1]);
                    return;
                },
            }
            match n.read() {
                Ok(buf) => {
                    for b in buf.iter() {
                        print!("{}", *b as char);
                    }
                },
                Err(e) => println!("could not read file: {}", &args[1]),
            }
            println!();
            match n.close() {
                Ok(()) => {},
                Err(e) => println!("could not close file: {}", &args[1]),
            }
        },
        Err(e) => println!("file not found: {}", &args[1]),
    }

}

pub fn info_fn(args: Vec<String>) {
    if args.len() <= 1 {
        println!("please specify a file");
        return;
    }

    match vfs::find_node(console::get_cdir().id, args[1].clone(), 0) {
        Ok(n) => {
            println!("{}", vfs::sfn(n.name));
            println!("owner: {}", n.owner);
            println!("size: {}B", n.size);
            println!("created: {}", n.t_creation);
            println!("edited: {}", n.t_edit);
        },
        Err(e) => println!("file not found: {}", &args[1]),
    }
}

pub fn pcd_fn(args: Vec<String>) {
    println!("{}/", vfs::sfn(console::get_cdir().name));
}

pub fn mkf_fn(args: Vec<String>) {
    match vfs::create_node(console::get_cdir().id, args[1].clone(), 0, 0, 0) {
        Ok(n) => return,
        Err(e) => println!("could not create file"),
    }
}

pub fn write_fn(args: Vec<String>) {
    if args.len() <= 1 {
        println!("please specify a file");
        return;
    }

    let path = args[1].clone();
    let mut a = args;
    a.remove(0);
    a.remove(0);
    let text = a.join(" ").into_bytes();

    match vfs::node_from_local_path(&console::get_cdir(), path) {
        Ok(mut n) => {
            match n.open() {
                Ok(()) => {},
                Err(e) => {
                    println!("could not open file: {}", &a[1]);
                    return;
                },
            }
            match n.append(text) {
                Ok(()) => {},
                Err(e) => println!("could not write to file: {}", &a[1]),
            }
            match n.close() {
                Ok(()) => {},
                Err(e) => println!("could not close file: {}", &a[1]),
            }
        },
        Err(e) => println!("file not found: {}", &a[1]),
    }
}

pub fn del_fn(args: Vec<String>) {
    if args.len() <= 1 {
        println!("Please specify a file!");
        return;
    }
    
    match vfs::node_from_local_path(&console::get_cdir(), args[1].clone()) {
        Ok(mut n) => {
            match n.open() {
                Ok(()) => {},
                Err(e) => {
                    println!("could not open file: {}", &args[1]);
                    return;
                },
            }
            match n.delete() {
                Ok(()) => {},
                Err(e) => println!("could not delete file: {}", &args[1]),
            }
            match n.close() {
                Ok(()) => {},
                Err(e) => println!("could not close file: {}", &args[1]),
            }
        },
        Err(e) => println!("file not found: {}", &args[1]),
    }

}
/*

pub fn cd_fn(args: Vec<String>) {
    if args.len() == 1 { 
        console::set_wd(&mut vfs::FS_ROOT.lock().node);
        return ();
    }

    unsafe { 
        match vfs::get_node(&mut(*console::get_wd()), args[1].clone()) {
            Some(n) => console::set_wd(&mut(*n)),
            None => println!("file not found: {}", &args[1]),
        }
    }
}

pub fn mv_fn(args: Vec<String>) { unsafe {
    if args.len() != 3 {
        println!("please specify a file and a destination");
        return ();
    }

    let mut f: *mut vfs::FsNode = ptr::null_mut();
    let mut d: *mut vfs::FsNode = ptr::null_mut();

    match vfs::get_node(&mut(*console::get_wd()), args[1].clone()) {
        Some(n) => f = n,
        None => {
            println!("file not found: {}", &args[1]);
            return ();
        },
    }
    match vfs::get_node(&mut(*console::get_wd()), args[2].clone()) {
        Some(n) => d = n,
        None => {
            println!("file not found: {}", &args[2]);
            return ();
        },
    }

    vfs::reparent(f, d);
}}
*/


