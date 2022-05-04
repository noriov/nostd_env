use super::X86FarPtr;

/// Get the address of `self` and converts it into an X86 far pointer.
pub trait X86GetAddr {
    /// Get the address of `self` and converts it into usize.
    #[inline]
    fn get_linear_addr(&self) -> usize {
	self as *const Self as *const () as usize
    }

    /// Get the address of `self` and converts it into an X86 far pointer.
    fn get_far_ptr(&self) -> Option<X86FarPtr> {
	X86FarPtr::from_linear_addr(self.get_linear_addr())
    }
}

impl<T> X86GetAddr for [T] {}
