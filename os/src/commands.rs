use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloc::slice::SliceConcatExt;
use spin::Mutex;
use lazy_static::lazy_static;
use alloc::string::String;
use crate::vga_buffer;
use crate::println;
use crate::print;

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
}

pub fn init_command(n: String, c: Command) {
    COMMANDS.lock().insert(String::from(n), c);
}

pub fn get_command(name: String, args: Vec<String>) {
    match COMMANDS.lock().get(&name) {
        Some(com) => (com.func)(args),
        None => println!("Command not found: {}", name),
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
