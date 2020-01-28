use std::fs::{metadata, File};
use std::io::prelude::*;
use std::env;
use std::mem;
use std::vec::Vec;
use std::string::String;

#[repr(C)]
#[derive(Copy, Clone)]
struct FileHeader {
    name: [char; 30],
    size: u32,
    offset: u32,
}

fn main() -> std::io::Result<()> {
    gen_img()?;
    gen_rs()?;
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
        initrd.write(contents.as_bytes())?;
    }

    Ok(())
}

fn gen_rs() -> std::io::Result<()> {
    //hacky initrd solution: generate .rs file with initrd.img as &[u8]
    let mut rs = File::create("../initrd_img.rs")?;
    rs.write(b"use core::str;\n")?;
    rs.write(b"pub const IMG: &[u8] = \"")?;
    let mut img_contents = String::new();
    let mut ird2 = File::open("initrd.img")?;
    ird2.read_to_string(&mut img_contents);
    rs.write(img_contents.as_bytes())?;
    rs.write(b"\".as_bytes();")?;

    Ok(())
}
