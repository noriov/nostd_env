#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(allocator_api)]

mod bios;
mod man_heap;
mod mu;
mod query_vbe;
mod test_alloc;
mod test_diskio;
mod text_writer;
mod x86;

extern crate alloc;
use core::panic::PanicInfo;

use crate::man_heap::{ALLOC_UNDER16, ALLOC_UNDER20, GLOBAL_ALLOC};
use crate::x86::halt_forever;


#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    halt_forever();
}


#[no_mangle]
pub extern "C" fn __bare_start() -> ! {
    // Print the current stack usage.
    println!("Stack max = {}", bios::StackUsage::new());

    // Initialize the global allocator (size = 1MB)
    man_heap::init_global_alloc(1024 * 1024, &ALLOC_UNDER20);

    // Query VESA BIOS Extentions.
    query_vbe::query_vbe(1280, 1024, 24, &ALLOC_UNDER20);

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
