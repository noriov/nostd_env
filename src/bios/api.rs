use super::ffi;


/// Gets the boot drive id.
pub fn get_boot_drive_id() -> u8 {
    unsafe {
	ffi::lmbios_get_boot_drive_id()
    }
}
