/*!

Manages heap areas.

 */


use alloc::alloc::Layout;
use alloc::vec::Vec;
use core::alloc::Allocator;

use crate::bios::{self, int15he820h::AddrRange};
use crate::mu::{MuAlloc16, MuAlloc32};


// Heap area in 16-bit address space: 0x0500 - 0x2FFF (10KB+)
// Mainly for buffers to be exchanged with BIOS.
pub static ALLOC_UNDER16: MuAlloc16 =
    unsafe { MuAlloc16::heap(0x0500, 0x2b00) };

// Heap area in 20-bit address space: 0x60000 - 0x7FFFF (128KB)
// Mainly for buffers to be exchanged with BIOS.
pub static ALLOC_UNDER20: MuAlloc32 =
    unsafe { MuAlloc32::heap(0x60000, 0x20000) };

// Heap area in 64-bit address space: (Initialized in the function above)
// For the global allocator.
#[global_allocator]
pub static GLOBAL_ALLOC: MuAlloc32 = MuAlloc32::noheap();


#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("Failed to allocate {:?}", layout)
}


// Initialize the Global Allocator.
pub fn init_global_alloc<A20>(size: usize, alloc20: A20) -> Vec<AddrRange, A20>
where
    A20: Allocator,
{
    let lowest_addr = 1 << 20;  // Above 20-bit address space (i.e., above 1MB)

    if let Some(addr_ranges) = bios::int15he820h::call(alloc20) {
	for entry in &addr_ranges {
	    #[allow(unused_parens)]
	    if (entry.atype == AddrRange::TYPE_USABLE &&
		entry.addr >= lowest_addr && entry.length as usize >= size) {
		let base = entry.addr as usize;
		unsafe {
		    GLOBAL_ALLOC.lock().set_heap(base, size);
		}
		return addr_ranges;
	    }
	}
    }

    panic!("Failed to initialize the global allocator");
}
