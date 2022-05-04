/*!

BIOS INT 10h AX=4F01h : Return VBE Mode Information

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
use crate::x86::X86GetAddr;


#[doc(hidden)]
const DEBUG: bool = false;


/// Calls BIOS INT 10h AX=4F01h (Return VBE Mode Information).
pub fn call<A20>(mode: u16, alloc20: A20) -> Option<Box<ModeInfoBlock, A20>>
where
    A20: Allocator,
{
    // Allocate a buffer in 20-bit address space.
    let buf = Box::new_in(ModeInfoBlock::uninit(), alloc20);

    // Get the far pointer of the buffer.
    let buf_fp = buf.get_far_ptr()?;

    unsafe {
	// INT 10h AH=4Fh AL=01h
	// IN
	//   CX    = Mode Number
	//   ES:DI = Address of ModeInfoBlock
	// OUT
	//   AX    = Status
	let mut regs = LmbiosRegs {
	    fun: 0x10,			// INT 10h
	    eax: 0x4f01,		// AH=4Fh AL=01h
	    ecx: mode as u32,		// Mode Number
	    edi: buf_fp.offset as u32,	// Offset of ModeInfoBlock
	    es: buf_fp.segment,		// Segment of ModeInfoBlock
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


/// VBE Mode Information
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ModeInfoBlock {
    pub mode_attributes: u16,		//00-01: Mode Attributes
    pub win_a_attributes: u8,		//02   : Window A Attributes
    pub win_b_attributes: u8,		//03   : Window B Attributes
    pub win_granularity: u16,		//04-05: Window Granularity
    pub win_size: u16,			//06-07: Window Size
    pub win_a_segment: u16,		//08-09: Window A Start Segment
    pub win_b_segment: u16,		//0A-0B: Window B Start Segment
    pub win_func_ptr: [u16; 2],		//0C-0F: Pointer to Window Function
    pub bytes_per_scan_line: u16,	//10-11: Bytes per Scan Line

    pub x_resolution: u16,		//12-13: Horizontal Resolution
    pub y_resolution: u16,		//14-15: Vertical Resolution
    pub x_char_size: u8,		//16   : Character Cell Width
    pub y_char_size: u8,		//17   : Character Cell Height
    pub number_of_planes: u8,		//18   : Number of Memory Planes
    pub bits_per_pixel: u8,		//19   : Bits per Pixel
    pub number_of_banks: u8,		//1A   : Number of Banks
    pub memory_model: u8,		//1B   : Memory Model Type
    pub bank_size: u8,			//1C   : Bank Size in KB
    pub number_of_image_pages: u8,	//1D   : Number of Images
    pub reserved0: u8,			//1E   : (reserved for page function)

    pub red_mask_size: u8,		//1F   : Size of Direct Color Red Mask
    pub red_field_position: u8,		//20   : Bit Pos of LSB of Red Mask
    pub green_mask_size: u8,		//21   : Size of Direct Color GreenMask
    pub green_field_position: u8,	//22   : Bit Pos of LSB of Green Mask
    pub blue_mask_size: u8,		//23   : Size of Direct Color Blue Mask
    pub blue_field_position: u8,	//24   : Bit Pos of LSB of Blue Mask
    pub rsvd_mask_size: u8,		//25   : Size of Direct Color Rsvd Mask
    pub rsvd_field_position: u8,	//26   : Bit Pos of LSB of Rsvd Mask
    pub direct_color_mode_info: u8,	//27   : Direct Color Mode Attributes

    pub phys_base_ptr: [u16; 2],	//28-2B: Physical Address for FrameBuf
    pub reserved1: u32,			//2C-2F: (reserved)
    pub reserved2: u16,			//30-31: (reserved)

    pub lin_bytes_per_scan_line: u16,	//32-33: Bytes per Scan Line for Linear
    pub bnk_number_of_image_pages: u8,	//34   : Number of Images for Banked
    pub lin_number_of_image_pages: u8,	//35   : Number of Images for Linear
    pub lin_red_mask_size: u8,		//36   : Size of Direct Color Red Mask
    pub lin_red_field_position: u8,	//37   : Bit Pos of LSB of Red Mask
    pub lin_green_mask_size: u8,	//38   : Size of Direct Color GreenMask
    pub lin_green_field_position: u8,	//39   : Bit Pos of LSB of Green Mask
    pub lin_blue_mask_size: u8,		//3A   : Size of Direct Color Blue Mask
    pub lin_blue_field_position: u8,	//3B   : Bit Pos of LSB of Blue Mask
    pub lin_rsvd_mask_size: u8,		//3C   : Size of Direct Color Rsvd Mask
    pub lin_rsvd_field_position: u8,	//3D   : Bit Pos of LSB of Rsvd Mask
    pub max_pixel_clock: [u16; 2],	//3E-41: Maximum Pixel Clock (Hz) (u32)

    pub reserved4: [u8; 190],		//42-FF: (reserved)
}

const _: () = assert!(size_of::<ModeInfoBlock>() == 0x100);

impl X86GetAddr for ModeInfoBlock {}

impl ModeInfoBlock {
    pub const ATTR_GRAPHICS: u16 = 1 << 4;
    pub const ATTR_FRAME_BUF: u16 = 1 << 7;
    pub const MEM_TEXT: u8 = 0;
    pub const MEM_PACKED_PIXEL: u8 = 4;
    pub const MEM_DIRECT_COLOR: u8 = 6;

    fn uninit() -> Self {
	unsafe {
	    MaybeUninit::<Self>::uninit().assume_init()
	}
    }

    #[inline]
    pub fn phys_base_ptr(&self) -> u32 {
	#[allow(unused_parens)]
	((self.phys_base_ptr[0] as u32) |
	 (self.phys_base_ptr[1] as u32) << 16)
    }
}


// Print struct members for debugging
impl ModeInfoBlock {
    pub fn print(&self) {
	println!("ModeInfoBlock:");

	print!("  Attributes: {:#x}", self.mode_attributes);
	if (self.mode_attributes & Self::ATTR_GRAPHICS) != 0 {
	    print!(", Graphics");
	}
	if (self.mode_attributes & Self::ATTR_FRAME_BUF) != 0 {
	    print!(", Linear Frame Buffer");
	}
	println!();

	print!("  Memory Model: {} = ", self.memory_model);
	match self.memory_model {
	    Self::MEM_TEXT => print!("Text Mode"),
	    Self::MEM_PACKED_PIXEL => print!("Packed Pixel"),
	    Self::MEM_DIRECT_COLOR => print!("Direct Color"),
	    _ => {},
	}
	println!();

	println!("  Resolutions: (x, y) = ({}, {}), bpp={}",
		 self.x_resolution, self.y_resolution, self.bits_per_pixel);

	if self.phys_base_ptr() != 0 {
	    println!("  Frame Buffer Address: {:#x}", self.phys_base_ptr());
	}
    }
}
