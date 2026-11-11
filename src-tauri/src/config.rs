use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub hotkey: String,
    pub bottom_offset: i32,
    pub theme: String,
    pub hide_on_blur: bool,
    pub ollama_model: String,
    pub file_search_dirs: Vec<String>,
    pub file_exclude_patterns: Vec<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        let mut default_dirs = Vec::new();
        if let Some(user_dirs) = dirs::document_dir() {
            default_dirs.push(user_dirs.to_string_lossy().to_string());
        }
        if let Some(dl_dir) = dirs::download_dir() {
            default_dirs.push(dl_dir.to_string_lossy().to_string());
        }
        if let Some(dt_dir) = dirs::desktop_dir() {
            default_dirs.push(dt_dir.to_string_lossy().to_string());
        }

        Self {
            hotkey: "RightAlt".to_string(),
            bottom_offset: 80,
            theme: "oled".to_string(),
            hide_on_blur: true,
            ollama_model: "auto".to_string(),
            file_search_dirs: default_dirs,
            file_exclude_patterns: vec![
                "node_modules".into(),
                ".git".into(),
                "target".into(),
                "vendor".into(),
                "Windows".into(),
                "proc".into(),
                "sys".into(),
                "$Recycle.Bin".into(),
                "AppData\\Local\\Temp".into(),
            ],
        }
    }
}

impl AppConfig {
    pub fn config_path() -> PathBuf {
        let base_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        base_dir.join("omni-launcher").join("config.toml")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(cfg) = toml::from_str::<AppConfig>(&content) {
                    return cfg;
                }
            }
        }
        let default_cfg = Self::default();
        let _ = default_cfg.save();
        default_cfg
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
