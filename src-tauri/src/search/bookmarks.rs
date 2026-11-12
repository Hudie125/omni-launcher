use super::SearchResultItem;
use serde_json::Value;
use std::path::PathBuf;

pub struct ChromeBookmarksScanner;

impl ChromeBookmarksScanner {
    pub fn search(query: &str) -> Vec<SearchResultItem> {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return Vec::new();
        }

        let paths = Self::get_bookmarks_paths();
        let mut results = Vec::new();

        for path in paths {
            if !path.exists() {
                continue;
            }
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(json) = serde_json::from_str::<Value>(&content) {
                    if let Some(roots) = json.get("roots").and_then(|r| r.as_object()) {
                        for root_val in roots.values() {
                            Self::traverse_bookmarks(root_val, &q, &mut results);
                            if results.len() >= 5 {
                                return results;
                            }
                        }
                    }
                }
            }
        }

        results
    }

    fn get_bookmarks_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        #[cfg(target_os = "windows")]
        {
            if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
                let base = PathBuf::from(local_app_data).join(r"Google\Chrome\User Data");
                paths.push(base.join(r"Default\Bookmarks"));
                paths.push(base.join(r"Profile 1\Bookmarks"));
                paths.push(base.join(r"Profile 2\Bookmarks"));
            }
        }

        #[cfg(target_os = "linux")]
        {
            if let Some(config_dir) = dirs::config_dir() {
                let chrome_base = config_dir.join("google-chrome");
                paths.push(chrome_base.join("Default/Bookmarks"));
                paths.push(chrome_base.join("Profile 1/Bookmarks"));
                let chromium_base = config_dir.join("chromium");
                paths.push(chromium_base.join("Default/Bookmarks"));
            }
        }

        paths
    }

    fn traverse_bookmarks(node: &Value, q: &str, out: &mut Vec<SearchResultItem>) {
        if out.len() >= 5 {
            return;
        }

        if let Some(obj) = node.as_object() {
            let node_type = obj.get("type").and_then(|t| t.as_str()).unwrap_or_default();
            if node_type == "url" {
                let name = obj.get("name").and_then(|n| n.as_str()).unwrap_or_default();
                let url = obj.get("url").and_then(|u| u.as_str()).unwrap_or_default();

                if name.to_lowercase().contains(q) || url.to_lowercase().contains(q) {
                    out.push(SearchResultItem {
                        id: format!("bookmark_{}", url),
                        title: format!("[Bookmark] {}", name),
                        subtitle: Some(url.to_string()),
                        category: "Files".into(),
                        item_type: "bookmark".into(),
                        card_type: None,
                        path: Some(url.to_string()),
                        action_hint: Some("↵ Open Browser".into()),
                        score: 80,
                        snippet: None,
                        admin_path: None,
                    });
                }
            } else if let Some(children) = obj.get("children").and_then(|c| c.as_array()) {
                for child in children {
                    Self::traverse_bookmarks(child, q, out);
                }
            }
        }
    }
}
