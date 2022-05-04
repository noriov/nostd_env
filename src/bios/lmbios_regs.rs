use core::ops::Deref;
use core::mem::size_of;

use super::ffi;
use crate::mu::MuMutex;


//
// Function numbers
//	  0x00 -   0xFF : Software Interrupt Number (INT n)
//	 0x100 -  0x3FF : (reserved)
//	 0x400 - 0xFFFE : Subroutine address
//	0xFFFF          : (unsupported)
//

/// Calls BIOS functions.
#[repr(C)]
#[derive(Default)]
pub struct LmbiosRegs {	// Offset:
    pub fun: u16,	// 00-01 : Function number	(IN)
    pub flags: u16,	// 02-03 : FLAGS		(OUT)
    pub eax: u32,	// 04-07 : EAX			(IN/OUT)
    pub ebx: u32,	// 08-0B : EBX			(IN/OUT)
    pub ecx: u32,	// 0C-0F : ECX			(IN/OUT)
    pub edx: u32,	// 10-13 : EDX			(IN/OUT)
    pub esi: u32,	// 14-17 : ESI			(IN/OUT)
    pub edi: u32,	// 18-1B : EDI			(IN/OUT)
    pub ebp: u32,	// 1C-1F : EBP			(IN/OUT)
    pub ds: u16,	// 20-21 : DS			(IN/OUT)
    pub es: u16,	// 22-23 : ES			(IN/OUT)
}

const _: () = assert!(size_of::<LmbiosRegs>() == 0x24);


impl LmbiosRegs {
    pub unsafe fn call(&mut self) -> u16 {
	let _guard = BIOS_TICKET.lock();
	ffi::lmbios_call(self)
    }
}


struct LmbiosTicket;

struct LmbiosMutex {
    ticket: MuMutex<LmbiosTicket>,
}

impl LmbiosMutex {
    pub const fn ticket() -> Self {
	Self {
	    ticket: MuMutex::new(LmbiosTicket),
	}
    }
}

impl Deref for LmbiosMutex
{
    type Target = MuMutex<LmbiosTicket>;
    fn deref(&self) -> &MuMutex<LmbiosTicket> {
	&self.ticket
    }
}

static BIOS_TICKET: LmbiosMutex = LmbiosMutex::ticket();
