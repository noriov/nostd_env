pub mod halt_forever;
pub mod x86_addr;

pub use self::halt_forever::halt_forever;
pub use self::x86_addr::X86Addr;

pub const FLAGS_CF: u16 = 0x0001;
