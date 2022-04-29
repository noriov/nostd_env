pub trait X86Addr
{
    #[inline]
    fn get_linear_addr(&self) -> usize {
	self as *const Self as *const () as usize
    }

    fn get_rm16_addr(&self) -> Option<(u16, u16)> {
	let linear_addr = self.get_linear_addr();

	if linear_addr < 1_usize << 20 {
	    let segment = (linear_addr >> 4) as u16;
	    let offset = (linear_addr as u16) & 0x000f;

	    if false {
		crate::println!("X86Addr: {:#x} or {:#x}:{:#x}",
				linear_addr, segment, offset);
	    }

	    Some((segment, offset))
	} else {
	    None
	}
    }
}

impl<T> X86Addr for [T] {}
