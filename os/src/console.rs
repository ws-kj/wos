extern crate alloc;

use crate::vfs;
use alloc::string::{ToString, String};
use crate::stdin;
use lazy_static::lazy_static;
use spin::Mutex;
use crate::print;
use alloc::vec::Vec;
use crate::commands;
use crate::vga_buffer;

pub struct Console {
    WD: vfs::FsNode,
}

lazy_static! {
    pub static ref CONSOLE: Mutex<Console> = Mutex::new(Console {
        WD: vfs::FS_ROOT.lock().node.clone(),   
    });
}


pub fn prompt() {
    print!("{}", &CONSOLE.lock().WD.name);
    vga_buffer::WRITER.lock().set_color(vga_buffer::Color::LightCyan, vga_buffer::Color::Black);
    print!(" >>> ");
    vga_buffer::WRITER.lock().set_color(vga_buffer::Color::White, vga_buffer::Color::Black);

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

pub fn get_wd() -> vfs::FsNode {
    CONSOLE.lock().WD.clone()
}

pub fn set_wd(node: &vfs::FsNode) {
    CONSOLE.lock().WD = node.clone();
}

