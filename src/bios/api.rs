use super::ffi;


pub fn get_boot_drive_id() -> u8 {
    unsafe {
	ffi::lmbios_get_boot_drive_id()
    }
}
