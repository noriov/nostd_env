//
// Test a simple Disk I/O by usinig BIOS.
//
// Reads the first 512 bytes of data from the boot drive,
// and show the first 8 bytes.
//

use alloc::vec::Vec;
use core::alloc::Allocator;

use crate::bios;
use crate::println;


pub fn try_read_sectors<A16>(alloc: A16)
where
    A16: Copy + Allocator
{
    // Allocate 512 bytes of read buffer in 16-bit address space.
    let mut buf: Vec<u8, A16> = Vec::with_capacity_in(512, alloc);
    buf.resize(512, 0);
    let buf_addr = buf.as_ptr() as usize;
    assert!(buf_addr < (1_usize << 16));

    // Get the boot drive ID.
    let drive_id = bios::get_boot_drive_id();

    unsafe {
	// INT 13h AH=02h (Read Sectors From Drive)
	// AL: #Sectors, ECX: Cylinder and Sector, DH: Head, DL: Drive ID,
	// ES:BX : Buffer Address.
	// cf. https://en.wikipedia.org/wiki/INT_13H
	bios::LmbiosRegs {
	    fun: 0x13,				// INT 13h AH=02h
	    eax: 0x0201,			// AL: Number of sectors = 1
	    ecx: 0x0001,			// Cylinder = 0, Secotr = 1
	    edx: 0x0000 | drive_id as u32,	// Head = 0, Drive = drive_id
	    ebx: buf_addr as u32,		// Buffer address
	    ..Default::default()
	}.call();
    }

    // Print the results.
    println!();
    println!("Boot Drive ID = {:#x}", drive_id);
    println!("Data in LBA=1 of the boot drive are loaded at {:#x}",
	     buf_addr);
    println!("{:#x}: {:#x} {:#x} {:#x} {:#x} {:#x} {:#x} {:#x} {:#x} ..",
	     buf_addr,
	     buf[0], buf[1], buf[2], buf[3],
	     buf[4], buf[5], buf[6], buf[7]);
}
