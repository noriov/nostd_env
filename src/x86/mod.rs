pub mod halt_forever;
pub mod x86_addr;
pub mod x86_far_ptr;
pub mod x86_get_addr;

pub use self::halt_forever::halt_forever;
pub use self::x86_addr::X86Addr;
pub use self::x86_far_ptr::X86FarPtr;
pub use self::x86_get_addr::X86GetAddr;

pub const FLAGS_CF: u16 = 0x0001;
