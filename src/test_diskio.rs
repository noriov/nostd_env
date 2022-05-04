/*!

Tests simple disk I/O using BIOS.

 */


use core::alloc::Allocator;

use crate::bios;
use crate::{print, println};
use crate::x86::X86GetAddr;



///
/// Tests simple disk I/O using
/// BIOS INT 13h AH=02h (Read Sectors From Drive).
///
/// It reads a couple of sectors of data from the boot drive, and
/// shows the first 16 bytes.
///
pub fn try_read_sectors1<A20>(alloc20: A20)
where
    A20: Allocator
{
    // Read sectors: CHS=(0, 0, 1), #sectors=3
    let (cylinder, head, sector, nsectors) = (0, 0, 1, 3);
    let drive_id = bios::get_boot_drive_id();

    print!("Read sectors: CHS=({}, {}, {}), nsectors={}, drive={:#x} ... ",
	   cylinder, head, sector, nsectors, drive_id);

    match bios::int13h02h::call(drive_id, cylinder, head, sector, nsectors,
				alloc20) {
	Some(vec) => {
	    println!("OK!");
	    dump(&vec, 16);
	},
	None => {
	    println!("failed");
	},
    }
}

///
/// Tests simple disk I/O using
/// BIOS INT 13h AH=42h (Extended Read Sectors From Drive).
///
/// It reads a couple of sectors of data from the boot drive, and
/// shows the first 16 bytes.
///
pub fn try_read_sectors2<A20>(alloc20: A20)
where
    A20: Allocator
{
    // Read sectors: LBA=1, #sectors=3
    let (lba, nsectors) = (1, 3);
    let drive_id = bios::get_boot_drive_id();

    print!("Read sectors: LBA={}, nsectors={}, drive={:#x} ... ",
	   lba, nsectors, drive_id);

    match bios::int13h42h::call(drive_id, lba, nsectors, alloc20) {
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
