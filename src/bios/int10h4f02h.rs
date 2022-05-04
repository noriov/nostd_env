/*!

BIOS INT 10h AX=4F02h : Set VBE Mode

# Resource

* [VESA BIOS Extension Core Function Standard Version 3.0](http://www.petesqbsite.com/sections/tutorials/tuts/vbe3.pdf) (VESA, 1998-09-16)

# Supplementary Resources

* [VESA Video Modes](https://wiki.osdev.org/VESA_Video_Modes) (OS Dev)
* [Display Industry Standards Archive](https://glenwing.github.io/docs/) (Glen Wing)

 */

//
// Resource:
//	"VESA BIOS Extension Core Function Standard Version 3.0" (1998-09-16)
//	http://www.petesqbsite.com/sections/tutorials/tuts/vbe3.pdf
//
// Supplementary Resources:
//	https://wiki.osdev.org/VESA_Video_Modes
//
//	"Display Industry Standards Archive"
//	https://glenwing.github.io/docs/
//

use core::mem::size_of;

use super::LmbiosRegs;
use crate::println;
use crate::x86::{X86GetAddr, X86FarPtr};


#[doc(hidden)]
const DEBUG: bool = true;


/// Calls BIOS INT 10h AX=4F02h (Set VBE Mode).
pub fn call(mode: u16, crtc_info_block: Option<CRTCInfoBlock>) -> bool
{
    let buf_fp;
    if let Some(crtc_info_block) = crtc_info_block {
	// Get the far pointer of the buffer.
	if let Some(far_ptr) = crtc_info_block.get_far_ptr() {
	    buf_fp = far_ptr;
	} else {
	    return false;
	}
    } else {
	buf_fp = X86FarPtr::null();
    }

    unsafe {
	// INT 10h AH=4Fh AL=02h
	// IN
	//   BX    = Desired Mode to set
	//   ES:DI = Address of CRTCInfoBlock
	// OUT
	//   AX    = Status
	let mut regs = LmbiosRegs {
	    fun: 0x10,			// INT 10h
	    eax: 0x4f02,		// AH=4Fh AL=02h
	    ebx: mode as u32,		// Desired Mode to set
	    edi: buf_fp.offset as u32,	// Offset of CRTCInfoBlock
	    es: buf_fp.segment,		// Segment of CRTCInfoBlock
	    ..Default::default()
	};

	if DEBUG {
	    println!("IN:  EAX={:#x}, EBX={:#x}, ES:EDI={:#x}:{:#x}",
		     regs.eax, regs.ebx, regs.es, regs.edi);
	}

	regs.call();

	if DEBUG {
	    println!("OUT: EAX={:#x}",
		     regs.eax);
	}

	// Check whether an error is detected.
	// Note: If successful, AL = 0x4f and AH = 0x00.
	if (regs.eax & 0xffff) != 0x004f {
	    return false;
	}
    }

    // Return the result.
    true
}


/// CRTC Information Block
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CRTCInfoBlock {
    pub horizontal_total: u16,		//00-01: Horizontal Total in Pixels
    pub horizontal_sync_start: u16,	//02-03: Horizontal Sync Start
    pub horizontal_sync_end: u16,	//04-05: Horizontal Sync End
    pub vertical_total: u16,		//06-07: Vertical Total in Lines
    pub vertical_sync_start: u16,	//08-09: Vertical Sync Start
    pub vertical_sync_end: u16,		//0A-0B: Vertical Sync End
    pub flags: u8,			//0C   : Flags
    pub pixel_clock: [u8; 4],		//0D-10: Pixel Clock in Hz (u32)
    pub refresh_rate: [u8; 2],		//11-12: Refresh Rate in 0.01 Hz
    pub reserved: [u8; 41],		//13-3A: (reserved)
}

const _: () = assert!(size_of::<CRTCInfoBlock>() == 0x3c);

impl X86GetAddr for CRTCInfoBlock {}
