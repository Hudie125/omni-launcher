#![windows_subsystem = "windows"]

#[cfg(target_os = "windows")]
fn is_another_instance_running() -> bool {
    extern "system" {
        fn CreateMutexW(
            lp_mutex_attributes: *const std::ffi::c_void,
            b_initial_owner: i32,
            lp_name: *const u16,
        ) -> *mut std::ffi::c_void;
        fn GetLastError() -> u32;
    }
    const ERROR_ALREADY_EXISTS: u32 = 183;

    unsafe {
        let name: [u16; 23] = [
            'L' as u16, 'o' as u16, 'c' as u16, 'a' as u16, 'l' as u16, '\\' as u16,
            'F' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'e' as u16, 'r' as u16,
            'S' as u16, 'i' as u16, 'n' as u16, 'g' as u16, 'l' as u16, 'e' as u16,
            'I' as u16, 'n' as u16, 's' as u16, 't' as u16, 0,
        ];
        let handle = CreateMutexW(std::ptr::null(), 0, name.as_ptr());
        if handle.is_null() || GetLastError() == ERROR_ALREADY_EXISTS {
            return true;
        }
    }
    false
}

#[cfg(not(target_os = "windows"))]
fn is_another_instance_running() -> bool {
    false
}

fn main() {
    if is_another_instance_running() {
        return;
    }

    finder_lib::run();
}
