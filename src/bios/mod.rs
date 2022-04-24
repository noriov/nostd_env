pub mod ffi;

use core::arch::global_asm;

global_asm!(include_str!("lmboot0.s"), options(att_syntax));
global_asm!(include_str!("lmbios1.s"), options(att_syntax));
global_asm!(include_str!("wrapper_sysv.s"), options(att_syntax));

global_asm!(include_str!("debug_helper.s"), options(att_syntax));
