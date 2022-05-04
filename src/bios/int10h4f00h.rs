//
// BIOS INT 10h AX=4F00h (VESA BIOS Extentions)
//
// Resource:
//	VESA BIOS Extension Core Function Standard Version 3.0 (1998-09-16)
//	http://www.petesqbsite.com/sections/tutorials/tuts/vbe3.pdf
//
// Supplementary Resources:
//	https://wiki.osdev.org/VESA_Video_Modes
//
//	Display Technology Information Repository and Utilities
//	https://glenwing.github.io/docs/
//

use alloc::boxed::Box;
use core::alloc::Allocator;
use core::mem::{MaybeUninit, size_of};

use super::LmbiosRegs;
use crate::println;
use crate::x86::X86Addr;


const DEBUG: bool = false;


pub fn call<A>(alloc: A) -> Option<Box<VbeInfoBlock, A>>
where
    A: Allocator,
{
    // Allocate a buffer in 20-bit address space.
    let buf = Box::new_in(VbeInfoBlock::uninit(), alloc);

    // Get segment and offset of buf.
    let (buf_seg, buf_off) = buf.get_rm16_addr()?;

    unsafe {
	// INT 10h AH=4Fh AL=00h
	// IN
	//   ES:DI = Buffer Address
	// OUT
	//   AX    = Status
	let mut regs = LmbiosRegs {
	    fun: 0x10,			// INT 10h
	    eax: 0x4f00,		// AH=4Fh AL=00h
	    edi: buf_off as u32,	// Buffer Address
	    es: buf_seg,		// Buffer Address
	    ..Default::default()
	};

	if DEBUG {
	    println!("IN:  EAX={:#x}, ES:EDI={:#x}:{:#x}",
		     regs.eax, regs.es, regs.edi);
	}

	regs.call();

	if DEBUG {
	    println!("OUT: EAX={:#x}",
		     regs.eax);
	}

	// Check whether an error is detected.
	// Note: If successful, AL = 0x4f and AH = 0x00.
	if (regs.eax & 0xffff) != 0x004f {
	    return None;
	}
    }

    // Return the result.
    Some(buf)
}


#[repr(C)]
#[derive(Clone, Copy)]
pub struct VbeInfoBlock {
    pub signature: [u8; 4],		//00-03: VBE Signature
    pub version: u16,			//04-05: VBE Version
    pub oem_string_ptr: [u16; 2],	//06-09: OEM String (far ptr)
    pub capabilities: [u16; 2],		//0A-0D: Capabilities of gra ctrl (u32)
    pub video_mode_ptr: [u16; 2],	//0E-11: Video Mode List (far ptr)
    pub total_memory: u16,		//12-13: Number of 64KB memory blocks

    // Added for VBE 2.0+
    pub oem_software_rev: u16,		//14-15: VBE impl Software Revision
    pub oem_vendor_name_ptr: [u16; 2],	//16-19: Vender Name String (far ptr)
    pub oem_product_name_ptr: [u16; 2],	//1A-1D: Product Name String (far ptr)
    pub oem_product_rev_ptr: [u16; 2],	//1E-21: Product Rev String (far ptr)
    pub reserved: [u8; 222],		//22-FF: (reserved)

    pub oem_data: [u8; 256],		//100-1FF: Data Area for OEM Strings
}

const _: () = assert!(size_of::<VbeInfoBlock>() == 0x200);

impl X86Addr for VbeInfoBlock {}

impl VbeInfoBlock {
    fn uninit() -> Self {
	unsafe {
	    MaybeUninit::<Self>::uninit().assume_init()
	}
    }
}
