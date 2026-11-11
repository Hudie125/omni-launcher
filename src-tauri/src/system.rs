use sysinfo::System;

pub fn execute_system_command(cmd: &str) -> Result<String, String> {
    let parts: Vec<&str> = cmd.trim().split_whitespace().collect();
    if parts.is_empty() {
        return Err("Empty command".into());
    }

    match parts[0].to_lowercase().as_str() {
        "lock" => {
            #[cfg(target_os = "windows")]
            {
                let _ = std::process::Command::new("rundll32.exe")
                    .args(["user32.dll,LockWorkStation"])
                    .spawn();
            }
            #[cfg(target_os = "linux")]
            {
                let _ = std::process::Command::new("loginctl")
                    .arg("lock-session")
                    .spawn();
            }
            Ok("Screen locked".into())
        }
        "sleep" | "suspend" => {
            #[cfg(target_os = "windows")]
            {
                let _ = std::process::Command::new("rundll32.exe")
                    .args(["powrprof.dll,SetSuspendState", "0,1,0"])
                    .spawn();
            }
            #[cfg(target_os = "linux")]
            {
                let _ = std::process::Command::new("systemctl")
                    .arg("suspend")
                    .spawn();
            }
            Ok("Entering sleep mode".into())
        }
        "restart" | "reboot" => {
            #[cfg(target_os = "windows")]
            {
                let _ = std::process::Command::new("shutdown.exe")
                    .args(["/r", "/t", "0"])
                    .spawn();
            }
            #[cfg(target_os = "linux")]
            {
                let _ = std::process::Command::new("systemctl")
                    .arg("reboot")
                    .spawn();
            }
            Ok("System restarting".into())
        }
        "shutdown" | "poweroff" => {
            #[cfg(target_os = "windows")]
            {
                let _ = std::process::Command::new("shutdown.exe")
                    .args(["/s", "/t", "0"])
                    .spawn();
            }
            #[cfg(target_os = "linux")]
            {
                let _ = std::process::Command::new("systemctl")
                    .arg("poweroff")
                    .spawn();
            }
            Ok("System shutting down".into())
        }
        "empty" => {
            if parts.get(1).map(|s| s.to_lowercase()) == Some("trash".into())
                || parts.get(1).map(|s| s.to_lowercase()) == Some("bin".into())
            {
                #[cfg(target_os = "windows")]
                {
                    let _ = std::process::Command::new("powershell")
                        .args(["-NoProfile", "-Command", "Clear-RecycleBin -Force -ErrorAction SilentlyContinue"])
                        .spawn();
                }
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
            if let Some(proc_name) = parts.get(1) {
                let mut sys = System::new_all();
                sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
                let target = proc_name.to_lowercase();
                let mut killed = 0;
                for process in sys.processes().values() {
                    let name = process.name().to_string_lossy().to_lowercase();
                    if name.contains(&target) {
                        if process.kill() {
                            killed += 1;
                        }
                    }
                }
                Ok(format!("Terminated {} process(es) matching '{}'", killed, target))
            } else {
                Err("Usage: kill <process_name>".into())
            }
        }
        _ => Err(format!("Unknown command '{}'", parts[0])),
    }
}
