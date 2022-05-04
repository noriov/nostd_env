/*!

BIOS INT 10h AX=4F03h : Return Current VBE Mode

# Resource

* [VESA BIOS Extension Core Function Standard Version 3.0](http://www.petesqbsite.com/sections/tutorials/tuts/vbe3.pdf) (VESA, 1998-09-16)

# Supplementary Resources

* [VESA Video Modes](https://wiki.osdev.org/VESA_Video_Modes) (OS Dev)
* [Display Industry Standards Archive](https://glenwing.github.io/docs/) (Glen Wing)

 */

//
// BIOS INT 10h AX=4F03h (Return Current VBE Mode)
//
// Resource:
//	"VESA BIOS Extension Core Function Standard Version 3.0" (1998-09-16)
//	http://www.petesqbsite.com/sections/tutorials/tuts/vbe3.pdf
//
// Supplementary Resources:
//	https://wiki.osdev.org/VESA_Video_Modes
//
//	"Display Industry Standards Archive"
//	https://glenwing.github.io/docs/
//

use super::LmbiosRegs;
use crate::println;


#[doc(hidden)]
const DEBUG: bool = false;


/// Calls BIOS INT 10h AX=4F03h (Return Current VBE Mode).
pub fn call() -> u16
{
    unsafe {
	// INT 10h AH=4Fh AL=03h
	// OUT
	//   AX    = Status
	//   BX    = Current VBE mode
	let mut regs = LmbiosRegs {
	    fun: 0x10,		// INT 10h
	    eax: 0x4f03,	// AH=4Fh AL=03h
	    ..Default::default()
	};

	if DEBUG {
	    println!("IN:  EAX={:#x}",
		     regs.eax);
	}

	regs.call();

	if DEBUG {
	    println!("OUT: EAX={:#x}, EBX={:#x}",
		     regs.eax, regs.ebx);
	}

	regs.ebx as u16
    }
}
