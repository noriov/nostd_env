/*!

Calls BIOS functions.

 */

#[doc(hidden)] pub mod api;
pub mod asm;
pub mod ffi;
pub mod int10h0eh;
pub mod int10h4f00h;
pub mod int10h4f01h;
pub mod int10h4f02h;
pub mod int10h4f03h;
pub mod int13h02h;
pub mod int13h42h;
pub mod int15he820h;
#[doc(hidden)] pub mod lmbios_regs;
#[doc(hidden)] pub mod stack_usage;

#[doc(inline)] pub use self::api::get_boot_drive_id;
#[doc(inline)] pub use self::lmbios_regs::LmbiosRegs;
#[doc(inline)] pub use self::stack_usage::StackUsage;
