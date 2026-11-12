#[cfg(target_os = "windows")]
use windows_sys::Win32::Foundation::{BOOL, HWND, LPARAM};
#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowTextLengthW, GetWindowTextW, IsWindowVisible, SetForegroundWindow,
    ShowWindow, SW_RESTORE,
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WindowInfo {
    pub title: String,
    pub hwnd: isize,
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let out = &mut *(lparam as *mut Vec<WindowInfo>);
    if IsWindowVisible(hwnd) != 0 {
        let len = GetWindowTextLengthW(hwnd);
        if len > 0 {
            let mut buf = vec![0u16; (len + 1) as usize];
            let read = GetWindowTextW(hwnd, buf.as_mut_ptr(), len + 1);
            if read > 0 {
                let title = String::from_utf16_lossy(&buf[..read as usize]);
                let trimmed = title.trim();
                // Filter out invisible overlays and self window
                if !trimmed.is_empty() && trimmed != "Finder" && trimmed != "Program Manager" {
                    out.push(WindowInfo {
                        title: trimmed.to_string(),
                        hwnd: hwnd as isize,
                    });
                }
            }
        }
    }
    1
}

pub fn get_running_windows(query: &str) -> Vec<WindowInfo> {
    let q = query.to_lowercase();
    #[cfg(target_os = "windows")]
    {
        let mut list: Vec<WindowInfo> = Vec::new();
        unsafe {
            EnumWindows(
                Some(enum_windows_proc),
                &mut list as *mut Vec<WindowInfo> as LPARAM,
            );
        }
        if q.is_empty() {
            list.into_iter().take(5).collect()
        } else {
            list.into_iter()
                .filter(|w| w.title.to_lowercase().contains(&q))
                .take(5)
                .collect()
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        Vec::new()
    }
}

pub fn focus_window(hwnd: isize) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    unsafe {
        ShowWindow(hwnd as HWND, SW_RESTORE);
        SetForegroundWindow(hwnd as HWND);
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = hwnd;
        Err("Active window switching is supported on Windows".into())
    }
}
