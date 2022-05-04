/*!

BIOS INT 15h AX=E820h : Query System Address Map

# Supplementary Resources

* [Detecting Memory (x86)](https://wiki.osdev.org/Detecting_Memory_%28x86%29) (OS Dev)
* [INT 15H, E820H - Query System Address Map](https://uefi.org/specs/ACPI/6.4/15_System_Address_Map_Interfaces/int-15h-e820h---query-system-address-map.html) (UEFI)
* [INT 15h, AX=E820h - Query System Address Map](http://www.uruk.org/orig-grub/mem64mb.html)

 */

//
// Supplementary Resources:
//	https://wiki.osdev.org/Detecting_Memory_(x86)
//	https://uefi.org/specs/ACPI/6.4/15_System_Address_Map_Interfaces/int-15h-e820h---query-system-address-map.html
//	http://www.uruk.org/orig-grub/mem64mb.html
//

use alloc::vec::Vec;
use core::alloc::Allocator;
use core::mem::{MaybeUninit, size_of};

use super::LmbiosRegs;
use crate::println;
use crate::mu::PushBulk;
use crate::x86::{FLAGS_CF, X86GetAddr};


#[doc(hidden)]
const DEBUG: bool = false;


/// Calls BIOS INT 15h AX=E820h (Query System Address Map).
pub fn call<A20>(alloc20: A20) -> Option<Vec<AddrRange, A20>>
where
    A20: Allocator,
{
    // Initialize parameters.
    const ADDR_RANGE_SIZE: u32 = size_of::<AddrRange>() as u32;
    const SMAP_SIGNATURE:u32 = 0x534D4150;  // "SMAP"

    // Prepare a result buffer in 20-bit address space.
    let mut vec = Vec::new_in(alloc20);

    // Initialize the continuation value to zero.
    let mut continuation: u32 = 0;

    loop {
	unsafe {
	    vec.push_bulk(1, | buf | {
		// Fill the AddrRange buffer with the initial value.
		buf[0] = AddrRange::initial_value();

		// Get the far pointer of the buffer.
		let buf_fp = buf.get_far_ptr().ok_or(())?;

		// INT 15h AH=E8h AL=20h (Query System Address Map)
		// IN
		//   EAX   = E820h
		//   EBX   = Continuation
		//   ECX   = Buffer Size
		//   EDX   = Signature "SMAP"
		//   ES:DI = Buffer Address
		// OUT
		//   EAX   = Signature "SMAP"
		//   EBX   = Continuation
		//   ECX   = Buffer Size
		//   CF    = 0 if Ok, 1 if Err
		let mut regs = LmbiosRegs {
		    fun: 0x15,			// INT 15h
		    eax: 0xe820,		// AH=E8 AL=20h
		    ebx: continuation,		// Continuation
		    ecx: ADDR_RANGE_SIZE,	// Buffer Size
		    edx: SMAP_SIGNATURE,	// Signature "SMAP"
		    edi: buf_fp.offset as u32,	// Buffer Address
		    es: buf_fp.segment,		// Buffer Address
		    ..Default::default()
		};

		if DEBUG {
		    println!("IN:  EAX={:#x}, EBX={:#x}, ECX={:#x}, \
			      EDX={:#x}, ES:EDI={:#x}:{:#x}",
			     regs.eax, regs.ebx, regs.ecx,
			     regs.edx, regs.es, regs.edi);
		}

		regs.call();

		if DEBUG {
		    println!("OUT: EAX={:#x}, EBX={:#x}, ECX={:#x}, \
			      EDX={:#x}, ES:EDI={:#x}:{:#x}, FLAGS={:#x}",
			     regs.eax, regs.ebx, regs.ecx,
			     regs.edx, regs.es, regs.edi, regs.flags);
		}

		// Check the result.
		if regs.eax != SMAP_SIGNATURE || (regs.flags & FLAGS_CF) != 0 {
		    return Err(());
		}

		// Save the continuation value.
		continuation = regs.ebx;

		Ok(())
	    }).ok()?;
	}

	// If the continuation value is zero, this is the last entry.
	if continuation == 0 {
	    break;
	}
    }

    vec.shrink_to_fit();

    if DEBUG {
	println!("System Address Map:");
	for addr_range in &vec {
	    addr_range.print();
	}
    }

    Some(vec)
}


/// Address Range Descriptor
#[repr(C)]
#[derive(Clone, Copy)]
pub struct AddrRange {
    pub addr: u64,	//00-07: Base Address
    pub length: u64,	//08-0F: Length in Bytes
    pub atype: u32,	//10-13: Address Type
    pub attr: u32,	//14-17: Extended Attributes (ACPI 3.0)
}

const _: () = assert!(size_of::<AddrRange>() == 0x18);

impl AddrRange {
    // Address Range Types
    pub const TYPE_USABLE	: u32 = 1;
    pub const TYPE_RESERVED	: u32 = 2;
    pub const TYPE_ACPI		: u32 = 3;
    pub const TYPE_NVS		: u32 = 4;
    pub const TYPE_UNUSABLE	: u32 = 5;
    pub const TYPE_DISABLED	: u32 = 6;
    pub const TYPE_PERSISTENT	: u32 = 7;

    // Extended Attributes
    pub const ATTR_DEFAULT	: u32 = 1;  // Bit 0 must be set to 1.

    fn uninit() -> Self {
	unsafe {
	    MaybeUninit::<Self>::uninit().assume_init()
	}
    }

    fn initial_value() -> Self {
	Self {
	    attr: Self::ATTR_DEFAULT,  // for compatibility with ACPI 3.0 entry
	    ..Self::uninit()
	}
    }

    pub fn print(&self) {
	let type_name =
	    match self.atype {
		Self::TYPE_USABLE	=> "Usable",
		Self::TYPE_RESERVED	=> "Reserved",
		Self::TYPE_ACPI		=> "ACPI Reclaimable",
		Self::TYPE_NVS		=> "ACPI Non-Volatile Storage",
		Self::TYPE_UNUSABLE	=> "Containing Bad Memory",
		Self::TYPE_DISABLED	=> "Disabled",
		Self::TYPE_PERSISTENT	=> "Persistent Memory",
		_ => "unknown",
	    };

	println!("addr={:#x}, length={:#x}, type={} ({}), attr={:#x}",
		 self.addr, self.length, self.atype, type_name, self.attr);
    }
}

impl X86GetAddr for AddrRange {}
