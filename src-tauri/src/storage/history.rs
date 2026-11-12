use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

const MAX_HISTORY: usize = 30;

pub struct HistoryStorage;

impl HistoryStorage {
    fn config_dir() -> PathBuf {
        let base_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        let dir = base_dir.join("Finder");
        let _ = fs::create_dir_all(&dir);
        dir
    }

    // 1. Query History
    pub fn load_history() -> Vec<String> {
        let path = Self::config_dir().join("search_history.json");
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(list) = serde_json::from_str::<Vec<String>>(&content) {
                    return list;
                }
            }
        }
        Vec::new()
    }

    pub fn push_history(query: &str) {
        let q = query.trim();
        if q.len() < 2 {
            return;
        }

        let mut list = Self::load_history();
        list.retain(|item| item.to_lowercase() != q.to_lowercase());
        list.insert(0, q.to_string());
        if list.len() > MAX_HISTORY {
            list.truncate(MAX_HISTORY);
        }

        let path = Self::config_dir().join("search_history.json");
        if let Ok(content) = serde_json::to_string_pretty(&list) {
            let _ = fs::write(path, content);
        }
    }

    // 2. Usage Frequency Stats
    pub fn load_usage() -> HashMap<String, u32> {
        let path = Self::config_dir().join("usage_stats.json");
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(map) = serde_json::from_str::<HashMap<String, u32>>(&content) {
                    return map;
                }
            }
        }
        HashMap::new()
    }

    pub fn bump_usage(key: &str) {
        let mut usage = Self::load_usage();
        let count = usage.entry(key.to_string()).or_insert(0);
        *count += 1;

        let path = Self::config_dir().join("usage_stats.json");
        if let Ok(content) = serde_json::to_string_pretty(&usage) {
            let _ = fs::write(path, content);
        }
    }

    pub fn get_top_used(limit: usize) -> Vec<(String, u32)> {
        let usage = Self::load_usage();
        let mut items: Vec<(String, u32)> = usage.into_iter().collect();
        items.sort_by(|a, b| b.1.cmp(&a.1));
        items.truncate(limit);
        items
    }

    // 3. Custom Key Binds
    pub fn load_binds() -> HashMap<String, String> {
        let path = Self::config_dir().join("key_binds.json");
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(map) = serde_json::from_str::<HashMap<String, String>>(&content) {
                    return map;
                }
            }
        }
        HashMap::new()
    }

    pub fn save_binds(binds: &HashMap<String, String>) -> Result<(), String> {
        let path = Self::config_dir().join("key_binds.json");
        let content = serde_json::to_string_pretty(binds).map_err(|e| e.to_string())?;
        fs::write(path, content).map_err(|e| e.to_string())
    }
}
