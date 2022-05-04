//
// Micro Heap - A first-fit memroy allocator using doubly linked list.
//

use core::{
    cmp::{PartialOrd, min},
    fmt,
    mem::size_of,
    ops,
    ptr::{copy_nonoverlapping, null_mut},
    slice,
};

use crate::println;


#[doc(hidden)] const DEBUG_HEAP: bool = false;
#[doc(hidden)] const DEBUG_PRIOR_CHECK: bool = false;
#[doc(hidden)] const DEBUG_POST_CHECK: bool = true;
#[doc(hidden)] const DEBUG_CHECK_PTR: bool = true;
#[doc(hidden)] const DEBUG_FILL_JUNK: bool = false;


///
/// Provides a first-fit memroy allocator using doubly linked list.
///
/// `MuHeap` manages a heap area as an array of struct `HeapCell`s.
///
/// # Struct HeapCell
///
/// A heap area is an array of struct `HeapCell`s, whose use cases can
/// be categorized into two: management cells and data cells.
///
/// The management cells form a doubly linked list using two signed
/// integer fields: the `prev` and `next` fields.
///
/// * The `prev` field holds the index of the previous cell in the
///   cell list.
///
/// * The `next` field holds the index of the next cell in the cell
///   list.
///
/// The data cells are sandwiched between two management cells.  They
/// hold data instead of indexes.
///
/// * If both the `next` field of the prepending management cell and the
///   `prev` field of the postpending management cell are non-negative,
///   those data cells between them are in use.
///   (i.e., in use if indexes >= 0)
///
/// * If both the `next` field of the prepending management cell and the
///   `prev` field of the postpending management cell are negative,
///   those data cells between them are free.
///   (i.e., free if indexes < 0)
///
/// Note: The negation of indexes is computed using *ones' complement*
/// instead of *two's complement* in order to distinguish positive-zero
/// and negative-zero.
///
/// # Types of the `next` and `prev` Field
///
/// As described above, struct `HeapCell` has two signed integer
/// fields: the `prev` and `next` fields.  From the practical point of
/// view, `i16` or `i32` are useful as their types.
///
/// * If `i16` is chosen, the size of struct `HeapCell` is 4 bytes,
///   and the maximum managable heap area size is 128KiB (= 4 * 2^15).
///
/// * If `i32` is chosen, the size of struct `HeapCell` is 8 bytes,
///   and the maximum managable heap area size is 16GiB (= 8 * 2^31).
///   (Needless to say, 16GiB space is too huge to manage with a
///   first-fit memory allocator)
///
/// In order to make `MuHeap` independent from the type of index,
/// trait [`MuHeapIndex`] is defined.
///

//
// Because mutable references are not allowed in constant functions,
// the address in usize and the length of the array are recorded in
// struct MuHeap.  Each time when the array is referred, a slice of
// HeapCell's is constructed using method heapcells.
//
pub struct MuHeap<I>
where
    I: MuHeapIndex	// I: Type of Index
{
    base: usize,	// Base Address of the HeapCell Array.
    ncells: I,		// Number of HeapCell's in the HeapCell Array.
    search_start: I,	// Index where the next search starts.
    given_base: usize,	// Given Base Address of Heap Area (for debug)
    given_size: usize,	// Given Size in Bytes of Heap Area (for debug)
    stat: HeapStat,	// Statistics (for debug)
}


#[repr(C)]
struct HeapCell<I>
where
    I: MuHeapIndex	// I: Type of Index
{
    prev: I,		// Index of Previous HeapCell
    next: I,		// Index of Next HeapCell
}

// Enumerations of public methods.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Caller {
    Alloc,		// Method alloc
    Dealloc,		// Method dealloc
    Grow,		// Method grow
    Shrink,		// Method shrink
}


//
// Because non-null pointer must be returned for any successful
// allocation, and zero-sized allocation is allowed in Allocator,
// a very small well-aligned value (i.e., alignment) is returned
// for zero-sized allocation.
//
// Consequently, memory address lower than the maximum alignment
// must not be allocated.  However, there seems to be no simple way
// to know the maximum alignment.  Therefore, MAX_ALIGNMENT is
// defined here to tweak the possible maximum alignment.
//
const MAX_ALIGNMENT: usize = 1024 / 8;	// 1024 bit (for now)


//
// Defines the minimum number of cells in heap area.
// The number below is chosen because at least the 0-th cell must exist.
// (Obviously, it is not a practical number)
//
const MIN_NCELLS: usize = 1;


impl<I> MuHeap<I>
where
    I: MuHeapIndex
{
    // Returns all-zero initializer for a static heap declaration.
    const fn zero() -> Self {
	Self {
	    given_base: 0,
	    given_size: 0,
	    base: 0,
	    ncells: I::ZERO,
	    search_start: I::ZERO,
	    stat: HeapStat::zero(),
	}
    }

    /// Returns a heap initializer with the address and the size in
    /// bytes for a static heap declaration.
    // The remaining fields will be initialized later when method
    // alloc is called at the first time.
    pub const unsafe fn heap(given_base: usize, given_size: usize) -> Self {
	Self {
	    given_base,
	    given_size,
	    ..Self::zero()
	}
    }

    /// Returns a no-heap initializer for a static heap declaration.
    /// The address and the size of a heap area should be set later
    /// by calling method set_heap.
    pub const fn noheap() -> Self {
	Self::zero()
    }

    /// Sets the address and the size in bytes of a heap area
    /// to the statically initialized no-heap area.
    pub unsafe fn set_heap(&mut self, given_base: usize, given_size: usize) {
	debug_assert!(self.given_base == 0 && self.given_size == 0 &&
		      self.base == 0 && self.ncells == I::ZERO);

	self.given_base = given_base;
	self.given_size = given_size;

	self.build_heap();
    }

    /// Attempts to allocate a block of memory.
    pub unsafe fn alloc(&mut self, size: usize, align: usize) -> *mut u8 {
	debug_assert!(self.given_base != 0 && self.given_size != 0);

	if self.base == 0 {
	    // When given_base and given_size are initialized by method heap,
	    // other fields must be initialized here.
	    self.build_heap();
	}

	if DEBUG_HEAP {
	    // Update statistics.
	    self.stat.alloc_calls += 1;
	    if self.stat.largest_size < size {
		self.stat.largest_size = size;
	    }
	    if self.stat.largest_align < align {
		self.stat.largest_align = align;
	    }
	}

	if size == 0 {
	    // For zero-sized allocation,
	    // alignment is returned without allocating memory.
	    align as *mut u8
	} else {
	    // Allocate a new memory area.
	    self.do_alloc(size, align)
	}
    }

    /// Deallocates the memory referenced by ptr.
    pub unsafe fn dealloc(&mut self, ptr: *mut u8, size: usize, align: usize) {
	debug_assert!(self.given_base != 0 && self.given_size != 0 &&
		      self.base != 0 && self.ncells != I::ZERO);

	if DEBUG_HEAP {
	    // Update statistics.
	    self.stat.dealloc_calls += 1;
	}

	if size == 0 {
	    // For zero-sized allocation,
	    // alignment was returned without allocating memory.
	    debug_assert_eq!(ptr as usize, align);
	} else {
	    // Deallocate the memory area.
	    self.do_dealloc(ptr, size, align)
	}
    }

    /// Attempts to extend the memory block.
    pub unsafe fn grow(&mut self, old_ptr: *mut u8,
		       old_size: usize, new_size: usize, align: usize)
		       -> *mut u8 {
	debug_assert!(self.given_base != 0 && self.given_size != 0 &&
		      self.base != 0 && self.ncells != I::ZERO);
	debug_assert!(old_size <= new_size);

	if DEBUG_HEAP {
	    // Update statistics.
	    self.stat.grow_calls += 1;
	    if self.stat.largest_size < new_size {
		self.stat.largest_size = new_size;
	    }
	    if self.stat.largest_align < align {
		self.stat.largest_align = align;
	    }
	}

	if old_size == 0 {
	    // For zero-sized allocation,
	    // alignment was returned without allocating memory.
	    debug_assert_eq!(old_ptr as usize, align);
	    // Allocate a new memory area.
	    self.do_alloc(new_size, align)
	} else {
	    // Grow the memory area.
	    self.do_grow(old_ptr, old_size, new_size, align)
	}
    }

    /// Shrink the memory block.
    pub unsafe fn shrink(&mut self, ptr: *mut u8,
			 old_size: usize, new_size: usize, align: usize)
			 -> *mut u8 {
	debug_assert!(self.given_base != 0 && self.given_size != 0 &&
		      self.base != 0 && self.ncells != I::ZERO);
	debug_assert!(old_size >= new_size);

	if DEBUG_HEAP {
	    // Update statistics.
	    self.stat.shrink_calls += 1;
	}

	if old_size == 0 {
	    // For zero-sized allocation,
	    // alignment was returned without allocating memory.
	    debug_assert_eq!(ptr as usize, align);
	    // Therefore, just return the current ptr.
	    ptr
	} else {
	    // Shrink the memory area.
	    self.do_shrink(ptr, old_size, new_size, align)
	}
    }

    fn do_alloc(&mut self, size: usize, align: usize) -> *mut u8 {
	// Calculate requested number of cells.
	let req_ncells = Self::ncells_up(size);

	let cells = self.heapcells();

	let search_start = self.search_start;
	let mut cur_i = search_start;
	loop {
	    let next_val = cells[cur_i.to_usize()].next;
	    if next_val > I::ZERO {
		// If next_val is positive, those cells between this
		// cell and the next cell are in use.  Skip to the
		// next cell.
		cur_i = next_val;
	    } else if next_val < I::ZERO {
		// If next_val is negative, those cells between this
		// cell and the next cell are free.
		let nxt_i = !next_val;
		let bgn_i = Self::align_cell(cur_i, align);
		let free_ncells = nxt_i - bgn_i - I::ONE;
		if free_ncells >= req_ncells {
		    // Required size of memory can be allocated.
		    let end_i = bgn_i + req_ncells + I::ONE;
		    self.alloc_cells(cells, cur_i, bgn_i, end_i, nxt_i,
				     Caller::Alloc);
		    // Return the allocated address.
		    return self.cell_to_ptr_checked(cells, bgn_i, size, align);
		} else {
		    // Required size of memory cannot be allocated.
		    // Skip to the next cell.
		    cur_i = nxt_i;
		}
	    } else {
		// If next_val is zero, those cells following this
		// cell are free.
		let nxt_i = self.ncells;
		let bgn_i = Self::align_cell(cur_i, align);
		let free_ncells = nxt_i - bgn_i - (I::ONE + I::ONE);
		if free_ncells >= req_ncells {
		    // Required size of memory can be allocated.
		    let end_i = bgn_i + req_ncells + I::ONE;
		    self.alloc_cells(cells, cur_i, bgn_i, end_i, nxt_i,
				     Caller::Alloc);
		    // Return the allocated address.
		    return self.cell_to_ptr_checked(cells, bgn_i, size, align);
		} else {
		    // Required size of memory cannot be allocated.
		    // Skip to the next cell.
		    cur_i = I::ZERO;
		}
	    }

	    if cur_i == search_start {
		// Unfortunately, we get back to the search-start index.
		// It means the required size of memory could not be found.
		// Return null pointer to indicate the failure of allocation.
		return null_mut();
	    }

	    if DEBUG_HEAP {
		if cur_i == I::ZERO {
		    println!("alloc: wrapped cur_i={}, search_start={}, \
			      {:?}, {:?}",
			     cur_i, search_start, self.stat,
			     self.debug_check_list(cur_i, Caller::Alloc));
		}
	    }
	}
    }

    fn do_dealloc(&mut self, ptr: *mut u8, size: usize, align: usize) {
	let cells = self.heapcells();
	let cur_i = self.ptr_to_cell_checked(ptr, size, align);

	// Free cells.
	let nxt_i = cells[cur_i.to_usize()].next;
	self.free_cells(cells, cur_i, nxt_i, Caller::Dealloc);

	if DEBUG_HEAP {
	    if cells[0].next == I::ZERO {
		println!("  heap is now empty! (base={:#x})", self.base);
	    }
	}
    }

    fn do_grow(&mut self, old_ptr: *mut u8,
	       old_size: usize, new_size: usize, align: usize) -> *mut u8 {
	let cells = self.heapcells();
	let cur_i = self.ptr_to_cell_checked(old_ptr, old_size, align);

	let nxt_i = cells[cur_i.to_usize()].next;

	let far_val = cells[nxt_i.to_usize()].next;
	if far_val <= I::ZERO {
	    let far_i = if far_val < I::ZERO { !far_val } else { I::ZERO };
	    let req_ncells = Self::ncells_up(new_size);
	    let end_i = cur_i + req_ncells + I::ONE;
	    if end_i <= far_i {
		self.alloc_cells(cells, cur_i, cur_i, end_i, far_i,
				 Caller::Grow);
		return self.ptr_checked(old_ptr, cur_i, new_size, align);
	    }
	}

	let new_ptr = self.do_alloc(new_size, align);
	if !new_ptr.is_null() {
	    unsafe {
		copy_nonoverlapping::<u8>(old_ptr, new_ptr, old_size);
	    }
	}
	self.do_dealloc(old_ptr, old_size, align);

	new_ptr
    }

    fn do_shrink(&mut self, ptr: *mut u8,
		 old_size: usize, new_size: usize, align: usize) -> *mut u8 {
	let cells = self.heapcells();
	let cur_i = self.ptr_to_cell_checked(ptr, old_size, align);

	let req_ncells = Self::ncells_up(new_size);
	let end_i = cur_i + req_ncells + I::ONE;
	let nxt_i = cells[cur_i.to_usize()].next;
	if end_i < nxt_i {
	    self.alloc_cells(cells, cur_i, cur_i, end_i, nxt_i,
			     Caller::Shrink);
	    self.free_cells(cells, end_i, nxt_i, Caller::Shrink);
	}

	self.ptr_checked(ptr, cur_i, new_size, align)
    }

    fn alloc_cells(&mut self, cells: &mut [HeapCell<I>],
		   cur_i: I, bgn_i: I, end_i: I, nxt_i: I, caller: Caller) {
	if DEBUG_HEAP {
	    assert!(cur_i >= I::ZERO &&
		    cur_i <= bgn_i && bgn_i < end_i && end_i <= nxt_i);
	    if DEBUG_PRIOR_CHECK {
		self.debug_check_list(cur_i, caller);
	    }
	}

	// Free cells
	if cur_i < bgn_i {
	    cells[cur_i.to_usize()].next = !bgn_i;
	    cells[bgn_i.to_usize()].prev = !cur_i;
	}

	// Allocated cells
	cells[bgn_i.to_usize()].next = end_i;
	cells[end_i.to_usize()].prev = bgn_i;

	// Free cells
	if end_i < nxt_i {
	    if nxt_i < self.ncells {
		cells[end_i.to_usize()].next = !nxt_i;
		cells[nxt_i.to_usize()].prev = !end_i;
	    } else {
		cells[end_i.to_usize()].next = I::ZERO;
	    }
	}

	// Update search-start index if it points to an inside cell.
	#[allow(unused_parens)]
	if (caller == Caller::Alloc ||
	    (self.search_start >= cur_i && self.search_start <= nxt_i)) {
	    self.search_start = end_i;
	}

	if DEBUG_HEAP {
	    if caller == Caller::Alloc {
		self.stat.inuse_count += 1;
	    }
	    if DEBUG_POST_CHECK && caller != Caller::Shrink {
		self.debug_check_list(cur_i, caller);
	    }
	}
    }

    fn free_cells(&mut self, cells: &mut [HeapCell<I>], cur_i: I, nxt_i: I,
		  caller: Caller) {
	if DEBUG_HEAP {
	    if DEBUG_PRIOR_CHECK && caller != Caller::Shrink {
		self.debug_check_list(cur_i, caller);
	    }
	}

	// Find the head of preceding free cells.
	let mut prev = cur_i;
	while prev > I::ZERO && cells[prev.to_usize()].prev < I::ZERO {
	    prev = !cells[cur_i.to_usize()].prev;
	}

	// Find the tail of succeeding free cells.
	let mut next = nxt_i;
	while next > I::ZERO && cells[next.to_usize()].next < I::ZERO {
	    next = !cells[next.to_usize()].next;
	}
	if cells[next.to_usize()].next == I::ZERO {
	    next = I::ZERO;
	}

	// Update the head of the merged free cells.
	if prev != next {
	    if next == I::ZERO {
		cells[prev.to_usize()].next = I::ZERO;
	    } else {
		cells[prev.to_usize()].next = !next;
	    }
	} else if prev == I::ZERO { // && next == I::ZERO
	    cells[0].next = I::ZERO;
	}

	// Update the tail of the merged free cells.
	// If next == I::ZERO, prev is the final cell.
	if next > I::ZERO {
	    cells[next.to_usize()].prev = !prev;
	}

	// Update search-start index if it points to a cell
	// in the merged free area.
	#[allow(unused_parens)]
	if (self.search_start > prev &&
	    (next == I::ZERO || self.search_start <= next)) {
	    self.search_start = prev;
	}

	if DEBUG_HEAP {
	    if caller == Caller::Dealloc {
		self.stat.inuse_count -= 1;
	    }
	    if DEBUG_POST_CHECK {
		self.debug_check_list(prev, caller);
	    }
	    if DEBUG_FILL_JUNK {
		self.debug_fill_junk(prev, next);
	    }
	}
    }

    fn build_heap(&mut self) {
	let (adj_base, adj_ncells) = Self::adjust_heap(self.given_base,
						       self.given_size);

	// Initialize self.
	self.base = adj_base;
	self.ncells = adj_ncells;

	// Initialize the 0-th cell.
	let cells = self.heapcells();
	cells[0].prev = I::ZERO;
	cells[0].next = I::ZERO;

	if DEBUG_HEAP {
	    if DEBUG_FILL_JUNK {
		self.debug_fill_junk(I::ONE, self.ncells);
	    }
	}
    }

    fn adjust_heap(given_base: usize, given_size: usize) -> (usize, I) {
	// Calculate the minimum allocatable address, then
	// calculate the minimum base address.
	let min_addr = Self::round_up(MAX_ALIGNMENT + 1,
				      Self::heapcell_size());
	let min_base = min_addr - Self::heapcell_size();

	// Adjust the base and the size allocatable.
	let (mut adj_base, mut adj_size) = (given_base, given_size);
	if given_base < min_base {
	    let adjust = min_base - given_base;
	    if DEBUG_HEAP {
		assert!(given_size > adjust,
			"Given heap is too small: \
			 given_size={:#x}, adjust={:#x}",
			given_size, adjust);
	    }
	    (adj_base, adj_size) = (given_base + adjust, given_size - adjust);
	}

	// Adjust the number of usable cells.
	let adj_ncells = Self::ncells_down(adj_size);

	// Check the number of usable cells.
	adj_size = adj_ncells.to_usize() * Self::heapcell_size();
	assert!(adj_ncells >= I::from_usize(MIN_NCELLS),
		"Given heap is too small: \
		 given=({:#x}, {:#x}), adjusted=({:#x}, {:#x} (#{:#x}))",
		given_base, given_size, adj_base, adj_size, adj_ncells);

	if DEBUG_HEAP {
	    println!("given_heap=({:#x}, {:#x}), \
		      usable_heap=({:#x}, {:#x} (#{:#x}))",
		     given_base, given_size,
		     adj_base, adj_size, adj_ncells);
	}

	(adj_base, adj_ncells)
    }

    fn heapcells<'a, 'b>(&'a self) -> &'b mut [HeapCell<I>] {
	unsafe {
	    slice::from_raw_parts_mut(self.base as *mut HeapCell<I>,
				      self.ncells.to_usize())
	}
    }

    fn cell_to_ptr(cells: &mut [HeapCell<I>], cur_i: I) -> *mut u8 {
	&mut cells[(cur_i + I::ONE).to_usize()] as *mut HeapCell<I> as *mut u8
    }

    fn ptr_to_cell(&self, ptr: *mut u8) -> I {
	// Calculate the offset from the base, then
	let off = (ptr as usize) - self.base;

	// Calculate the index of manager cell.
	Self::ncells_down(off) - I::ONE
    }

    fn cell_to_ptr_checked(&self, cells: &mut [HeapCell<I>], cur_i: I,
			   size: usize, align: usize) -> *mut u8 {
	let ptr = Self::cell_to_ptr(cells, cur_i);

	if DEBUG_HEAP && DEBUG_CHECK_PTR {
	    self.debug_check_ptr(ptr, cur_i, size, align);
	}

	ptr
    }

    fn ptr_to_cell_checked(&self, ptr: *mut u8,
			   size: usize, align: usize) -> I {
	let cur_i = self.ptr_to_cell(ptr);

	if DEBUG_HEAP && DEBUG_CHECK_PTR {
	    self.debug_check_ptr(ptr, cur_i, size, align);
	}

	cur_i
    }

    fn ptr_checked(&self, ptr: *mut u8, cur_i: I,
		   size: usize, align: usize) -> *mut u8 {
	if DEBUG_HEAP && DEBUG_CHECK_PTR {
	    self.debug_check_ptr(ptr, cur_i, size, align);
	}

	ptr
    }

    #[inline]
    const fn heapcell_size() -> usize {
	size_of::<HeapCell<I>>()
    }

    #[inline]
    fn ncells_up(n: usize) -> I {
	fn nelem_up(n: usize, m: usize) -> usize {
	    (n + m - 1) / m
	}
	let r = nelem_up(n, Self::heapcell_size());
	I::from_usize(min(r, I::MAX_USIZE))
    }

    #[inline]
    fn ncells_down(n: usize) -> I {
	fn nelem_down(n: usize, m: usize) -> usize {
	    n / m
	}
	let r = nelem_down(n, Self::heapcell_size());
	I::from_usize(min(r, I::MAX_USIZE))
    }

    #[inline]
    fn align_cell(cur_i: I, align: usize) -> I {
	let cur_mem_i = cur_i + I::ONE;
	let cur_mem_off = cur_mem_i.to_usize() * Self::heapcell_size();
	let ali_mem_off = Self::round_up(cur_mem_off, align);
	let ali_mem_i = I::from_usize(ali_mem_off / Self::heapcell_size());
	ali_mem_i - I::ONE
    }

    #[inline]
    const fn round_up(n: usize, m: usize) -> usize {
	((n + m - 1) / m) * m
    }
}

impl<I> MuHeap<I>
where
    I: MuHeapIndex
{
    fn debug_check_list(&self, check_index: I, _caller: Caller)
			-> HeapFigures<I> {
	let cells = self.heapcells();
	let search_start = self.search_start;
	let mut search_start_found = false;
	let mut check_index_found = false;
	let mut figures = HeapFigures::<I>::zero();

	let mut cur_i = I::ZERO;
	loop {
	    if cur_i == search_start {
		search_start_found = true;
	    }
	    if cur_i == check_index {
		check_index_found = true;
	    }
	    let next_val = cells[cur_i.to_usize()].next;
	    let nxt_i;
	    if next_val > I::ZERO {
		nxt_i = next_val;
		let cur_ncells = nxt_i - cur_i - I::ONE;
		figures.add_inuse(cur_ncells);
		assert!(nxt_i < self.ncells);
		assert_eq!(cells[nxt_i.to_usize()].prev, cur_i);
		assert!(cur_ncells > I::ZERO);
	    } else if next_val < I::ZERO {
		nxt_i = !next_val;
		let cur_ncells = nxt_i - cur_i - I::ONE;
		figures.add_free(cur_ncells);
		assert!(nxt_i < self.ncells);
		assert_eq!(cells[nxt_i.to_usize()].prev, !cur_i);
		assert!(cells[nxt_i.to_usize()].next >= I::ZERO);
	    } else { // next_val == I::ZERO
		let cur_ncells = self.ncells - cur_i - I::ONE;
		figures.add_free(cur_ncells);
		assert!(cells[cur_i.to_usize()].prev >= I::ZERO);
		break;
	    }
	    cur_i = nxt_i;
	}

	assert!(search_start_found && check_index_found);
	assert_eq!(figures.inuse_count.to_usize(), self.stat.inuse_count);
	assert_eq!(cells[0].prev, I::ZERO);

	figures
    }

    fn debug_check_ptr(&self, ptr: *mut u8, cur_i: I,
		       size: usize, align: usize) {
	let cells = self.heapcells();

	{
	    let off = (ptr as usize) - self.base;
	    assert!((ptr as usize) > self.base,
		    "ptr={:p} is out of range", ptr);

	    let index = Self::ncells_down(off).to_usize();
	    assert!(index * Self::heapcell_size() == off,
		    "ptr={:p} is invalid", ptr);
	    assert!(index > 0 && index < self.ncells.to_usize(),
		    "ptr={:p} is out of range", ptr);
	}

	assert!(cur_i >= I::ZERO && cur_i < self.ncells);
	let nxt_i = cells[cur_i.to_usize()].next;
	assert!(nxt_i > I::ZERO && nxt_i < self.ncells);
	assert!(cells[nxt_i.to_usize()].prev == cur_i);

	let req_ncells = Self::ncells_up(size);
	let cur_ncells = nxt_i - cur_i - I::ONE;
	assert_eq!(req_ncells, cur_ncells);

	let mem_addr = Self::cell_to_ptr(cells, cur_i) as usize;
	let aligned_addr = Self::round_up(mem_addr, align);
	assert_eq!(mem_addr, aligned_addr);
    }

    fn debug_fill_junk(&self, cur_i: I, nxt_i: I) {
	if cur_i < nxt_i {
	    let cells = self.heapcells();
	    let slice =
		unsafe {
		    let ncells = nxt_i - cur_i - I::ONE;
		    let nbytes = ncells.to_usize() * Self::heapcell_size();
		    let ptr = Self::cell_to_ptr(cells, cur_i);
		    slice::from_raw_parts_mut::<u8>(ptr, nbytes)
		};
	    slice.fill(0x5a);
	}
    }
}


#[derive(Debug)]
struct HeapStat
{
    alloc_calls: usize,
    dealloc_calls: usize,
    grow_calls: usize,
    shrink_calls: usize,
    inuse_count: usize,
    largest_size: usize,
    largest_align: usize,
}

impl HeapStat {
    const fn zero() -> Self {
	Self {
	    alloc_calls: 0,
	    dealloc_calls: 0,
	    grow_calls: 0,
	    shrink_calls: 0,
	    inuse_count: 0,
	    largest_size: 0,
	    largest_align: 0,
	}
    }
}


#[derive(Debug)]
struct HeapFigures<I>
where
    I: MuHeapIndex
{
    inuse_count: I,
    inuse_ncells: I,
    free_count: I,
    free_ncells: I,
    free_largest: I,
}

impl<I> HeapFigures<I>
where
    I: MuHeapIndex
{
    fn zero() -> Self {
	Self {
	    inuse_count: I::ZERO,
	    inuse_ncells: I::ZERO,
	    free_count: I::ZERO,
	    free_ncells: I::ZERO,
	    free_largest: I::ZERO,
	}
    }

    fn add_inuse(&mut self, ncells: I) {
	self.inuse_count += I::ONE;
	self.inuse_ncells += ncells;
    }

    fn add_free(&mut self, ncells: I) {
	if ncells > I::ZERO {
	    self.free_count += I::ONE;
	    self.free_ncells += ncells;
	    if self.free_largest < ncells {
		self.free_largest = ncells;
	    }
	}
    }
}


/// A trait that the types of indexes in heap cells must satisfy.
///
/// From the practical point of view, `i16` or `i32` are useful.
pub trait MuHeapIndex
where
    Self: 'static + Copy + PartialOrd
    + fmt::Debug + fmt::Display + fmt::LowerHex
    + ops::Add<Output = Self> + ops::Sub<Output = Self>
    + ops::AddAssign + ops::SubAssign
    + ops::Neg<Output = Self> + ops::Not<Output = Self>,
{
    /// Zero in Self.
    const ZERO: Self;
    /// One in Self.
    const ONE: Self;
    /// The maximum value in usize.
    const MAX_USIZE: usize;
    /// Converts a value from usize into Self.
    fn from_usize(n: usize) -> Self;
    /// Converts a value from Self into usize.
    fn to_usize(&self) -> usize;
}

impl MuHeapIndex for i16 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX_USIZE: usize = Self::MAX as usize;

    #[inline]
    fn from_usize(n: usize) -> Self {
	n as Self
    }

    #[inline]
    fn to_usize(&self) -> usize {
	*self as usize
    }
}

impl MuHeapIndex for i32 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX_USIZE: usize = Self::MAX as usize;

    #[inline]
    fn from_usize(n: usize) -> Self {
	n as Self
    }

    #[inline]
    fn to_usize(&self) -> usize {
	*self as usize
    }
}
