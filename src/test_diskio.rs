//
// Test a simple Disk I/O usinig
//	BIOS INT 13h AH=02h (Read Sectors From Drive) and
//	BIOS INT 13h AH=42h (Extended Read Sectors From Drive)
//
// Reads a couple of sectors of data from the boot drive,
// and show the first 16 bytes.
//

use core::alloc::Allocator;

use crate::bios;
use crate::{print, println};
use crate::x86::X86Addr;


pub fn try_read_sectors1<A>(alloc: A)
where
    A: Allocator
{
    // Read sectors: CHS=(0, 0, 1), #sectors=3
    let (cylinder, head, sector, nsectors) = (0, 0, 1, 3);
    let drive_id = bios::get_boot_drive_id();

    print!("Read sectors: CHS=({}, {}, {}), nsectors={}, drive={:#x} ... ",
	   cylinder, head, sector, nsectors, drive_id);

    match bios::Int13h02h::call(drive_id, cylinder, head, sector, nsectors,
				alloc) {
	Some(vec) => {
	    println!("OK!");
	    dump(&vec, 16);
	},
	None => {
	    println!("failed");
	},
    }
}

pub fn try_read_sectors2<A>(alloc: A)
where
    A: Allocator
{
    // Read sectors: LBA=1, #sectors=3
    let (lba, nsectors) = (1, 3);
    let drive_id = bios::get_boot_drive_id();

    print!("Read sectors: LBA={}, nsectors={}, drive={:#x} ... ",
	   lba, nsectors, drive_id);

    match bios::Int13h42h::call(drive_id, lba, nsectors, alloc) {
	Some(vec) => {
	    println!("OK!");
	    dump(&vec, 16);
	},
	None => {
	    println!("failed");
	},
    }
}

fn dump(buf: &[u8], n: usize) {
    print!("{:#x}:", buf.get_linear_addr());
    for i in 0 .. n {
	print!(" {:02x}", buf[i]);
    }
    println!();
}
