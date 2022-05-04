use core::fmt;
use super::ffi;


/// Reports stack usage.
pub struct StackUsage {
    pub start: usize,
    pub end: usize,
    pub max: usize,
}

impl StackUsage {
    pub fn new() -> Self {
	unsafe {
	    Self {
		start: &ffi::__lmb_stack_start as *const u8 as usize,
		end: &ffi::__lmb_stack_end as *const u8 as usize,
		max: ffi::debug_clear_stack_area() as usize,
	    }
	}
    }
}

impl fmt::Display for StackUsage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
	write!(f, "{:#x} in {:#x} - {:#x}", self.max, self.start, self.end)
    }
}
