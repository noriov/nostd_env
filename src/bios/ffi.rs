pub use super::LmbiosRegs;


extern "C" {
    // defined in lmbios1.s
    pub fn lmbios_call(regs: &mut LmbiosRegs) -> u16;
    pub fn lmbios_get_boot_drive_id() -> u8;

    // defined in debug_helper.s
    pub fn debug_clear_stack_area() -> u64;
}

extern {
    // defined in the linker script ( config/x86_64-unknown-none.ld )
    pub static __lmb_heap16_start: u8;
    pub static __lmb_heap16_end: u8;
    pub static __lmb_heap32_start: u8;
    pub static __lmb_heap32_end: u8;
    pub static __lmb_stack_start: u8;
    pub static __lmb_stack_end: u8;
}
