/*!

BIOS INT 10h AH=0Eh : Teletype Output

# Supplementary Resource

* <https://en.wikipedia.org/wiki/INT_10H>

 */

//
// Supplementary Resource:
//	https://en.wikipedia.org/wiki/INT_10H
//

use super::LmbiosRegs;


/// Calls BIOS INT 10h AH=0Eh (Teletype Output).
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
