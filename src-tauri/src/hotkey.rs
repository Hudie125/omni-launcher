use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::AppHandle;

pub struct HotkeyManager {
    app: AppHandle,
    bottom_offset: i32,
    is_running: Arc<AtomicBool>,
}

impl HotkeyManager {
    pub fn new(app: AppHandle, bottom_offset: i32) -> Self {
        Self {
            app,
            bottom_offset,
            is_running: Arc::new(AtomicBool::new(true)),
        }
    }

    pub fn start_listener(&self, hotkey_name: &str) {
        let app = self.app.clone();
        let bottom_offset = self.bottom_offset;
        let is_running = self.is_running.clone();
        let hotkey_str = hotkey_name.to_lowercase();

        #[cfg(target_os = "windows")]
        {
            if hotkey_str == "rightalt" || hotkey_str == "alt_r" || hotkey_str == "right_alt" {
                std::thread::spawn(move || {
                    start_windows_right_alt_hook(app, bottom_offset, is_running);
                });
                return;
            }
        }

        // For combo hotkeys like "Alt+Space", "Ctrl+Space"
        let app_handle = self.app.clone();
        let hotkey_owned = hotkey_name.to_string();
        std::thread::spawn(move || {
            register_standard_hotkey(app_handle, &hotkey_owned, bottom_offset);
        });
    }
}

fn register_standard_hotkey(app: AppHandle, hotkey_str: &str, bottom_offset: i32) {
    use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
    if let Ok(shortcut) = hotkey_str.parse::<Shortcut>() {
        let app_for_cb = app.clone();
        let _ = app.global_shortcut().on_shortcut(shortcut, move |_app, _sc, event| {
            if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                let _ = crate::window::toggle_launcher(&app_for_cb, bottom_offset);
            }
        });
    }
}

#[cfg(target_os = "windows")]
fn start_windows_right_alt_hook(app: AppHandle, bottom_offset: i32, is_running: Arc<AtomicBool>) {
    use std::sync::atomic::AtomicU64;
    use std::sync::OnceLock;
    use windows_sys::Win32::Foundation::{HINSTANCE, LPARAM, LRESULT, WPARAM};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW,
        TranslateMessage, UnhookWindowsHookEx, HHOOK, KBDLLHOOKSTRUCT, MSG,
        WH_KEYBOARD_LL, WM_KEYUP, WM_SYSKEYUP,
    };

    struct HookContext {
        app: AppHandle,
        bottom_offset: i32,
    }

    static CTX: OnceLock<HookContext> = OnceLock::new();
    let _ = CTX.set(HookContext { app, bottom_offset });
    static LAST_KEY_UP: AtomicU64 = AtomicU64::new(0);

    unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        if code >= 0 {
            let msg = wparam as u32;
            if msg == WM_KEYUP || msg == WM_SYSKEYUP {
                let kbd = *(lparam as *const KBDLLHOOKSTRUCT);
                // VK_RMENU is 0xA5 (Right Alt), or VK_MENU (0x12) with extended bit 0x1
                let is_right_alt = kbd.vkCode == 0xA5 || (kbd.vkCode == 0x12 && (kbd.flags & 1) != 0);
                if is_right_alt {
                    let now = crate::window::now_millis();
                    let last = LAST_KEY_UP.load(Ordering::Relaxed);
                    if now.saturating_sub(last) >= 280 {
                        LAST_KEY_UP.store(now, Ordering::Relaxed);
                        if let Some(ctx) = CTX.get() {
                            let _ = crate::window::toggle_launcher(&ctx.app, ctx.bottom_offset);
                        }
                    }
                }
            }
        }
        CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam)
    }

    unsafe {
        let hook: HHOOK = SetWindowsHookExW(
            WH_KEYBOARD_LL,
            Some(hook_proc),
            std::ptr::null_mut() as HINSTANCE,
            0,
        );

        let mut msg: MSG = std::mem::zeroed();
        while is_running.load(Ordering::Relaxed) && GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        if !hook.is_null() {
            UnhookWindowsHookEx(hook);
        }
    }
}
