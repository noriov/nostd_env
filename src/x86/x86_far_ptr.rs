use core::fmt;


/// X86 Far Pointer (i.e., segment and offset)
pub struct X86FarPtr {
    pub offset: u16,
    pub segment: u16,
}

impl X86FarPtr {
    /// Returns null far pointer.
    pub fn null() -> Self {
	Self {
	    offset: 0,
	    segment: 0,
	}
    }

    /// Converts an array [u16; 2] containing offset and segment
    /// into an X86 far pointer.
    pub fn from_array(far_ptr: [u16; 2]) -> Self {
	Self {
	    offset: far_ptr[0],
	    segment: far_ptr[1],
	}
    }

    /// Converts an linear address into an X86 far pointer
    /// if the linear address is in 20-bit address space.
    pub fn from_linear_addr(linear_addr: usize) -> Option<Self> {
	if linear_addr < (1_usize << 20) {
	    Some(Self {
		offset: (linear_addr as u16) & 0x000f,
		segment: (linear_addr >> 4) as u16,
	    })
	} else {
	    None
	}
    }

    /// Converts the X86 far pointer into a linear address.
    pub fn to_linear_addr(&self) -> usize {
	(self.segment as usize) << 4 | (self.offset as usize)
    }

    /// Converts the X86 far pointer into a linear address pointer.
    pub fn to_linear_ptr<T>(&self) -> *const T {
	self.to_linear_addr() as *const T
    }

    /// Converts the X86 far pointer into a linear address mutable pointer.
    pub fn to_linear_mut_ptr<T>(&self) -> *mut T {
	self.to_linear_addr() as *mut T
    }
}


impl fmt::Display for X86FarPtr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
	write!(f, "{:#x}:{:#x}", self.segment, self.offset)
    }
}
