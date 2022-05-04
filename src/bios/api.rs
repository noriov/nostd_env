use super::ffi;


pub fn get_boot_drive_id() -> u8 {
    unsafe {
	ffi::lmbios_get_boot_drive_id()
    }
}

pub fn check_stack_usage() -> usize {
    unsafe {
	ffi::debug_clear_stack_area() as usize
    }
}
