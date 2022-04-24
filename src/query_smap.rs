//
// Query System Address Map using INT 15h AX=E820h
//

use alloc::vec::Vec;
use core::alloc::Allocator;
use core::mem::size_of;

use crate::bios;
use crate::println;


#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct AddrRange {
    pub addr: u64,	//00-07: Base Address
    pub length: u64,	//08-0F: Length in Bytes
    pub atype: u32,	//10-13: Address Type
    pub attr: u32,	//14-17: Extended Attributes (ACPI 3.0)
}

const _: () = assert!(size_of::<AddrRange>() == 0x18);

const ADDR_RANGE_SIZE: u32 = size_of::<AddrRange>() as u32;
const SMAP_SIGNATURE: u32 = 0x534D4150; // "SMAP"

pub const ATYPE_USABLE: u32 = 1;
pub const ATTR_DEFAULT: u32 = 1; // for compatibility with ACPI 3.0 entry

// pub const ATYPE_RESERVED: u32 = 2;
// pub const ATYPE_ACPI_RECLAIMABLE: u32 = 3;
// pub const ATYPE_ACPI_NVS: u32 = 4;
// pub const ATYPE_BAD_MEMORY: u32 = 5;


pub fn query_smap<A16, A20>(alloc16: A16, alloc20: A20) -> Vec<AddrRange, A20>
where
    A16: Copy + Allocator,
    A20: Copy + Allocator,
{
    // Allocate a buffer for AddrRange in 16-bit address space.
    let mut buf = Vec::with_capacity_in(1, alloc16);
    buf.resize(1, AddrRange::default());
    let buf_addr = buf.as_ptr() as usize;
    assert!(buf_addr < (1_usize << 16));

    // Allocate a result vector in 20-bit address space.
    let mut addr_ranges = Vec::new_in(alloc20);

    // Initialize the continuation value to zero.
    let mut continuation: u32 = 0;

    loop {
	buf[0] = AddrRange {
	    attr: ATTR_DEFAULT,
	    ..AddrRange::default()
	};

	unsafe {
	    // INT 15h AH=e8h AL=20h (Query System Address Map)
	    let mut regs = bios::LmbiosRegs {
		fun: 0x15,			// INT 15h
		eax: 0xe820,			// AH=E8 AL=20h
		ebx: continuation,		// Continuation
		ecx: ADDR_RANGE_SIZE,		// Buffer Size
		edx: SMAP_SIGNATURE,		// Signature "SMAP"
		edi: buf_addr as u32,		// Buffer Address
		..Default::default()
	    };
	    regs.call();

	    // Check whether an error is detected.
	    // Note: If the carry flag (CF) is set, an error is detected.
	    const FLAGS_CF: u16 = 0x0001;
	    if (regs.flags & FLAGS_CF) != 0 {
		break;
	    }

	    // Save the result.
	    addr_ranges.push(buf[0]);

	    continuation = regs.ebx;
	    if continuation == 0 {
		// This is the last entry.
		break;
	    }
	}
    }

    if false {
	println!("System Address Map:");
	for addr_range in &addr_ranges {
	    println!("addr={:#x}, length={:#x}, atype={:#x}, attr={:#x}",
		     addr_range.addr,
		     addr_range.length,
		     addr_range.atype,
		     addr_range.attr);
	}
    }

    addr_ranges
}


//
// Supplementary Resources
//	https://wiki.osdev.org/Detecting_Memory_(x86)
//	https://uefi.org/specs/ACPI/6.4/15_System_Address_Map_Interfaces/int-15h-e820h---query-system-address-map.html
//	http://www.uruk.org/orig-grub/mem64mb.html
//
