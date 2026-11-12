use super::SearchResultItem;
use std::path::Path;
use tokio_util::sync::CancellationToken;

pub async fn search_filesystem(
    query: &str,
    search_dirs: &[String],
    exclude_patterns: &[String],
    cancel_token: CancellationToken,
) -> Vec<SearchResultItem> {
    let q = query.trim().to_lowercase();
    if q.len() < 2 {
        return Vec::new();
    }

    let search_dirs_owned: Vec<String> = search_dirs.to_vec();
    let excludes_owned: Vec<String> = exclude_patterns.to_vec();

    // Run directory crawling in a blocking thread pool
    tokio::task::spawn_blocking(move || {
        let mut results = Vec::new();

        for root_str in &search_dirs_owned {
            if cancel_token.is_cancelled() || results.len() >= 25 {
                break;
            }

            let root = Path::new(root_str);
            if !root.exists() {
                continue;
            }

            let mut builder = ignore::WalkBuilder::new(root);
            builder.hidden(true); // Skip hidden files/folders by default
            builder.git_ignore(true);
            builder.git_global(true);
            builder.git_exclude(true);
            builder.max_depth(Some(6));

            for result in builder.build() {
                if cancel_token.is_cancelled() || results.len() >= 25 {
                    break;
                }

                let entry = match result {
                    Ok(e) => e,
                    Err(_) => continue,
                };

                let path = entry.path();
                let path_str = path.to_string_lossy();

                // Check user exclude patterns
                if excludes_owned.iter().any(|pattern| path_str.contains(pattern)) {
                    continue;
                }

                if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                    let file_name_lower = file_name.to_lowercase();
                    if file_name_lower.contains(&q) {
                        let is_dir = path.is_dir();
                        let size_str = if is_dir {
                            "Folder".to_string()
                        } else if let Ok(meta) = entry.metadata() {
                            format_file_size(meta.len())
                        } else {
                            "File".to_string()
                        };

                        results.push(SearchResultItem {
                            id: format!("file_{}", path_str),
                            title: file_name.to_string(),
                            subtitle: Some(format!("{} • {}", size_str, path_str)),
                            category: "Files".into(),
                            item_type: "file".into(),
                            card_type: None,
                            path: Some(path_str.to_string()),
                            action_hint: Some("↵ Open • Tab Copy".into()),
                            score: if file_name_lower.starts_with(&q) { 500 } else { 300 },
                            snippet: None,
                            admin_path: None,
                        });
                    }
                }
            }
        }

        results
    })
    .await
    .unwrap_or_default()
}

fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
