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
    println!("wOS v0.1.0    {}\n", cmos::RTC.lock().get_datetime());

    wfs::demo();

    println!("\nkernel debug console - enter 'help' for a list of commands\n");
    console::prompt();
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
