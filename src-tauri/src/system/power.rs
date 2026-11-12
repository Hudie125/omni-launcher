use std::process::Command;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

const CREATE_NO_WINDOW: u32 = 0x08000000;

fn silent_cmd(prog: &str, args: &[&str]) -> Result<(), String> {
    let mut cmd = Command::new(prog);
    cmd.args(args);
    #[cfg(target_os = "windows")]
    {
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd.spawn().map(|_| ()).map_err(|e| e.to_string())
}

pub fn lock_workstation() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        extern "system" {
            fn LockWorkStation() -> i32;
        }
        let success = unsafe { LockWorkStation() };
        if success != 0 {
            Ok(())
        } else {
            Err("Failed to lock workstation".into())
        }
    }
    #[cfg(target_os = "linux")]
    {
        silent_cmd("loginctl", &["lock-session"])
    }
}

pub fn sleep_system() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        silent_cmd("rundll32.exe", &["powrprof.dll,SetSuspendState", "0,1,0"])
    }
    #[cfg(target_os = "linux")]
    {
        silent_cmd("systemctl", &["suspend"])
    }
}

pub fn reboot_system(delay_secs: u32) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let delay_str = delay_secs.to_string();
        silent_cmd("shutdown", &["/r", "/t", &delay_str])
    }
    #[cfg(target_os = "linux")]
    {
        silent_cmd("systemctl", &["reboot"])
    }
}

pub fn shutdown_system(delay_secs: u32) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let delay_str = delay_secs.to_string();
        silent_cmd("shutdown", &["/s", "/t", &delay_str])
    }
    #[cfg(target_os = "linux")]
    {
        silent_cmd("systemctl", &["poweroff"])
    }
}

pub fn cancel_shutdown() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        silent_cmd("shutdown", &["/a"])
    }
    #[cfg(target_os = "linux")]
    {
        silent_cmd("shutdown", &["-c"])
    }
}
