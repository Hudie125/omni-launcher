use chrono::Local;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const MAX_CLIP_HISTORY: usize = 50;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardItem {
    pub id: String,
    pub text: String,
    pub ts: String,
}

pub struct ClipboardStorage;

impl ClipboardStorage {
    fn file_path() -> PathBuf {
        let base_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        let dir = base_dir.join("Finder");
        let _ = fs::create_dir_all(&dir);
        dir.join("search_clipboard.json")
    }

    pub fn load_items() -> Vec<ClipboardItem> {
        let path = Self::file_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(items) = serde_json::from_str::<Vec<ClipboardItem>>(&content) {
                    return items;
                }
            }
        }
        Vec::new()
    }

    pub fn push_item(text: &str) {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return;
        }

        let mut items = Self::load_items();
        // Avoid duplicate identical top entry
        if let Some(first) = items.first() {
            if first.text == trimmed {
                return;
            }
        }

        items.retain(|i| i.text != trimmed);
        items.insert(
            0,
            ClipboardItem {
                id: format!("clip_{}", Local::now().timestamp_millis()),
                text: trimmed.to_string(),
                ts: Local::now().format("%H:%M:%S").to_string(),
            },
        );

        if items.len() > MAX_CLIP_HISTORY {
            items.truncate(MAX_CLIP_HISTORY);
        }

        let path = Self::file_path();
        if let Ok(content) = serde_json::to_string_pretty(&items) {
            let _ = fs::write(path, content);
        }
    }

    pub fn search(query: &str) -> Vec<ClipboardItem> {
        let q = query.trim().to_lowercase();
        let items = Self::load_items();
        if q.is_empty() {
            items
        } else {
            items
                .into_iter()
                .filter(|i| i.text.to_lowercase().contains(&q))
                .collect()
        }
    }
}
