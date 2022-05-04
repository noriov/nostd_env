use core::arch::asm;
use crate::println;

/// Halt forever.
pub fn halt_forever() -> ! {
    println!("halt");
    loop {
	unsafe {
	    asm!("hlt");
	}
    }
}
