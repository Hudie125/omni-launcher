use std::sync::atomic::{AtomicU64, Ordering};
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, WebviewWindow};

pub static LAST_SHOWN_MS: AtomicU64 = AtomicU64::new(0);
static LAST_TOGGLE_MS: AtomicU64 = AtomicU64::new(0);

pub fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

pub fn setup_window_effects(window: &WebviewWindow) {
    #[cfg(target_os = "windows")]
    unsafe {
        use windows_sys::Win32::Graphics::Dwm::{
            DwmSetWindowAttribute, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND,
        };
        if let Ok(hwnd) = window.hwnd() {
            // Apply DWM hardware rounded corners on Windows 11
            let preference = DWMWCP_ROUND;
            let _ = DwmSetWindowAttribute(
                hwnd.0 as _,
                DWMWA_WINDOW_CORNER_PREFERENCE as u32,
                &preference as *const _ as *const _,
                std::mem::size_of::<u32>() as u32,
            );

            // Apply native Windows 11 DWM backdrop: Acrylic (3) or Mica (2)
            const DWMWA_SYSTEMBACKDROP_TYPE: u32 = 38;
            let backdrop_type: u32 = 3; // DWMSBT_TRANSIENTWINDOW
            let _ = DwmSetWindowAttribute(
                hwnd.0 as _,
                DWMWA_SYSTEMBACKDROP_TYPE,
                &backdrop_type as *const _ as *const _,
                std::mem::size_of::<u32>() as u32,
            );
        }
    }
}

pub fn reposition_bottom_center(window: &WebviewWindow, bottom_offset: i32) -> Result<(), String> {
    let monitor = window
        .current_monitor()
        .map_err(|e| e.to_string())?
        .or_else(|| window.primary_monitor().ok().flatten())
        .ok_or_else(|| "No monitor detected".to_string())?;

    let mon_pos = monitor.position();
    let mon_size = monitor.size();
    let scale_factor = monitor.scale_factor();

    let win_size = window.outer_size().map_err(|e| e.to_string())?;

    let offset_phys = (bottom_offset as f64 * scale_factor) as i32;

    let target_x = mon_pos.x + (mon_size.width as i32 - win_size.width as i32) / 2;
    let target_y = mon_pos.y + (mon_size.height as i32 - win_size.height as i32) - offset_phys;

    window
        .set_position(tauri::Position::Physical(PhysicalPosition::new(
            target_x, target_y,
        )))
        .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn toggle_launcher(app: &AppHandle, bottom_offset: i32) -> Result<(), String> {
    let now = now_millis();
    let last = LAST_TOGGLE_MS.load(Ordering::Relaxed);
    if now.saturating_sub(last) < 280 {
        return Ok(()); // Debounce rapid repeated triggers
    }
    LAST_TOGGLE_MS.store(now, Ordering::Relaxed);

    if let Some(window) = app.get_webview_window("main") {
        let is_visible = window.is_visible().unwrap_or(false);
        if is_visible {
            window.hide().map_err(|e| e.to_string())?;
        } else {
            LAST_SHOWN_MS.store(now, Ordering::Relaxed);
            reposition_bottom_center(&window, bottom_offset)?;
            window.show().map_err(|e| e.to_string())?;
            window.unminimize().map_err(|e| e.to_string())?;
            window.set_focus().map_err(|e| e.to_string())?;
            let _ = window.emit("launcher-shown", ());
        }
    }
    Ok(())
}
