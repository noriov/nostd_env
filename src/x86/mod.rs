/*!

Provides X86-related utilities.

 */


#[doc(hidden)] pub mod halt_forever;
#[doc(hidden)] pub mod x86_far_ptr;
#[doc(hidden)] pub mod x86_get_addr;

#[doc(inline)] pub use self::halt_forever::halt_forever;
#[doc(inline)] pub use self::x86_far_ptr::X86FarPtr;
#[doc(inline)] pub use self::x86_get_addr::X86GetAddr;

/// The Carry Flag (CF) in the FLAGS register.
pub const FLAGS_CF: u16 = 0x0001;
