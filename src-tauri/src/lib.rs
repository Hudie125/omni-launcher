pub mod config;
pub mod hotkey;
pub mod monitor;
pub mod search;
pub mod storage;
pub mod system;
pub mod window;

use config::AppConfig;
use monitor::ResourceMonitor;
use search::{SearchCoordinator, SearchResultItem};
use std::collections::HashMap;
use std::sync::Arc;
use storage::clipboard::{ClipboardItem, ClipboardStorage};
use storage::history::HistoryStorage;
use storage::notes::{NoteItem, NotesStorage};
use system::process::ProcessItem;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, State, WebviewWindow};
use tokio::sync::RwLock;

pub struct AppState {
    pub config: RwLock<AppConfig>,
    pub coordinator: Arc<SearchCoordinator>,
    pub monitor: Arc<ResourceMonitor>,
}

#[tauri::command]
async fn search_query(
    query: String,
    _epoch: u64,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<Vec<SearchResultItem>, String> {
    let cfg = state.config.read().await.clone();
    let results = state
        .coordinator
        .search(
            &query,
            &cfg.file_search_dirs,
            &cfg.file_exclude_patterns,
            &cfg.ollama_model,
            Some(app),
        )
        .await;

    Ok(results)
}

#[tauri::command]
async fn execute_action(
    item: SearchResultItem,
    as_admin: Option<bool>,
    window: WebviewWindow,
    _state: State<'_, AppState>,
) -> Result<(), String> {
    // Record usage frequency statistics
    HistoryStorage::bump_usage(&format!("{}|{}", item.item_type, item.title));

    // Admin elevation requested (Shift+Enter)
    if as_admin == Some(true) {
        if let Some(admin_target) = &item.admin_path {
            let _ = system::launch_as_admin(admin_target);
            let _ = window.hide();
            return Ok(());
        }
    }

    if let Some(path) = &item.path {
        if path.starts_with("window://") {
            if let Ok(hwnd) = path[9..].parse::<isize>() {
                let _ = system::windows::focus_window(hwnd);
            }
        } else if path.starts_with("cmd://") {
            let cmd_str = &path[6..];
            let _ = system::execute_system_command(cmd_str);
        } else if path.starts_with("search://") {
            // Frontend will trigger query
            return Ok(());
        } else if path.starts_with("http://") || path.starts_with("https://") {
            let _ = open::that_detached(path);
        } else {
            // Local file, application, or directory
            let _ = open::that_detached(path);
        }
    }

    // Hide window after successful execution
    let _ = window.hide();
    Ok(())
}

#[tauri::command]
async fn hide_launcher(window: WebviewWindow) -> Result<(), String> {
    window.hide().map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    let cfg = state.config.read().await.clone();
    Ok(cfg)
}

#[tauri::command]
async fn save_config(
    new_config: AppConfig,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<AppConfig, String> {
    {
        let mut cfg = state.config.write().await;
        *cfg = new_config.clone();
    }

    new_config.save().map_err(|e| e.to_string())?;

    // Reposition window with new bottom offset
    if let Some(win) = app.get_webview_window("main") {
        let _ = window::reposition_bottom_center(&win, new_config.bottom_offset);
    }

    Ok(new_config)
}

// --- Notes IPC Commands ---
#[tauri::command]
fn get_notes() -> Result<Vec<NoteItem>, String> {
    Ok(NotesStorage::load_notes())
}

#[tauri::command]
fn add_note(text: String) -> Result<NoteItem, String> {
    NotesStorage::add_note(&text)
}

#[tauri::command]
fn edit_note(id: String, text: String) -> Result<(), String> {
    NotesStorage::edit_note(&id, &text)
}

#[tauri::command]
fn delete_note(id: String) -> Result<(), String> {
    NotesStorage::delete_note(&id)
}

#[tauri::command]
fn delete_all_notes() -> Result<(), String> {
    NotesStorage::delete_all_notes()
}

#[tauri::command]
fn export_notes() -> Result<String, String> {
    NotesStorage::export_notes_to_desktop()
}

// --- Key Binds IPC Commands ---
#[tauri::command]
fn get_binds() -> Result<HashMap<String, String>, String> {
    Ok(HistoryStorage::load_binds())
}

#[tauri::command]
fn save_bind(combo: String, action: String) -> Result<HashMap<String, String>, String> {
    let mut binds = HistoryStorage::load_binds();
    binds.insert(combo, action);
    HistoryStorage::save_binds(&binds)?;
    Ok(binds)
}

#[tauri::command]
fn remove_bind(combo: String) -> Result<HashMap<String, String>, String> {
    let mut binds = HistoryStorage::load_binds();
    binds.remove(&combo);
    HistoryStorage::save_binds(&binds)?;
    Ok(binds)
}

// --- Process Manager IPC Commands ---
#[tauri::command]
fn list_processes(filter: Option<String>) -> Result<Vec<ProcessItem>, String> {
    Ok(system::process::list_running_processes(
        filter.as_deref().unwrap_or(""),
    ))
}

#[tauri::command]
fn kill_process(target: String) -> Result<String, String> {
    if let Ok(pid) = target.parse::<u32>() {
        system::process::kill_process_by_pid(pid)?;
        Ok(format!("PID {} terminated", pid))
    } else {
        let count = system::process::kill_process_by_name(&target)?;
        Ok(format!("{} process(es) terminated", count))
    }
}

// --- Autostart IPC Commands ---
#[tauri::command]
fn is_autostart() -> Result<bool, String> {
    Ok(system::autostart::is_autostart_enabled())
}

#[tauri::command]
fn set_autostart(enabled: bool) -> Result<(), String> {
    system::autostart::set_autostart(enabled)
}

// --- Clipboard & Reveal IPC Commands ---
#[tauri::command]
fn get_clipboard_items(query: Option<String>) -> Result<Vec<ClipboardItem>, String> {
    Ok(ClipboardStorage::search(query.as_deref().unwrap_or("")))
}

#[tauri::command]
fn push_clipboard_item(text: String) -> Result<(), String> {
    ClipboardStorage::push_item(&text);
    Ok(())
}

#[tauri::command]
fn reveal_file(path: String) -> Result<(), String> {
    system::reveal_in_file_manager(&path)
}

fn debug_log(msg: &str) {
    if let Some(mut path) = dirs::config_dir() {
        path.push("Finder");
        path.push("finder_debug.log");
        if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
            use std::io::Write;
            let _ = writeln!(file, "{}", msg);
        }
    }
}

pub fn run() {
    std::panic::set_hook(Box::new(|info| {
        debug_log(&format!("PANIC: {}", info));
    }));

    debug_log("finder_lib::run entered");
    debug_log("loading app config");
    let app_config = AppConfig::load();
    debug_log("creating SearchCoordinator");
    let coordinator = Arc::new(SearchCoordinator::new());
    debug_log("creating ResourceMonitor");
    let monitor = Arc::new(ResourceMonitor::new());
    debug_log("building tauri app");

    let state = AppState {
        config: RwLock::new(app_config.clone()),
        coordinator,
        monitor: monitor.clone(),
    };

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .manage(state)
        .setup(move |app| {
            debug_log("tauri setup callback entered");
            let handle = app.handle();

            if let Some(window) = app.get_webview_window("main") {
                debug_log("found main webview window, configuring effects and showing");
                // Apply Acrylic/Mica/DWM Hardware Rounded Corners
                window::setup_window_effects(&window);
                // Position centered at bottom of screen
                let _ = window::reposition_bottom_center(&window, app_config.bottom_offset);
                // Reveal launcher on initial startup so user immediately sees HUD
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
                let _ = window.emit("launcher-shown", ());
                debug_log("window shown successfully");
            } else {
                debug_log("[ERROR] main webview window not found in setup");
            }

            // Start resource monitor background worker
            monitor.set_active(true);
            monitor.spawn_worker(handle.clone());

            // Start Hotkey listener
            let hotkey_manager =
                hotkey::HotkeyManager::new(handle.clone(), app_config.bottom_offset);
            hotkey_manager.start_listener(&app_config.hotkey);

            // Setup System Tray
            if let Err(e) = setup_system_tray(handle) {
                debug_log(&format!("[WARN] setup_system_tray: {:?}", e));
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            debug_log(&format!("window event: {:?}", event));
            match event {
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    debug_log("CloseRequested received -> preventing close and hiding window");
                    api.prevent_close();
                    let _ = window.hide();
                }
                tauri::WindowEvent::Focused(focused) => {
                    if let Some(state) = window.try_state::<AppState>() {
                        state.monitor.set_active(*focused);
                    }
                }
                _ => {}
            }
        })
        .invoke_handler(tauri::generate_handler![
            search_query,
            execute_action,
            hide_launcher,
            get_config,
            save_config,
            get_notes,
            add_note,
            edit_note,
            delete_note,
            delete_all_notes,
            export_notes,
            get_binds,
            save_bind,
            remove_bind,
            list_processes,
            kill_process,
            is_autostart,
            set_autostart,
            get_clipboard_items,
            push_clipboard_item,
            reveal_file
        ]);

    debug_log("building tauri application with context");
    let app = builder
        .build(tauri::generate_context!())
        .expect("error while building Finder application");

    debug_log("running tauri application with ExitRequested interceptor");
    app.run(|_app_handle, event| {
        match event {
            tauri::RunEvent::ExitRequested { api, .. } => {
                debug_log("RunEvent::ExitRequested received -> calling api.prevent_exit()");
                api.prevent_exit();
            }
            _ => {}
        }
    });
}

fn setup_system_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let show_i = MenuItem::with_id(app, "show", "Show Finder", true, None::<&str>)?;
    let settings_i = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let notes_i = MenuItem::with_id(app, "notes", "Notes", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", "Exit", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&show_i, &settings_i, &notes_i, &quit_i])?;

    let mut builder = TrayIconBuilder::new()
        .menu(&menu)
        .show_menu_on_left_click(false);

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }

    let _tray = builder
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                let _ = window::toggle_launcher(app, 80);
            }
            "settings" => {
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.show();
                    let _ = win.set_focus();
                    let _ = win.emit("open-settings", ());
                }
            }
            "notes" => {
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.show();
                    let _ = win.set_focus();
                    let _ = win.emit("open-notes", ());
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                let _ = window::toggle_launcher(app, 80);
            }
        })
        .build(app)?;

    Ok(())
}
