#![no_std]
#![no_main]

mod bios;
mod text_writer;

use core::arch::asm;
use core::panic::PanicInfo;


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    halt_forever();
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

    // Read the first 512 bytes of data from the boot drive,
    // and show the first 8 bytes.
    unsafe {
	let drive_id = bios::ffi::lmbios_get_boot_drive_id();
	let buf_addr = 0x4000 as *const u8;

	// INT 13h AH=02h: Read Sectors From Drive
	// AL: #Sectors, ECX: Cylinder and Sector, DH: Head, D: Drive,
	// ES:BX : Buffer Address.
	// cf. https://en.wikipedia.org/wiki/INT_13H
	let mut regs = bios::ffi::LmbiosRegs {
	    fun: 0x13,				// INT 13h AH=02h
	    eax: 0x0201,			// AL: Number of sectors = 1
	    ecx: 0x0001,			// Cylinder = 0, Secotr = 1
	    edx: 0x0000 | drive_id as u32,	// Head = 0, Drive = drive_id
	    ebx: buf_addr as u32,		// Buffer address
	    ..Default::default()
	};
	bios::ffi::lmbios_call(&mut regs);

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
