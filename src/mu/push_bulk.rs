use alloc::vec::Vec;
use core::alloc::Allocator;
use core::result::Result;
use core::slice;


pub trait PushBulk<T, R, E> {
    unsafe fn push_bulk<F>(&mut self, nelem: usize, fill_new_slice: F)
			   -> Result<R, E>
    where
	F: FnMut(&mut [T]) -> Result<R, E>;
}

impl<T, R, E, A> PushBulk<T, R, E> for Vec<T, A>
where
    A: Allocator
{
    unsafe fn push_bulk<F>(&mut self, nelem: usize, mut fill_new_slice: F)
			   -> Result<R, E>
    where
	F: FnMut(&mut [T]) -> Result<R, E>
    {
	// Prepare enough size of hidden area.
	self.reserve(nelem);

	// Fill hidden area with caller-supplied closure `fill_new_slice`.
	// The hidden area is passed as an ephemeral slice (soon dropped).
	let result = fill_new_slice(
	    slice::from_raw_parts_mut(
		self.as_mut_ptr().offset(self.len() as isize),
		nelem)
	);

	// If the result is ok, extend the length.
	if result.is_ok() {
	    self.set_len(self.len() + nelem);
	}

	result
    }
}
