use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use spin::Mutex;
use lazy_static::lazy_static;
use alloc::string::String;
use crate::vga_buffer;
use crate::println;
use crate::print;

pub struct Command {
    com_name: String,
    func: fn(args: Vec<String>),
}

lazy_static! {
    pub static ref COMMANDS: Mutex<BTreeMap<String, fn(args: Vec<String>)>> = 
        Mutex::new(BTreeMap::new()); 
}

pub fn init() {
    let clear = Command {
        com_name: String::from("clear"),
        func: clear_fn,
    };
    COMMANDS.lock().insert(String::from(clear.com_name), clear.func);

    let echo = Command {
        com_name: String::from("echo"),
        func: echo_fn,
    };
    COMMANDS.lock().insert(String::from(echo.com_name), echo.func);
}

pub fn get_command(name: String, args: Vec<String>) {
    match COMMANDS.lock().get(&name) {
        Some(com) => (com)(args),
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
    print!("{}", text);
}
