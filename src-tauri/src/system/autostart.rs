#[cfg(target_os = "windows")]
use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_SET_VALUE};
#[cfg(target_os = "windows")]
use winreg::RegKey;

const APP_NAME: &str = "Finder";

pub fn is_autostart_enabled() -> bool {
    #[cfg(target_os = "windows")]
    {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok(run_key) = hkcu.open_subkey_with_flags(
            r"Software\Microsoft\Windows\CurrentVersion\Run",
            KEY_READ,
        ) {
            let res: Result<String, _> = run_key.get_value(APP_NAME);
            return res.is_ok();
        }
        false
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(config_dir) = dirs::config_dir() {
            let desktop_file = config_dir.join("autostart").join("finder.desktop");
            return desktop_file.exists();
        }
        false
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        false
    }
}

pub fn set_autostart(enabled: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let run_key = hkcu
            .open_subkey_with_flags(
                r"Software\Microsoft\Windows\CurrentVersion\Run",
                KEY_SET_VALUE,
            )
            .map_err(|e| e.to_string())?;

        if enabled {
            let current_exe = std::env::current_exe().map_err(|e| e.to_string())?;
            let exe_str = format!("\"{}\"", current_exe.to_string_lossy());
            run_key
                .set_value(APP_NAME, &exe_str)
                .map_err(|e| e.to_string())?;
        } else {
            let _ = run_key.delete_value(APP_NAME);
        }
        Ok(())
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(config_dir) = dirs::config_dir() {
            let autostart_dir = config_dir.join("autostart");
            let desktop_file = autostart_dir.join("finder.desktop");
            if enabled {
                let _ = std::fs::create_dir_all(&autostart_dir);
                let current_exe = std::env::current_exe().map_err(|e| e.to_string())?;
                let content = format!(
                    "[Desktop Entry]\nType=Application\nName=Finder\nExec={}\nHidden=false\nNoDisplay=false\nX-GNOME-Autostart-enabled=true\n",
                    current_exe.to_string_lossy()
                );
                std::fs::write(desktop_file, content).map_err(|e| e.to_string())?;
            } else if desktop_file.exists() {
                let _ = std::fs::remove_file(desktop_file);
            }
        }
        Ok(())
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        let _ = enabled;
        Ok(())
    }
}
