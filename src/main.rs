#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(allocator_api)]

mod bios;
mod mu;
mod text_writer;

extern crate alloc;
use alloc::alloc::Layout;
use core::arch::asm;
use core::panic::PanicInfo;

use crate::mu::{MuAlloc16, MuAlloc32};


#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    halt_forever();
}

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("Failed to allocate {:?}", layout)
}

fn halt_forever() -> ! {
    loop {
	unsafe {
	    asm!("hlt");
	}
    }
}


#[no_mangle]
pub extern "C" fn __bare_start() -> ! {
    println!("Hello, world!");

    // The unsafe block below
    // reads the first 512 bytes of data from the boot drive,
    // and show the first 8 bytes.
    unsafe {
	let drive_id = bios::ffi::lmbios_get_boot_drive_id();
	let buf_addr = 0x5000 as *const u8;

	// INT 13h AH=02h: Read Sectors From Drive
	// AL: #Sectors, ECX: Cylinder and Sector, DH: Head, DL: Drive ID,
	// ES:BX : Buffer Address.
	// cf. https://en.wikipedia.org/wiki/INT_13H
	bios::ffi::LmbiosRegs {
	    fun: 0x13,				// INT 13h AH=02h
	    eax: 0x0201,			// AL: Number of sectors = 1
	    ecx: 0x0001,			// Cylinder = 0, Secotr = 1
	    edx: 0x0000 | drive_id as u32,	// Head = 0, Drive = drive_id
	    ebx: buf_addr as u32,		// Buffer address
	    ..Default::default()
	}.call();

	println!();
	println!("Boot Drive ID = {:#x}", drive_id);
	println!("Data in LBA=1 of the boot drive are loaded at {:#x}",
		 buf_addr as usize);
	println!("{:#x}: {:#x} {:#x} {:#x} {:#x} {:#x} {:#x} {:#x} {:#x} ..",
		 buf_addr as usize,
		 *buf_addr.offset(0), *buf_addr.offset(1),
		 *buf_addr.offset(2), *buf_addr.offset(3),
		 *buf_addr.offset(4), *buf_addr.offset(5),
		 *buf_addr.offset(6), *buf_addr.offset(7));
    }

    halt_forever();
}


//
// Three allocators are initialized:
// - ALLOC_UNDER16 (in 16-bit address space) and ALLOC_UNDER20
//   (in 20-bit address space) are heap areas maily for buffers
//   to be exchanged with BIOS.  Their base address and size in bytes
//   are specified in their declarations.
// - GLOBAL_ALLOC is the heap area for the global allocator.
//

// 0x0500 - 0x2FFF (10KB+) : Heap area in 16-bit address space
static ALLOC_UNDER16: MuAlloc16 = unsafe { MuAlloc16::heap(0x0500, 0x2b00) };

// 0x60000 - 0x7FFFF (128KB) : Heap area in 20-bit address space
static ALLOC_UNDER20: MuAlloc16 = unsafe { MuAlloc16::heap(0x60000, 0x20000) };

// Heap area for global allocator in 32-bit address space
#[global_allocator]
static GLOBAL_ALLOC: MuAlloc32 = MuAlloc32::uninit();
