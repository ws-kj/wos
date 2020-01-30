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
    wd: *mut vfs::FsNode,
}
unsafe impl Send for Console {}

lazy_static! {
    pub static ref CONSOLE: Mutex<Console> = Mutex::new(Console {
        wd: &mut vfs::FS_ROOT.lock().node,   
    });
}

pub fn prompt() {
    unsafe {
        print!("{}", &(*CONSOLE.lock().wd).name);

        if (*CONSOLE.lock().wd).name != String::from("/") {
            print!("/");
        }

        vga_buffer::WRITER.lock().set_color(vga_buffer::Color::LightCyan, vga_buffer::Color::Black);
        print!(" >>> ");
        vga_buffer::WRITER.lock().set_color(vga_buffer::Color::White, vga_buffer::Color::Black);

        stdin::BUF.force_unlock(); 
    }
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

pub fn get_wd() -> *mut vfs::FsNode {
    CONSOLE.lock().wd
}

pub fn set_wd(node: &mut vfs::FsNode) {
    CONSOLE.lock().wd = node;
}

