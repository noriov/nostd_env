/*!

Tests a memory allocator and a heap manager.

 */


use alloc::collections::VecDeque;
use alloc::vec::Vec;
use core::alloc::Allocator;

use crate::{print, println};


///
/// Tests a memory allocator and a heap manager by repeating
/// `Sieve of Eratosthenes` implemented with Vec and VecDeque.
///
pub fn try_sieve<A>(n: usize, m: usize, count: usize, alloc: A)
where
    A: Copy + Allocator
{
    // Create an empty vector that will hold resulting sieves.
    let mut results_queue = VecDeque::new_in(alloc);

    print!("Running: ");
    for i in 0 .. count {
	if (i % 10) == 0 {
	    print!("{},", i);
	}

	if i > m {
	    // Remove the first element if the queue is too large.
	    results_queue.pop_front();	// method dealloc is called
	}

	// Execute `Sieve of Eratosthenes`
	let sieved_vec = {
	    // Create a list of consecutive integers from 0 through n - 1.
	    // Each element is a non-empty vector.
	    let mut sieve_vec = Vec::new_in(alloc);
	    for i in 0 .. n {
		// Method alloc is called to create a vector.
		let mut inner_vec = Vec::with_capacity_in(1, alloc);
		for _j in 0 .. i * 3 + 1 {
		    inner_vec.push(0_usize);	// Method grow may be called
		}
		inner_vec.truncate(i * 2 + 1);
		inner_vec.shrink_to_fit();	// Method shrink is called
		sieve_vec.push(inner_vec);
	    }
	    sieve_vec.shrink_to(n);	// Method shrink is called

	    // Empty 0 and 1 because they are not prime numbers.
	    for i in 0 .. 2 {
		// method dealloc is called to clear an existing vector.
		sieve_vec[i] = Vec::new_in(alloc);
	    }

	    // Empty numbers that are not prime numbers.
	    for i in 2 .. n {
		if !sieve_vec[i].is_empty() {
		    let mut j = i * 2;
		    while j < n {
			// Method dealloc is called to clear an existing vector
			sieve_vec[j] = Vec::new_in(alloc);
			j += i;
		    }
		}
	    }

	    // Return the resulting vec.
	    sieve_vec
	};

	// Save the resulting sieve.
	// Method grow may be called for the first m times.
	results_queue.push_back(sieved_vec);
    }

    println!();
}
