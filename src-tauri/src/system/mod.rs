pub mod autostart;
pub mod power;
pub mod process;
pub mod windows;

use std::process::Command;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn spawn_silent(program: &str, args: &[&str]) {
    let mut cmd = Command::new(program);
    cmd.args(args);
    #[cfg(target_os = "windows")]
    {
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    let _ = cmd.spawn();
}

pub fn open_path(path: &str) -> Result<(), String> {
    open::that_detached(path).map_err(|e| e.to_string())
}

pub fn reveal_in_file_manager(path: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let arg = format!("/select,\"{}\"", path);
        let mut cmd = Command::new("explorer");
        cmd.raw_arg(&arg);
        cmd.spawn().map(|_| ()).map_err(|e| e.to_string())
    }
    #[cfg(target_os = "linux")]
    {
        if let Some(parent) = std::path::Path::new(path).parent() {
            open::that_detached(parent).map_err(|e| e.to_string())
        } else {
            open::that_detached(path).map_err(|e| e.to_string())
        }
    }
}

pub fn launch_as_admin(path: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::UI::Shell::ShellExecuteW;
        use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

        let op: Vec<u16> = OsStr::new("runas").encode_wide().chain(Some(0)).collect();
        let file: Vec<u16> = OsStr::new(path).encode_wide().chain(Some(0)).collect();

        let ret = unsafe {
            ShellExecuteW(
                std::ptr::null_mut(),
                op.as_ptr(),
                file.as_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                SW_SHOWNORMAL,
            )
        };

        if ret as isize > 32 {
            Ok(())
        } else {
            Err(format!("ShellExecute runas failed with code {}", ret as isize))
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = path;
        Err("Admin elevation via runas is only supported on Windows".into())
    }
}

pub fn execute_system_command(cmd: &str) -> Result<String, String> {
    let parts: Vec<&str> = cmd.trim().split_whitespace().collect();
    if parts.is_empty() {
        return Err("Empty command".into());
    }

    match parts[0].to_lowercase().as_str() {
        "lock" => {
            power::lock_workstation()?;
            Ok("Screen locked".into())
        }
        "sleep" | "suspend" => {
            power::sleep_system()?;
            Ok("Entering sleep mode".into())
        }
        "restart" | "reboot" => {
            let delay = parts.get(1).and_then(|s| s.parse::<u32>().ok()).unwrap_or(10);
            power::reboot_system(delay)?;
            Ok(format!("System restarting in {}s", delay))
        }
        "shutdown" | "poweroff" => {
            let delay = parts.get(1).and_then(|s| s.parse::<u32>().ok()).unwrap_or(10);
            power::shutdown_system(delay)?;
            Ok(format!("System shutting down in {}s", delay))
        }
        "cancel" | "cancel_shutdown" => {
            power::cancel_shutdown()?;
            Ok("Scheduled reboot/shutdown cancelled".into())
        }
        "home" => {
            if let Some(p) = dirs::home_dir() {
                open_path(&p.to_string_lossy())?;
                Ok("Opened Home folder".into())
            } else {
                Err("Could not find Home directory".into())
            }
        }
        "downloads" => {
            if let Some(p) = dirs::download_dir() {
                open_path(&p.to_string_lossy())?;
                Ok("Opened Downloads folder".into())
            } else {
                Err("Could not find Downloads directory".into())
            }
        }
        "docs" | "documents" => {
            if let Some(p) = dirs::document_dir() {
                open_path(&p.to_string_lossy())?;
                Ok("Opened Documents folder".into())
            } else {
                Err("Could not find Documents directory".into())
            }
        }
        "pics" | "pictures" => {
            if let Some(p) = dirs::picture_dir() {
                open_path(&p.to_string_lossy())?;
                Ok("Opened Pictures folder".into())
            } else {
                Err("Could not find Pictures directory".into())
            }
        }
        "videos" => {
            if let Some(p) = dirs::video_dir() {
                open_path(&p.to_string_lossy())?;
                Ok("Opened Videos folder".into())
            } else {
                Err("Could not find Videos directory".into())
            }
        }
        "desktop" => {
            if let Some(p) = dirs::desktop_dir() {
                open_path(&p.to_string_lossy())?;
                Ok("Opened Desktop folder".into())
            } else {
                Err("Could not find Desktop directory".into())
            }
        }
        "exit" => {
            std::process::exit(0);
        }
        "taskmgr" => {
            open_path("taskmgr.exe")?;
            Ok("Opened Task Manager".into())
        }
        "panel" => {
            open_path("control.exe")?;
            Ok("Opened Control Panel".into())
        }
        "settings" => {
            open_path("ms-settings:")?;
            Ok("Opened Windows Settings".into())
        }
        "snip" => {
            open_path("ms-screenclip:")?;
            Ok("Opened Snipping Tool".into())
        }
        "ps" | "powershell" => {
            open_path("powershell.exe")?;
            Ok("Opened PowerShell".into())
        }
        "wt" | "terminal" => {
            open_path("wt.exe")?;
            Ok("Opened Terminal".into())
        }
        "regedit" => {
            open_path("regedit.exe")?;
            Ok("Opened Registry Editor".into())
        }
        "services" => {
            open_path("services.msc")?;
            Ok("Opened Services".into())
        }
        "disk" => {
            open_path("diskmgmt.msc")?;
            Ok("Opened Disk Management".into())
        }
        "devmgmt" => {
            open_path("devmgmt.msc")?;
            Ok("Opened Device Manager".into())
        }
        "eventvwr" => {
            open_path("eventvwr.msc")?;
            Ok("Opened Event Viewer".into())
        }
        "msinfo" => {
            open_path("msinfo32.exe")?;
            Ok("Opened System Info".into())
        }
        "paint" => {
            open_path("mspaint.exe")?;
            Ok("Opened Paint".into())
        }
        "wordpad" => {
            open_path("wordpad.exe")?;
            Ok("Opened WordPad".into())
        }
        "osk" => {
            open_path("osk.exe")?;
            Ok("Opened On-Screen Keyboard".into())
        }
        "charmap" => {
            open_path("charmap.exe")?;
            Ok("Opened Character Map".into())
        }
        "empty" => {
            if parts.get(1).map(|s| s.to_lowercase()) == Some("trash".into())
                || parts.get(1).map(|s| s.to_lowercase()) == Some("bin".into())
            {
                #[cfg(target_os = "windows")]
                spawn_silent(
                    "powershell",
                    &["-NoProfile", "-Command", "Clear-RecycleBin -Force -ErrorAction SilentlyContinue"],
                );
                #[cfg(target_os = "linux")]
                {
                    if let Some(home) = dirs::home_dir() {
                        let trash_dir = home.join(".local/share/Trash/files");
                        let _ = std::fs::remove_dir_all(&trash_dir);
                        let _ = std::fs::create_dir_all(&trash_dir);
                    }
                }
                Ok("Recycle Bin emptied".into())
            } else {
                Err("Unknown empty command".into())
            }
        }
        "kill" => {
            if let Some(target) = parts.get(1) {
                if let Ok(pid) = target.parse::<u32>() {
                    process::kill_process_by_pid(pid)?;
                    Ok(format!("Terminated PID {}", pid))
                } else {
                    let count = process::kill_process_by_name(target)?;
                    Ok(format!("Terminated {} process(es) matching '{}'", count, target))
                }
            } else {
                Err("Usage: kill <process_name or pid>".into())
            }
        }
        _ => Err(format!("Unknown command '{}'", parts[0])),
    }
}
