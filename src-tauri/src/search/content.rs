use super::SearchResultItem;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::time::Instant;
use tokio_util::sync::CancellationToken;
use walkdir::WalkDir;

const MAX_FILE_SIZE: u64 = 2 * 1024 * 1024; // 2 MB
const TIME_BUDGET_SECS: f64 = 2.0;
const MAX_RESULTS: usize = 5;

pub struct ContentSearchEngine {
    extensions: HashSet<String>,
}

impl ContentSearchEngine {
    pub fn new() -> Self {
        let mut extensions = HashSet::new();
        let list = [
            "txt", "md", "py", "json", "csv", "log", "html", "htm", "css", "js", "ts", "ini",
            "cfg", "xml", "yml", "yaml", "rtf", "bat", "ps1", "sql", "java", "c", "cpp", "h",
            "cs", "go", "rs", "php", "rb", "sh", "toml", "env",
        ];
        for ext in list {
            extensions.insert(ext.to_string());
        }

        Self { extensions }
    }

    pub fn search(&self, query: &str, cancel_token: &CancellationToken) -> Vec<SearchResultItem> {
        let q = query.trim();
        if q.len() < 3 {
            return Vec::new();
        }

        let q_lower = q.to_lowercase();
        let deadline = Instant::now() + std::time::Duration::from_secs_f64(TIME_BUDGET_SECS);

        let mut roots = Vec::new();
        if let Some(user_dirs) = dirs::desktop_dir() {
            roots.push(user_dirs);
        }
        if let Some(user_dirs) = dirs::document_dir() {
            roots.push(user_dirs);
        }
        if let Some(user_dirs) = dirs::download_dir() {
            roots.push(user_dirs);
        }

        let mut results = Vec::new();

        for root in roots {
            if !root.exists() {
                continue;
            }

            for entry in WalkDir::new(&root)
                .into_iter()
                .filter_entry(|e| !is_ignored_dir(e.path()))
                .filter_map(|e| e.ok())
            {
                if cancel_token.is_cancelled() || Instant::now() > deadline {
                    return results;
                }

                if !entry.file_type().is_file() {
                    continue;
                }

                let path = entry.path();
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    if !self.extensions.contains(&ext_str) {
                        continue;
                    }
                } else {
                    continue;
                }

                // Check size limit (2 MB)
                if let Ok(meta) = entry.metadata() {
                    if meta.len() > MAX_FILE_SIZE {
                        continue;
                    }
                }

                if let Some(snippet) = search_file_for_snippet(path, &q_lower) {
                    let file_name = path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "File".into());

                    results.push(SearchResultItem {
                        id: format!("content_{}", path.to_string_lossy()),
                        title: file_name,
                        subtitle: Some(snippet.clone()),
                        category: "Content".into(),
                        item_type: "file".into(),
                        card_type: Some("snippet".into()),
                        path: Some(path.to_string_lossy().to_string()),
                        action_hint: Some("↵ Open File".into()),
                        score: 75,
                        snippet: Some(snippet),
                        admin_path: None,
                    });

                    if results.len() >= MAX_RESULTS {
                        return results;
                    }
                }
            }
        }

        results
    }
}

fn is_ignored_dir(path: &Path) -> bool {
    if let Some(name) = path.file_name() {
        let s = name.to_string_lossy();
        if s.starts_with('.') && s != "." {
            return true;
        }
        let lower = s.to_lowercase();
        if lower == "node_modules"
            || lower == "__pycache__"
            || lower == "venv"
            || lower == ".venv"
            || lower == "target"
            || lower == "dist"
            || lower == "build"
            || lower == "env"
        {
            return true;
        }
    }
    false
}

fn search_file_for_snippet(path: &Path, q_lower: &str) -> Option<String> {
    let file = File::open(path).ok()?;
    let reader = BufReader::new(file);

    for (idx, line) in reader.lines().enumerate() {
        if let Ok(line_text) = line {
            if line_text.to_lowercase().contains(q_lower) {
                let trimmed = line_text.trim();
                let snippet = if trimmed.len() > 100 {
                    format!("{}...", &trimmed[..97])
                } else {
                    trimmed.to_string()
                };
                return Some(format!("line {}: {}", idx + 1, snippet));
            }
        }
    }
    None
}
