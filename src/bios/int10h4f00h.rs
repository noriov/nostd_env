/*!

BIOS INT 10h AX=4F00h : Return VBE Controller Information

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

use alloc::boxed::Box;
use core::alloc::Allocator;
use core::mem::{MaybeUninit, size_of};

use super::LmbiosRegs;
use crate::{print, println};
use crate::x86::{X86GetAddr, X86FarPtr};


#[doc(hidden)]
const DEBUG: bool = false;


/// Calls BIOS INT 10h AX=4F00h (Return VBE Controller Information).
pub fn call<A20>(alloc20: A20) -> Option<Box<VbeInfoBlock, A20>>
where
    A20: Allocator,
{
    // Allocate a buffer in 20-bit address space.
    let buf = Box::new_in(VbeInfoBlock::uninit(), alloc20);

    // Get the far pointer of the buffer.
    let buf_fp = buf.get_far_ptr()?;

    unsafe {
	// INT 10h AH=4Fh AL=00h
	// IN
	//   ES:DI = Address of VbeInfoBlock
	// OUT
	//   AX    = Status
	let mut regs = LmbiosRegs {
	    fun: 0x10,			// INT 10h
	    eax: 0x4f00,		// AH=4Fh AL=00h
	    edi: buf_fp.offset as u32,	// Offset of VbeInfoBlock
	    es: buf_fp.segment,		// Segment of VbeInfoBlock
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


/// VBE Controller Information
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

impl X86GetAddr for VbeInfoBlock {}

impl VbeInfoBlock {
    fn uninit() -> Self {
	unsafe {
	    MaybeUninit::<Self>::uninit().assume_init()
	}
    }

    #[inline]
    fn capabilities(&self) -> u32 {
	#[allow(unused_parens)]
	((self.capabilities[0] as u32) |
	 (self.capabilities[1] as u32) << 16)
    }
}


// Print struct members for debugging
impl VbeInfoBlock {
    pub fn print(&self) {
	println!("VbeInfoBlock:");
	println!("  Signature: {}{}{}{}",
		 self.signature[0] as char,
		 self.signature[1] as char,
		 self.signature[2] as char,
		 self.signature[3] as char);
	println!("  Version: {:#x}", self.version);
	Self::print_capabilities("Capabilities", self.capabilities());
	Self::print_mode_list("Mode List", self.video_mode_ptr);
	Self::print_cstr("OEM String", self.oem_string_ptr);
	println!("  Total Memory: {:#x}_0000", self.total_memory);
	println!("  OEM Software Revision: {:#x}", self.oem_software_rev);
	Self::print_cstr("OEM Vendor Name", self.oem_vendor_name_ptr);
	Self::print_cstr("OEM Product Name", self.oem_product_name_ptr);
	Self::print_cstr("OEM Product Revision", self.oem_product_rev_ptr);
    }

    fn print_capabilities(title: &str, capabilities: u32) {
	const PRINT_DEFAULT: bool = false;

	print!("  {}:", title);

	// Bit 0 : The Width of Digital-to-Analog Converter
	if (capabilities & (1 << 0)) != 0 {
	    print!(" DAC width = 6 or 8 bits,");
	} else if PRINT_DEFAULT {
	    print!(" DAC width = 6 bits,");
	}

	// Bit 1 : VGA Compatibility
	if (capabilities & (1 << 1)) != 0 {
	    print!(" not VGA Compatible,");
	} else if PRINT_DEFAULT {
	    print!(" VGA Compatible,");
	}

	// Bit 2 : RAMDAC operation
	if (capabilities & (1 << 2)) != 0 {
	    print!(" Special RAMDAC operation,");
	} else if PRINT_DEFAULT {
	    print!(" Normal RAMDAC operation,");
	}

	// Bit 3 : Hardware Stereoscopic Signaling Support
	if (capabilities & (1 << 3)) != 0 {
	    print!(" Hardware stereoscopic signaling support,");
	} else if PRINT_DEFAULT {
	    print!(" No hardware stereoscopic signaling support,");
	}

	// Bit 4 : Stereo Signaling Support Connector
	if (capabilities & (1 << 4)) != 0 {
	    print!(" Stereo signaling supported via VESA EVC connector,");
	} else if PRINT_DEFAULT {
	    print!(" Stereo signaling supported via external \
		    VESA stereo connector,");
	}

	println!();
    }

    fn print_mode_list(title: &str, far_ptr: [u16; 2]) {
	let mode_fp = X86FarPtr::from_array(far_ptr);
	let mode_ptr = mode_fp.to_linear_ptr::<u16>();

	print!("  {}:", title);

	let mut i: isize = 0;
	loop {
	    let mode = unsafe { *mode_ptr.offset(i) };
	    match mode {
		0xffff	=> break,
		_	=> print!(" {:04x}", mode),
	    }
	    i += 1;
	}

	println!();
    }

    fn print_cstr(title: &str, far_ptr: [u16; 2]) {
	let str_fp = X86FarPtr::from_array(far_ptr);
	let str_ptr = str_fp.to_linear_ptr::<u8>();

	print!("  {}: {} \"", title, str_fp);

	let mut i: isize = 0;
	loop {
	    let byte = unsafe { *str_ptr.offset(i) };
	    match byte {
		0x00 => break,
		0x20 ..= 0x7E | b'\n' | b'\r' => {
		    print!("{}", byte as char);
		},
		_ => print!("."),
	    }
	    i += 1;
	}

	println!("\"");
    }
}
