#![no_std]
#![no_main]

use core::panic::PanicInfo;

// See src/lib.rs
use nostd_env::{
    bios,
    man_heap::{self, ALLOC_UNDER16, ALLOC_UNDER20, GLOBAL_ALLOC},
    man_video,
    println,
    test_alloc,
    test_diskio,
    x86::halt_forever,
};


// Panic handler (cf. https://doc.rust-lang.org/nomicon/panic-handler.html )
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    halt_forever();
}


// Entry point of the Rust world.
#[no_mangle]
pub extern "C" fn __bare_start() -> ! {
    // Print the current stack usage.
    println!("Stack max = {}", bios::StackUsage::new());

    // Initialize the global allocator (size = 1MB)
    man_heap::init_global_alloc(1024 * 1024, &ALLOC_UNDER20);

    // Find the best mode using VESA BIOS Extentions.
    man_video::find_graphics_mode(1280, 1024, 24, &ALLOC_UNDER20);

    // Try Checking Stack Usages of BIOS Text Output and Disk I/O.
    test_diskio::try_read_sectors1(&ALLOC_UNDER16);
    test_diskio::try_read_sectors2(&ALLOC_UNDER16);

    // Test: allocator and heap manager
    test_alloc::try_sieve(30, 100, 10000, &GLOBAL_ALLOC);

    // Print the current stack usage.
    println!("Stack max = {}", bios::StackUsage::new());

    // Halt
    halt_forever();
}
