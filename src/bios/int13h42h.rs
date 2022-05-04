/*!

BIOS INT 13h AH=42h : Extended Read Sectors From Drive

# Supplementary Resources

* [INT 13H](https://en.wikipedia.org/wiki/INT_13H) (Wikipedia)

 */

//
// Supplementary Resource:
//	https://en.wikipedia.org/wiki/INT_13H
//

use alloc::vec::Vec;
use core::alloc::Allocator;
use core::cmp::min;
use core::mem::size_of;

use super::LmbiosRegs;
use crate::mu::PushBulk;
use crate::x86::{FLAGS_CF, X86GetAddr};


/// Sector Size = 512
const SECTOR_SIZE: usize = 512;

/// The maximum number of sectors that can be read by one BIOS call.
const MAX_NSECTORS: u16 = 127;


/// Calls BIOS INT 13h AH=42h (Extended Read Sectors From Drive).
pub fn call<A20>(drive_id: u8, lba: u64, nsectors: u16, alloc20: A20)
		 -> Option<Vec<u8, A20>>
where
    A20: Allocator
{
    // Prepare a result buffer in 20-bit address space.
    let total_nbytes = (nsectors as usize) * SECTOR_SIZE;
    let mut vec = Vec::with_capacity_in(total_nbytes, alloc20);

    let mut cur_lba = lba;
    let mut unread_nsectors = nsectors;

    loop {
	let cur_nsectors = min(unread_nsectors, MAX_NSECTORS);
	let cur_nbytes = (cur_nsectors as usize) * SECTOR_SIZE;

	unsafe {
	    vec.push_bulk(cur_nbytes, | buf | {
		// Get the far pointer of the buffer.
		let buf_fp = buf.get_far_ptr().ok_or(())?;

		// Allocate a buffer for DAP on the stack.
		let dap =
		    DiskAddressPacket {
			size: 0x10,
			reserved: 0,
			nsectors: cur_nsectors,
			buf_offset: buf_fp.offset,
			buf_segment: buf_fp.segment,
			lba: cur_lba,
		    };

		// Get the far pointer of the Disk Address Packet.
		let dap_fp = dap.get_far_ptr().ok_or(())?;

		// INT 13h AH=42h (Extended Read Sectors From Drive)
		// IN
		//   DL    = Drive ID
		//   DS:SI = DAP Address
		// OUT
		//   CF    = 0 if Ok, 1 if Err
		let mut regs = LmbiosRegs {
		    fun: 0x13,
		    eax: 0x4200,
		    edx: drive_id as u32,
		    esi: dap_fp.offset as u32,
		    ds: dap_fp.segment,
		    ..Default::default()
		};

		regs.call();

		// Check the results.
		// Note: On error, the carry flag (CF) is set.
		if (regs.flags & FLAGS_CF) == 0 {
		    Ok(())
		} else {
		    Err(())
		}
	    }).ok()?;
	}

	cur_lba += cur_nsectors as u64;
	unread_nsectors -= cur_nsectors;
	if unread_nsectors == 0 {
	    break;
	}
    }

    Some(vec)
}


/// Disk Address Packet
#[repr(C)]
#[derive(Default)]
struct DiskAddressPacket {
    pub size: u8,		//00   : Size of DAP = 0x10
    pub reserved: u8,		//01   : (reserved)  = 0x00
    pub nsectors: u16,		//02-03: Number of blocks to be loaded
    pub buf_offset: u16,	//04-05: Offset to memory buffer
    pub buf_segment: u16,	//06-07: Segment of memory buffer
    pub lba: u64,		//08-0F: Start block
}

const _: () = assert!(size_of::<DiskAddressPacket>() == 0x10);

impl X86GetAddr for DiskAddressPacket {}
