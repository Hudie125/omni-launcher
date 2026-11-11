pub mod config;
pub mod hotkey;
pub mod monitor;
pub mod search;
pub mod system;
pub mod window;

use config::AppConfig;
use monitor::ResourceMonitor;
use search::{SearchCoordinator, SearchResultItem};
use std::sync::Arc;
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
) -> Result<Vec<SearchResultItem>, String> {
    let cfg = state.config.read().await.clone();
    let results = state
        .coordinator
        .search(
            &query,
            &cfg.file_search_dirs,
            &cfg.file_exclude_patterns,
            &cfg.ollama_model,
        )
        .await;

    Ok(results)
}

#[tauri::command]
async fn execute_action(
    item: SearchResultItem,
    window: WebviewWindow,
    _state: State<'_, AppState>,
) -> Result<(), String> {
    if let Some(path) = &item.path {
        if path.starts_with("cmd://") {
            let cmd_str = &path[6..];
            let _ = system::execute_system_command(cmd_str);
        } else if path.starts_with("note://") {
            let note_str = &path[7..];
            let _ = search::notes::save_note_to_file(note_str);
        } else if path.starts_with("http://") || path.starts_with("https://") {
            let _ = open::that_detached(path);
        } else {
            // Local file, app, or directory
            let _ = open::that_detached(path);
        }
    } else if item.card_type.as_deref() == Some("math") {
        // Fallback for math cards: copy value to clipboard
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

pub fn run() {
    let app_config = AppConfig::load();
    let coordinator = Arc::new(SearchCoordinator::new());
    let monitor = Arc::new(ResourceMonitor::new());

    let state = AppState {
        config: RwLock::new(app_config.clone()),
        coordinator,
        monitor: monitor.clone(),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .manage(state)
        .setup(move |app| {
            let handle = app.handle();

            if let Some(window) = app.get_webview_window("main") {
                // Apply Acrylic/Mica/DWM Hardware Rounded Corners
                window::setup_window_effects(&window);
                // Position centered at bottom of screen
                let _ = window::reposition_bottom_center(&window, app_config.bottom_offset);
                // Reveal launcher on initial startup so user immediately sees HUD
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
                let _ = window.emit("launcher-shown", ());
            }

            // Start resource monitor background worker
            monitor.set_active(true);
            monitor.spawn_worker(handle.clone());

            // Start Hotkey listener
            let hotkey_manager = hotkey::HotkeyManager::new(handle.clone(), app_config.bottom_offset);
            hotkey_manager.start_listener(&app_config.hotkey);

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::Focused(focused) = event {
                if let Some(state) = window.try_state::<AppState>() {
                    state.monitor.set_active(*focused);
                }
                // Never auto-hide on focus loss: only toggles on Right Alt
            }
        })
        .invoke_handler(tauri::generate_handler![
            search_query,
            execute_action,
            hide_launcher,
            get_config,
            save_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running Finder application");
}
