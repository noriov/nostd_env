/*!

BIOS INT 13h AH=02h : Read Sectors From Drive

# Supplementary Resources

* [INT 13H](https://en.wikipedia.org/wiki/INT_13H) (Wikipedia)
* [Cylinder-head-sector](https://en.wikipedia.org/wiki/Cylinder-head-sector) (Wikipedia)

 */

//
// Supplementary Resources:
//	https://en.wikipedia.org/wiki/INT_13H
//	https://en.wikipedia.org/wiki/Cylinder-head-sector
//

use alloc::vec::Vec;
use core::alloc::Allocator;

use super::LmbiosRegs;
use crate::mu::PushBulk;
use crate::x86::{FLAGS_CF, X86GetAddr};


/// Sector Size = 512
const SECTOR_SIZE: usize = 512;


/// Calls BIOS INT 13h AH=02h (Read Sectors From Drive).
pub fn call<A20>(drive_id: u8, cylinder: u16, head: u8, sector: u8,
		 nsectors: u8, alloc20: A20) -> Option<Vec<u8, A20>>
where
    A20: Allocator
{
    let nbytes = (nsectors as usize) * SECTOR_SIZE;

    // Prepare a result buffer in 20-bit address space.
    let mut vec = Vec::new_in(alloc20);

    unsafe {
	vec.push_bulk(nbytes, | buf | {
	    // Get the far pointer of the buffer.
	    let buf_fp = buf.get_far_ptr().ok_or(())?;

	    // INT 13h AH=02h (Read Sectors From Drive)
	    // IN
	    //   AL    = Number of Sectors
	    //   CX    = Cylinder and Sector
	    //   DH    = Head
	    //   DL    = Drive ID
	    //   ES:BX = Buffer Address
	    // OUT
	    //   CF    = 0 if Ok, 1 if Err
	    let mut regs = LmbiosRegs {
		fun: 0x13,
		eax: 0x0200 | (nsectors as u32),
		ecx: cylsec_to_cx(cylinder, sector) as u32,
		edx: (head as u32) << 8 | drive_id as u32,
		ebx: buf_fp.offset as u32,
		es: buf_fp.segment,
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

    Some(vec)
}

/// Calculate the CX register value from the cylinder number
/// (0 to 1023) and the sector number (1 to 63).
#[inline]
fn cylsec_to_cx(cylinder: u16, sector: u8) -> u16 {
    (cylinder & 0x00ff) << 8 | (cylinder & 0x0300) >> 2 | (sector as u16)
}
