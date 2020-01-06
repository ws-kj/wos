extern crate alloc;

use alloc::string::{ToString, String};
use crate::stdin;
use lazy_static::lazy_static;
use spin::Mutex;
use crate::{vga_buffer, print, println};
use alloc::vec::Vec;
use crate::commands;

pub struct Console {
    prompt: &'static str,
}

lazy_static! {
    pub static ref CONSOLE: Mutex<Console> = Mutex::new(Console {
        prompt: ">>> ",
    });
}

impl Console {

    pub fn prompt(&mut self) {
        print!("\n{}", self.prompt);
        unsafe { stdin::BUF.force_unlock(); }
        stdin::BUF.lock().set_func(proc_wrapper);
        stdin::BUF.lock().read_line();
    }

    pub fn process_command(&mut self, com: String) {
        if com == String::from("\n") {
            self.prompt();
        } else {
            let args: Vec<String> = com.split_whitespace().map(|s| s.to_string()).collect();
            commands::get_command(args.first().unwrap().to_string(), args); 
            self.prompt();
        }
    }
}

pub fn proc_wrapper(c: String) {
    CONSOLE.lock().process_command(c);
}

