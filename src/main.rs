#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(allocator_api)]

mod bios;
mod mu;
mod query_vbe;
mod test_alloc;
mod test_diskio;
mod text_writer;
mod x86;

extern crate alloc;
use alloc::alloc::Layout;
use alloc::vec::Vec;
use core::panic::PanicInfo;

use crate::mu::{MuAlloc16, MuAlloc32};
use crate::x86::halt_forever;


#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    halt_forever();
}

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("Failed to allocate {:?}", layout)
}


#[no_mangle]
pub extern "C" fn __bare_start() -> ! {
    // Initialize the global allocator (size = 1MB)
    init_global_alloc(1024 * 1024);

    // Query VESA BIOS Extentions.
    query_vbe::query_vbe(1280, 1024, 24, &ALLOC_UNDER20);

    // Try Checking Stack Usages of BIOS Text Output and Disk I/O.
    {
	bios::check_stack_usage();
	println!("Try Checking Stack Usages");
	println!("Stack max = {:#x}", bios::check_stack_usage());
	test_diskio::try_read_sectors1(&ALLOC_UNDER16);
	println!("Stack max = {:#x}", bios::check_stack_usage());
	test_diskio::try_read_sectors2(&ALLOC_UNDER16);
	println!("Stack max = {:#x}", bios::check_stack_usage());
    }

    // Test: allocator and heap manager
    test_alloc::try_sieve(30, 100, 10000, &GLOBAL_ALLOC);

    // Halt
    halt_forever();
}


fn init_global_alloc(size: usize) -> Vec<bios::AddrRange> {
    const ADDR_1MB: u64 = 0x10_0000;

    if let Some(addr_ranges) = bios::int15he820h::call(&ALLOC_UNDER20) {
	for entry in &addr_ranges {
	    #[allow(unused_parens)]
	    if (entry.is_usable() &&
		entry.addr >= ADDR_1MB && entry.length as usize >= size) {
		let base = entry.addr as usize;
		unsafe {
		    GLOBAL_ALLOC.lock().set_heap(base, size);
		}
		return addr_ranges.to_vec();
	    }
	}
    }

    panic!("Failed to initialize the global allocator");
}


//
// Three allocators are initialized:
// - ALLOC_UNDER16 (in 16-bit address space) and ALLOC_UNDER20
//   (in 20-bit address space) are heap areas maily for buffers
//   to be exchanged with BIOS.  Their base address and size in bytes
//   are specified in their declarations.
// - GLOBAL_ALLOC is the heap area for the global allocator.
//   Its base address and size in bytes are set in init_global_alloc()
//   above.
//

// 0x0500 - 0x2FFF (10KB+) : Heap area in 16-bit address space
static ALLOC_UNDER16: MuAlloc16 = unsafe { MuAlloc16::heap(0x0500, 0x2b00) };

// 0x60000 - 0x7FFFF (128KB) : Heap area in 20-bit address space
static ALLOC_UNDER20: MuAlloc16 = unsafe { MuAlloc16::heap(0x60000, 0x20000) };

// Heap area for global allocator in 32-bit address space
#[global_allocator]
static GLOBAL_ALLOC: MuAlloc32 = MuAlloc32::noheap();
