extern crate alloc;

use lazy_static::lazy_static;
use spin::Mutex;
use alloc::string::{String, ToString};
use crate::print;
use alloc::{vec::Vec};
use crate::vga_buffer;

pub enum InputType {
    Line,
    Char,
    Block,
}
pub struct InputBuffer {
    open: bool,
    buffer: String,
    in_type: InputType,
    func: fn(s: String),
}

impl InputBuffer {

    pub fn set_func(&mut self, f: fn(s: String)) {
        self.func = f;
    }

    pub fn read_line(&mut self) {
        self.open = true;
        self.buffer = String::from("");
        self.in_type = InputType::Line;
    }

    pub fn write_char(&mut self, c: char) {
        if self.open {
            print!("{}", c.to_string());
            self.buffer.push(c);
            
            match self.in_type {
                InputType::Char => {
                    (self.func)(c.to_string());
                    self.open = false;
                },
                InputType::Line => {
                    if c == '\n' {
                        self.open = false;
                        (self.func)(self.get_buffer());
                    }
                },
                InputType::Block =>  {
                    let cv:Vec<char> = self.buffer.chars().collect();
                    if c == 'C' && cv[cv.len() - 2] == '^' {
                        self.open = false;
                        (self.func)(self.get_buffer());
                    }
                }
            }
        }
    }

    pub fn check_writable(&mut self, character: char) -> bool {
        if self.open {
            match character {
                '\x08' => {
                    if self.buffer.len() > 0 {
                        vga_buffer::WRITER.lock().backspace();
                        let mut t = self.get_buffer();
                        t = (&t[0..self.get_buffer().len()-1]).to_string();
                        self.buffer = t;
                    }
                    false
                },
                character => true,
            }
        } else {
            false
        }
    }

    pub fn get_buffer(&mut self) -> String {
        let b = &self.buffer;
        String::from(b)
    }
}

lazy_static! {
    pub static ref BUF: Mutex<InputBuffer> = Mutex::new(InputBuffer {
        open: false,
        buffer: String::from(""),
        in_type: InputType::Line,
        func: empty_func,
    });
}

pub fn empty_func(s: String) {
    print!("");
}
