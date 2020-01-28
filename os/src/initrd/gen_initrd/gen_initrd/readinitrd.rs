use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::string::String;
use std::io::Read;
use std::str;

fn main() -> io::Result<()> {
    let mut initrd = File::open("initrd.img")?;
    let mut buffer = String::new();
    initrd.read_to_string(&mut buffer)?;
    let bytes = buffer.as_bytes();

    let nheaders = bytes[0];


    Ok(()) 
}

