//
// BIOS INT 10h AX=4F01h (VESA BIOS Extentions)
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


pub struct Int10h4f01h;

impl Int10h4f01h {
    pub fn call<A>(mode: u16, alloc: A) -> Option<Box<ModeInfoBlock, A>>
    where
	A: Allocator,
    {
	// Allocate a buffer in 20-bit address space.
	let buf = Box::new_in(ModeInfoBlock::uninit(), alloc);

	// Get segment and offset of buf.
	let (buf_seg, buf_off) = buf.get_rm16_addr()?;

	unsafe {
	    // INT 10h AH=4Fh AL=01h
	    // IN
	    //   CX    = Mode
	    //   ES:DI = Buffer Address
	    // OUT
	    //   AX    = Status
	    let mut regs = LmbiosRegs {
		fun: 0x10,		// INT 10h
		eax: 0x4f01,		// AH=4Fh AL=01h
		ecx: mode as u32,	// Mode
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
	    if regs.eax != 0x004f {
		return None;
	    }
	}

	// Return the result.
	Some(buf)
    }
}


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

impl X86Addr for ModeInfoBlock {}

impl ModeInfoBlock {
    fn uninit() -> Self {
	unsafe {
	    MaybeUninit::<Self>::uninit().assume_init()
	}
    }
}
