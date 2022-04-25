//
// TextWriter - A Text Writer using BIOS INT 10h AH=0Eh (Teletype Output)
//

use core::fmt;

use crate::bios;


pub struct TextWriter;

impl TextWriter {
    pub fn write_byte(&mut self, byte: u8) {
	unsafe {
	    // INT 10h AH=0Eh (Teletype Output)
	    // AL: Character, BH: Page Number, BL: Color
	    // cf. https://en.wikipedia.org/wiki/INT_10H
	    bios::LmbiosRegs {
		fun: 0x10,			// INT 10h AH=0Eh
		eax: 0x0E00 | byte as u32,	// AL: Character = `byte`
		ebx: 0x000F,			// BL: Color = 15 (white)
		..Default::default()
	    }.call();
	}
    }

    pub fn write_ascii_printables(&mut self, utf8_str: &str) {
	for byte in utf8_str.bytes() {
	    self.write_byte(
		match byte {
		    0x20 ..= 0x7E | b'\n' | b'\r' => byte,
		    _ => b'.'
		});
	}
    }
}

impl fmt::Write for TextWriter {
    fn write_str(&mut self, utf8_str: &str) -> fmt::Result {
	self.write_ascii_printables(utf8_str);
	Ok(())
    }
}


#[macro_export]
macro_rules! println {
    () => {
	$crate::print!("\r\n")
    };
    ( $($arg:tt)* ) => {
	$crate::print!("{}\r\n", format_args!( $($arg)* ))
    };
}

#[macro_export]
macro_rules! print {
    ( $($arg:tt)* ) => {
	$crate::text_writer::_text_print(format_args!( $($arg)* ))
    };
}

pub fn _text_print(args: fmt::Arguments) {
    use core::fmt::Write;
    let mut text_writer = TextWriter;
    text_writer.write_fmt(args).unwrap();
}
