
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![feature(asm)]
#![test_runner(os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::panic::PanicInfo;
use os::println;
use bootloader::{entry_point, BootInfo};
use os::vga_buffer;
use os::commands;
use os::drivers::cmos;
use os::drivers::ata;
use os::wfs;
use os::vfs;
use alloc::vec::Vec;
use alloc::string::String;
use os::print;
use os::console;
use alloc::string::ToString;
use bit_field::BitField;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use os::{allocator, memory};
    use x86_64::{VirtAddr};

    os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        memory::BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed"); 

    #[cfg(test)]
    test_main();
   
    vga_buffer::WRITER.lock().clear_screen();

    commands::init();
    ata::init();
    wfs::init();

    println!();
    console::init();

    let mut hello = vfs::create_node(1, String::from("hello.txt"), 0, 0, 0).unwrap();
    hello.open();
    hello.write(b"Welcome to the wOS filesystem, wFS!\n".to_vec());
    hello.close();

    vfs::create_node(1, String::from("Home"), *0u8.set_bit(vfs::ATTR_DIR, true), 0, 0).unwrap();

    os::hlt_loop();
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os::test_panic_handler(info)
}
