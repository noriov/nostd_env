//
// Micro Alloc - An implementation of alloc::GlobalAlloc and alloc::Allocator
//

use core::{
    alloc::{Allocator, AllocError, GlobalAlloc, Layout},
    ops::Deref,
    ptr::NonNull,
    slice,
};

use super::{
    mu_heap::{MuHeap, MuHeapIndex},
    mu_mutex::MuMutex,
};


//
// MuAlloc - A mutex'ed allocator backed by MuHeap
//

pub type MuAlloc16 = MuAlloc<i16>;
pub type MuAlloc32 = MuAlloc<i32>;

pub struct MuAlloc<I>
where
    I: MuHeapIndex
{
    heap: MuMutex<MuHeap<I>>,
}

impl<I> MuAlloc<I>
where
    I: MuHeapIndex
{
    pub const unsafe fn heap(given_base: usize, given_size: usize) -> Self {
	Self {
	    heap: MuMutex::new(MuHeap::<I>::heap(given_base, given_size)),
	}
    }

    pub const fn uninit() -> Self {
	Self {
	    heap: MuMutex::new(MuHeap::<I>::uninit())
	}
    }
}

impl<I> Deref for MuAlloc<I>
where
    I: MuHeapIndex
{
    type Target = MuMutex<MuHeap<I>>;
    fn deref(&self) -> &MuMutex<MuHeap<I>> {
	&self.heap
    }
}


//
// An implementation of alloc::GlobalAlloc
//
unsafe impl<I> GlobalAlloc for MuAlloc<I>
where
    I: MuHeapIndex
{
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
	self.lock().alloc(layout.size(), layout.align())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
	self.lock().dealloc(ptr, layout.size(), layout.align());
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize)
		      -> *mut u8 {
	if new_size < layout.size() {
	    self.lock().shrink(ptr, layout.size(), new_size, layout.align())
	} else if new_size > layout.size() {
	    self.lock().grow(ptr, layout.size(), new_size, layout.align())
	} else {
	    ptr
	}
    }
}


//
// An implementation of alloc::Allocator
//
unsafe impl<I> Allocator for &MuAlloc<I>
where
    I: MuHeapIndex
{
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
	unsafe {
	    let ptr = self.lock().alloc(layout.size(),
					layout.align());
	    alloc_result(ptr, layout.size())
	}
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
	self.lock().dealloc(ptr.as_ptr(),
			    layout.size(),
			    layout.align());
    }

    unsafe fn grow(&self, ptr: NonNull<u8>,
		   old_layout: Layout, new_layout: Layout)
		   -> Result<NonNull<[u8]>, AllocError> {
	let ptr = self.lock().grow(ptr.as_ptr(),
				   old_layout.size(),
				   new_layout.size(),
				   old_layout.align());
	alloc_result(ptr, new_layout.size())
    }

    unsafe fn shrink(&self, ptr: NonNull<u8>,
		     old_layout: Layout, new_layout: Layout)
		     -> Result<NonNull<[u8]>, AllocError> {
	let ptr = self.lock().shrink(ptr.as_ptr(),
				     old_layout.size(),
				     new_layout.size(),
				     old_layout.align());
	alloc_result(ptr, new_layout.size())
    }
}

unsafe fn alloc_result(ptr: *mut u8, size: usize)
		-> Result<NonNull<[u8]>, AllocError> {
    if !ptr.is_null() {
	let slice = slice::from_raw_parts_mut(ptr, size);
	Ok(NonNull::new(slice).unwrap())
    } else {
	Err(AllocError)
    }
}
