use alloc::vec::Vec;
use crate::print;
use alloc::string::String;
use crate::vga_buffer;
use alloc::string::ToString;
use lazy_static::lazy_static;
use spin::Mutex;
use crate::stdin;
use crate::commands;
use crate::vfs;

pub struct Console {
    cdir: Option<vfs::FsNode>,
    cdev: usize,
}
unsafe impl Send for Console {}

lazy_static! {
    pub static ref CONSOLE: Mutex<Console> = Mutex::new(Console {
        cdir: None,
        cdev: 0,
    });
}

pub fn init() {
    CONSOLE.lock().cdir = Some(vfs::find_node(0, String::from("ATA0"), 0).unwrap());

    prompt();
}

pub fn prompt() {
    print!("{}/", vfs::sfn(CONSOLE.lock().cdir.unwrap().name));

    vga_buffer::set_color(vga_buffer::Color::LightCyan, vga_buffer::Color::Black);
    print!(" >>> ");
    vga_buffer::set_color(vga_buffer::Color::White, vga_buffer::Color::Black);

    unsafe { stdin::BUF.force_unlock(); }
    stdin::BUF.lock().set_func(process_command);
    stdin::BUF.lock().read_line();
}

pub fn process_command(com: String) {
    if com == String::from("\n") {
        prompt();
    } else {
        let args: Vec<String> = com.split_whitespace().map(|s| s.to_string()).collect();
        commands::get_command(args.first().unwrap().to_string(), args); 
        prompt();
    }
}


pub fn get_cdir() -> vfs::FsNode {
    CONSOLE.lock().cdir.unwrap()
}

pub fn set_cdir(node: vfs::FsNode) {
    CONSOLE.lock().cdir = Some(node);
}

