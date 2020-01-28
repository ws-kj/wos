use std::fs::{metadata, File};
use std::io::prelude::*;
use std::env;
use std::mem;
use std::vec::Vec;
use std::string::String;
use std::str;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct FileHeader {
    name: [char; 30],
    size: u32,
    offset: u32,
}

fn main() -> std::io::Result<()> {
    gen_img()?;
    Ok(())
}

unsafe fn as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::std::slice::from_raw_parts(
        (p as *const T) as *const u8,
        mem::size_of::<T>(),
    )
}

fn gen_img() -> std::io::Result<()> {
    
    let mut args: Vec<String> = env::args().collect();
    let _t = args.remove(0);
    let nheaders =  args.len();
    let mut off = mem::size_of::<FileHeader>() * nheaders + mem::size_of::<u8>();

    let mut headers = vec![FileHeader {
        name: ['\0'; 30],
        size: 0,
        offset: 0,
        }; nheaders];

    for i in 0..nheaders {
        let chars: Vec<char> = args[i].chars().collect();

        print!("filename: ");
        for j in 0..args[i].chars().count() {
            headers[i].name[j] = chars[j];
            print!("{}", chars[j]);
        }
        headers[i].name[29] = '\0';

        headers[i].offset = off as u32;
        print!("    offset: {}", headers[i].offset);

        headers[i].size = metadata(&args[i])?.len() as u32;
        println!("    size: {}", headers[i].size);

        off += headers[i].size as usize;
    }
    
    let mut initrd = File::create("initrd.img")?;
    initrd.write(&[nheaders as u8])?;

    for i in 0..nheaders {
        let mut bytes: &[u8] = unsafe {as_u8_slice(&headers[i])};
        initrd.write(bytes);
    }

    for i in 0..nheaders {
        let mut file = File::open(&args[i])?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        //println!("{}", contents);
        initrd.write(contents.as_bytes())?;
    }

    Ok(())
}

