#![windows_subsystem = "windows"]

static mut MUTEX_HANDLE: *mut std::ffi::c_void = std::ptr::null_mut();

fn log(msg: &str) {
    if let Some(mut path) = dirs::config_dir() {
        path.push("Finder");
        let _ = std::fs::create_dir_all(&path);
        path.push("finder_debug.log");
        if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
            use std::io::Write;
            let pid = std::process::id();
            let _ = writeln!(file, "[PID {}] {}", pid, msg);
        }
    }
}

#[cfg(target_os = "windows")]
fn is_another_instance_running() -> bool {
    extern "system" {
        fn CreateMutexW(
            lp_mutex_attributes: *const std::ffi::c_void,
            b_initial_owner: i32,
            lp_name: *const u16,
        ) -> *mut std::ffi::c_void;
        fn GetLastError() -> u32;
        fn SetLastError(dw_err_code: u32);
    }
    const ERROR_ALREADY_EXISTS: u32 = 183;

    unsafe {
        SetLastError(0);
        let name: [u16; 23] = [
            'L' as u16, 'o' as u16, 'c' as u16, 'a' as u16, 'l' as u16, '\\' as u16,
            'F' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'e' as u16, 'r' as u16,
            'S' as u16, 'i' as u16, 'n' as u16, 'g' as u16, 'l' as u16, 'e' as u16,
            'I' as u16, 'n' as u16, 's' as u16, 't' as u16, 0,
        ];
        let handle = CreateMutexW(std::ptr::null(), 0, name.as_ptr());
        let err = GetLastError();
        if handle.is_null() {
            log(&format!("CreateMutexW returned NULL, err={}", err));
            return false;
        }
        if err == ERROR_ALREADY_EXISTS {
            log(&format!("CreateMutexW returned ERROR_ALREADY_EXISTS ({})", err));
            return true;
        }
        MUTEX_HANDLE = handle;
    }
    false
}

#[cfg(not(target_os = "windows"))]
fn is_another_instance_running() -> bool {
    false
}

fn main() {
    log("main() entered");
    if is_another_instance_running() {
        log("another instance detected, exiting immediately");
        return;
    }

    log("calling finder_lib::run()");
    finder_lib::run();
    log("finder_lib::run() returned");
}
