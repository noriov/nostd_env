//
// BIOS INT 13h AH=02h (Read Sectors From Drive)
//
// Supplementary Resources:
//	https://en.wikipedia.org/wiki/INT_13H
//	https://en.wikipedia.org/wiki/Cylinder-head-sector
//

use alloc::vec::Vec;
use core::alloc::Allocator;

use super::LmbiosRegs;
use crate::mu::PushBulk;
use crate::x86::{FLAGS_CF, X86Addr};


pub struct Int13h02h;

const SECTOR_SIZE: usize = 512;


impl Int13h02h {
    pub fn call<A>(drive_id: u8, cylinder: u16, head: u8, sector: u8,
		   nsectors: u8, alloc: A) -> Option<Vec<u8, A>>
    where
	A: Allocator
    {
	let nbytes = (nsectors as usize) * SECTOR_SIZE;

	// Prepare a result buffer in 20-bit address space.
	let mut vec = Vec::new_in(alloc);

	unsafe {
	    vec.push_bulk(nbytes, | buf | {
		// Get segment and offset of buf.
		let (buf_seg, buf_off) = buf.get_rm16_addr().ok_or(())?;

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
		    ecx: Self::cylsec_to_cx(cylinder, sector) as u32,
		    edx: (head as u32) << 8 | drive_id as u32,
		    ebx: buf_off as u32,
		    es: buf_seg,
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

    #[inline]
    fn cylsec_to_cx(cylinder: u16, sector: u8) -> u16 {
	(cylinder & 0x00ff) << 8 | (cylinder & 0x0300) >> 2 | (sector as u16)
    }
}
