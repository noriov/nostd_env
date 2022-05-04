/*!

Manages vide mode.

It finds the best video mode using VESA BIOS Extentions (INT 10h AX=4Fxxh).

*/


use core::alloc::Allocator;

use crate::bios;
use crate::bios::int10h4f01h::ModeInfoBlock;
use crate::{print, println};
use crate::x86::X86FarPtr;

const DEBUG: bool = false;


pub fn find_graphics_mode<A20>(width: u16, height: u16, bpp: u8, alloc20: A20)
			       -> Option<u16>
where
    A20: Copy + Allocator,
{
    {
	let cur_mode = VbeMode::get_mode();

	if DEBUG {
	    print!("Current ");
	    cur_mode.print(alloc20);
	}

	if false {
	    cur_mode.set_mode(0);
	}
    }

    {
	let best_mode = VbeMode::find_graphics_mode(width, height, bpp,
						    alloc20)?;

	if DEBUG {
	    print!("Best ");
	    best_mode.print(alloc20);
	}

	if false {
	    best_mode.set_mode(VbeMode::USE_FRAME_BUFFER);
	}

	Some(best_mode.mode)
    }
}


pub struct VbeMode {
    pub mode: u16,
}

impl VbeMode {
    pub const USE_FRAME_BUFFER: u16 = 1 << 14;

    pub fn find_graphics_mode<A20>(width: u16, height: u16, bpp: u8,
				   alloc20: A20) -> Option<Self>
    where
	A20: Copy + Allocator,
    {
	let vbe_info_block = bios::int10h4f00h::call(alloc20)?;

	if DEBUG {
	    vbe_info_block.print();
	}

	let mode_fp = X86FarPtr::from_array(vbe_info_block.video_mode_ptr);
	let mode_ptr = mode_fp.to_linear_ptr::<u16>();

	let mut desired_size = DesiredSize::new(width, height, bpp);

	let mut i: isize = 0;
	loop {
	    let mode = unsafe { *mode_ptr.offset(i) };
	    if mode == 0xffff {
		break;
	    }

	    let mib = bios::int10h4f01h::call(mode, alloc20)?;

	    #[allow(unused_parens)]
	    if (((mib.mode_attributes & ModeInfoBlock::ATTR_GRAPHICS) != 0 &&
		 (mib.mode_attributes & ModeInfoBlock::ATTR_FRAME_BUF) != 0) &&
		((mib.memory_model == ModeInfoBlock::MEM_PACKED_PIXEL ||
		  mib.memory_model == ModeInfoBlock::MEM_DIRECT_COLOR))) {
		if desired_size.does_match(mode,
					   mib.x_resolution,
					   mib.y_resolution,
					   mib.bits_per_pixel) {
		    break;
		}
	    }

	    i += 1;
	}

	let best_mode = desired_size.get_best_mode();

	Some(Self { mode: best_mode } )
    }

    pub fn get_mode() -> Self {
	Self {
	    mode: bios::int10h4f03h::call(),
	}
    }

    pub fn set_mode(&self, flags: u16) -> bool {
	bios::int10h4f02h::call(self.mode | flags, None)
    }

    pub fn print<A20>(&self, alloc20: A20)
    where
	A20: Allocator,
    {
	if let Some(mib) = bios::int10h4f01h::call(self.mode, alloc20) {
	    println!("mode = 0x{:04x}", self.mode);
	    mib.print();
	} else {
	    println!("mode=0x{:04x}: Failed to get ModeInfoBlock", self.mode);
	}
    }
}


struct DesiredSize {
    // Desired Size and BPP (fixed variables)
    x: u16,			// Desired Width
    y: u16,			// Desired Height
    bpp: u8,			// Desired Bits per Pixel (BPP)

    // Current Best Mode (working variables)
    best_mode: u16,		// Current Best Mode Number
    min_size_diff: i32,		// Square of the distance from the desired size
    min_bpp_diff: u8,		// Difference from the desired bpp
}

impl DesiredSize {
    // Returns the initial value.
    fn new(x: u16, y: u16, bpp: u8) -> Self {
	Self {
	    x: x,			// Desired Width
	    y: y,			// Desired Height
	    bpp: bpp,			// Desired Bits per Pixel (BPP)
	    best_mode: 0xffff,		// = No mode is examined.
	    min_size_diff: i32::MAX,	// = Theoretically worst value
	    min_bpp_diff: u8::MAX,	// = Theoretically worst value
	}
    }

    // Returns the square of the distance from the desired size.
    fn size_diff(&self, x: u16, y: u16) -> i32 {
	let x_diff = (self.x as i32) - (x as i32);
	let y_diff = (self.y as i32) - (y as i32);
	x_diff * x_diff + y_diff * y_diff
    }

    // Returns the difference from the desired bits per pixel (bpp).
    fn bpp_diff(&self, bpp: u8) -> u8 {
	if self.bpp >= bpp {
	    self.bpp - bpp
	} else {
	    bpp - self.bpp
	}
    }

    // If given (x, y, bpp) are equal to the desized numbers, returns true.
    // Otherwise, if given (x, y, bpp) are better than the current best mode,
    // save the mode number.  Then, returns false.
    fn does_match(&mut self, mode: u16, x: u16, y: u16, bpp: u8) -> bool {
	if self.x == x && self.y == y && self.bpp == bpp {
	    // Equal to the desired numbers.
	    self.min_size_diff = 0;
	    self.min_bpp_diff = 0;
	    self.best_mode = mode;
	    true
	} else {
	    let size_diff = self.size_diff(x, y);
	    let bpp_diff = self.bpp_diff(bpp);

	    if size_diff < self.min_size_diff {
		// Better size than the current best mode.
		self.min_size_diff = size_diff;
		self.min_bpp_diff = bpp_diff;
		self.best_mode = mode;
	    } else if size_diff == self.min_size_diff {
		if bpp_diff < self.min_bpp_diff {
		    // Better bpp than the current best mode.
		    self.min_bpp_diff = bpp_diff;
		    self.best_mode = mode;
		}
	    }
	    false
	}
    }

    // Returns the current best mode number.
    fn get_best_mode(&self) -> u16 {
	self.best_mode
    }
}
