//
// BIOS INT 10h AH=0Eh (Teletype Output)
//
// Supplementary Resource:
//	https://en.wikipedia.org/wiki/INT_10H
//

use super::LmbiosRegs;


pub struct Int10h0eh;

impl Int10h0eh {
    pub fn call(byte: u8, page_number: u8, color: u8) {
	unsafe {
	    // INT 10h AH=0Eh (Teletype Output)
	    // IN
	    //   AL = Character
	    //   BH = Page Number
	    //   BL = Color
	    LmbiosRegs {
		fun: 0x10,
		eax: 0x0E00 | byte as u32,
		ebx: (page_number as u32) << 8 | (color as u32),
		..Default::default()
	    }.call();
	}
    }
}
