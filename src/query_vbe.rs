//
// Query VESA BIOS Extentions using BIOS INT 10h AX=4Fxxh
//

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::alloc::Allocator;

use crate::bios;
use crate::println;


const DEBUG: bool = false;


pub fn query_vbe<A>(width: u16, height: u16, bpp: u8, alloc: A) -> Option<u16>
where
    A: Copy + Allocator,
{
    let vbe_info = VbeInfo::query(alloc)?;

    if DEBUG {
	vbe_info.print();
    }

    let best_mode = vbe_info.find_best_mode(width, height, bpp);

    if true {
	for mode in &vbe_info.mode_list {
	    if mode.mode == best_mode {
		println!("Best mode={:#x}, width={}, height={}, bpp={}",
			 mode.mode, mode.width, mode.height, mode.bpp);
		break;
	    }
	}
    }

    Some(best_mode)
}


pub struct VbeInfo<A>
where
    A: Allocator,
{
    pub version: u16,
    pub capabilities: u32,
    pub total_memory: u32,
    pub mode_list: Vec<ModeInfo, A>,
    pub oem_software_rev: u16,
    pub oem_string: VbeString<A>,
    pub oem_vendor_name: VbeString<A>,
    pub oem_product_name: VbeString<A>,
    pub oem_product_rev: VbeString<A>,
}

impl<A> VbeInfo<A>
where
    A: Copy + Allocator,
{
    pub fn query(alloc: A) -> Option<Box<Self, A>> {
	let vbe_info_block = bios::int10h4f00h::call(alloc)?;
	let vbe_info = Self::from_vbe_info_block(&vbe_info_block, alloc);
	let buf = Box::new_in(vbe_info, alloc);
	Some(buf)
    }

    fn from_vbe_info_block(vib: &bios::VbeInfoBlock, alloc: A) -> Self {
	Self {
	    version: vib.version,
	    capabilities: ((vib.capabilities[0] as u32) |
			   (vib.capabilities[1] as u32) << 16),
	    total_memory: (vib.total_memory as u32) << 16,
	    mode_list: Self::get_mode_list(vib.video_mode_ptr, alloc),
	    oem_software_rev: vib.oem_software_rev,
	    oem_string: VbeString::new_in(vib.oem_string_ptr, alloc),
	    oem_vendor_name: VbeString::new_in(vib.oem_vendor_name_ptr, alloc),
	    oem_product_name: VbeString::new_in(vib.oem_product_name_ptr,
						alloc),
	    oem_product_rev: VbeString::new_in(vib.oem_product_rev_ptr, alloc),
	}
    }

    fn get_mode_list(far_ptr: [u16; 2], alloc: A) -> Vec<ModeInfo, A> {
	let mut mode_list = Vec::new_in(alloc);

	let addr = (far_ptr[1] as u32) << 4 | (far_ptr[0] as u32);
	let ptr = addr as usize as *const u16;

	let mut i: isize = 0;
	loop {
	    let mode = unsafe { *ptr.offset(i) };
	    if mode == 0xffff {
		break;
	    }
	    if let Some(mode_info) = ModeInfo::query(mode, alloc) {
		mode_list.push(*mode_info);
	    }
	    i += 1;
	}

	mode_list.shrink_to_fit();

	mode_list
    }

    fn find_best_mode(&self, width: u16, height: u16, bpp: u8) -> u16 {
	fn size_diff(x1: u16, y1: u16, x2: u16, y2: u16) -> i32 {
	    let x_diff = (x1 as i32) - (x2 as i32);
	    let y_diff = (y1 as i32) - (y2 as i32);
	    x_diff * x_diff + y_diff * y_diff
	}

	fn bpp_diff(bpp1: u8, bpp2: u8) -> u8 {
	    if bpp1 >= bpp2 {
		bpp1 - bpp2
	    } else {
		bpp2 - bpp1
	    }
	}

	let mut best_mode = 0xffff;
	let mut min_size_diff = i32::MAX;
	let mut min_bpp_diff = u8::MAX;

	for mode in &self.mode_list {
	    #[allow(unused_parens)]
	    if (!mode.is_graphics_mode() ||
		!mode.has_linear_frame_buffer_mode() ||
		!(mode.is_packed_pixel() || mode.is_direct_color())) {
		continue;
	    }

	    if mode.width == width && mode.height == height && bpp == mode.bpp{
		best_mode = mode.mode;
		break;
	    }

	    let cur_size_diff = size_diff(width, height,
					  mode.width, mode.height);
	    let cur_bpp_diff = bpp_diff(bpp, mode.bpp);
	    if cur_size_diff < min_size_diff {
		min_size_diff = cur_size_diff;
		min_bpp_diff = cur_bpp_diff;
		best_mode = mode.mode;
	    } else if cur_size_diff == min_size_diff {
		if cur_bpp_diff < min_bpp_diff {
		    min_bpp_diff = cur_bpp_diff;
		    best_mode = mode.mode;
		}
	    }
	}

	best_mode
    }

    fn print(&self) {
	println!("Version = {:#x}", self.version);
	println!("Capabilities: {:#x}", self.capabilities);
	println!("Total Memory = {:#x}", self.total_memory);
	for mode in &self.mode_list {
	    mode.print();
	}
	println!("OEM Software Revision = {:#x}", self.oem_software_rev);
	println!("OEM String = {}", self.oem_string);
	println!("OEM Vendor Name Ptr = {}", self.oem_vendor_name);
	println!("OEM Product Name Ptr = {}", self.oem_product_name);
	println!("OEM Product Revison Ptr = {}", self.oem_product_rev);
    }
}


pub struct ModeInfo {
    pub mode: u16,
    pub attributes: u16,
    pub width: u16,
    pub height: u16,
    pub bpp: u8,
    pub memory_model: u8,
    pub phy_base: u32,
}

impl ModeInfo {
    pub fn query<A>(mode: u16, alloc: A) -> Option<Box<Self, A>>
    where
	A: Copy + Allocator,
    {
	let mode_info_block = bios::int10h4f01h::call(mode, alloc)?;
	let mode_info = Self::from_mode_info_block(mode, &mode_info_block);
	let buf = Box::new_in(mode_info, alloc);
	Some(buf)
    }

    fn from_mode_info_block(mode: u16, mib: &bios::ModeInfoBlock) -> Self {
	Self {
	    mode,
	    attributes: mib.mode_attributes,
	    width: mib.x_resolution,
	    height: mib.y_resolution,
	    bpp: mib.bits_per_pixel,
	    memory_model: mib.memory_model,
	    phy_base: ((mib.phys_base_ptr[0] as u32) |
		       (mib.phys_base_ptr[1] as u32) << 16),
	}
    }

    fn is_graphics_mode(&self) -> bool {
	(self.attributes & 0x10) != 0
    }

    fn has_linear_frame_buffer_mode(&self) -> bool {
	(self.attributes & 0x80) != 0
    }

    fn is_packed_pixel(&self) -> bool {
	self.memory_model == 4
    }

    fn is_direct_color(&self) -> bool {
	self.memory_model == 6
    }

    fn print(&self) {
	println!("Video mode=0x{:04x}, attr={:#x}, \
		  width={}, height={}, bbp={}, \
		  memory_model={}, phy_base={:#x}",
		 self.mode, self.attributes,
		 self.width, self.height, self.bpp,
		 self.memory_model, self.phy_base);
    }
}


pub struct VbeString<A>
where
    A: Allocator,
{
    pub segment: u16,
    pub offset: u16,
    pub string: Vec<u8, A>,
}

impl<A> VbeString<A>
where
    A: Allocator,
{
    pub fn new_in(far_ptr: [u16; 2], alloc: A) -> Self {
	Self {
	    segment: far_ptr[1],
	    offset: far_ptr[0],
	    string: Self::get_cstr(far_ptr, alloc),
	}
    }

    fn get_cstr(far_ptr: [u16; 2], alloc: A) -> Vec<u8, A> {
	let mut string = Vec::new_in(alloc);

	let addr = ((far_ptr[1] as u32) << 4) | (far_ptr[0] as u32);
	let ptr = addr as usize as *const u8;
	let mut i: isize = 0;
	loop {
	    let ch = unsafe { *ptr.offset(i) };
	    if ch == 0 {
		break;
	    }
	    string.push(ch);
	    i += 1;
	}

	string.shrink_to_fit();

	string
    }
}

impl<A> core::fmt::Display for VbeString<A>
where
    A: Allocator,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
	if let Err(e) = write!(f, "{:#x}:{:#x} \"",
			       self.segment, self.offset) {
	    return Err(e);
	}

	for byte in &self.string {
	    let ch =
		match byte {
		    0x20 ..= 0x7E | b'\n' | b'\r' => *byte,
		    _ => b'.'
		};
	    if let Err(e) = write!(f, "{}", ch as char) {
		return Err(e);
	    }
	}

	write!(f, "\"")
    }
}
