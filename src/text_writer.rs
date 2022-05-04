/*!

Provides a text writer using BIOS.

TextWriter - A Text Writer using BIOS INT 10h AH=0Eh (Teletype Output)

 */


use core::fmt;

use crate::bios;


pub struct TextWriter;

impl TextWriter {
    pub fn write_ascii_printables(&mut self, utf8_str: &str) {
	for byte in utf8_str.bytes() {
	    let ch =
		match byte {
		    0x20 ..= 0x7E | b'\n' | b'\r' => byte,
		    _ => b'.'
		};
	    let page_number = 0;
	    let color = 15; // White
	    bios::int10h0eh::call(ch, page_number, color);
	}
    }
}

impl fmt::Write for TextWriter {
    fn write_str(&mut self, utf8_str: &str) -> fmt::Result {
	self.write_ascii_printables(utf8_str);
	Ok(())
    }
}


/// Prints to the console with a newline.
#[macro_export]
macro_rules! println {
    () => {
	$crate::print!("\r\n")
    };
    ( $($arg:tt)* ) => {
	$crate::print!("{}\r\n", format_args!( $($arg)* ))
    };
}

/// Prints to the console.
#[macro_export]
macro_rules! print {
    ( $($arg:tt)* ) => {
	$crate::text_writer::_text_print(format_args!( $($arg)* ))
    };
}

pub fn _text_print(args: fmt::Arguments) {
    use fmt::Write;
    let mut text_writer = TextWriter;
    text_writer.write_fmt(args).unwrap();
}
