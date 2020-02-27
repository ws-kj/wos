use alloc::vec::Vec;
use crate::print;
use alloc::string::String;
use crate::vga_buffer;
use alloc::string::ToString;
use lazy_static::lazy_static;
use spin::Mutex;
use crate::stdin;
use crate::commands;

pub struct Console {
    cdir: String,
}
unsafe impl Send for Console {}

lazy_static! {
    pub static ref CONSOLE: Mutex<Console> = Mutex::new(Console {
        cdir: String::from("/"),   
    });
}

pub fn prompt() {
    print!("{}", CONSOLE.lock().cdir);

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

/*
pub fn get_cdir() -> *mut vfs::FsNode {
    CONSOLE.lock().cdir
}

pub fn set_cdir(node: &mut vfs::FsNode) {
    CONSOLE.lock().cdir = node;
}
*/
