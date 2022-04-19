use core::mem::size_of;

extern "C" {
    pub fn lmbios_call(regs: &mut LmbiosRegs) -> u16;
    pub fn lmbios_get_boot_drive_id() -> u8;
}


//
// Function numbers
//	  0x00 -   0xFF : Software Interrupt Number (INT n)
//	 0x100 -  0x3FF : (reserved)
//	 0x400 - 0xFFFE : Subroutine address
//	0xFFFF          : (unsupported)
//

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
	lmbios_call(self)
    }
}
