use super::SearchResultItem;
use chrono::Local;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

pub fn handle_notes(query: &str) -> Vec<SearchResultItem> {
    let q = query.trim();

    // Check if saving a new note: "/note <text>"
    if q.starts_with("/note ") || q.starts_with("/n ") {
        let note_text = if q.starts_with("/note ") {
            &q[6..]
        } else {
            &q[3..]
        };

        if !note_text.is_empty() {
            return vec![SearchResultItem {
                id: "save_quick_note".into(),
                title: format!("Save Note: \"{}\"", note_text),
                subtitle: Some("Append note to local markdown scratchpad".into()),
                category: "Quick Notes".into(),
                item_type: "note".into(),
                card_type: None,
                path: Some(format!("note://{}", note_text)),
                action_hint: Some("↵ Save Note".into()),
                score: 1500,
            }];
        }
    }

    // If query is "/notes", show recent notes
    if q == "/notes" || q == "/note" {
        return read_recent_notes();
    }

    Vec::new()
}

pub fn save_note_to_file(note: &str) -> Result<(), String> {
    let notes_path = get_notes_path();
    if let Some(parent) = notes_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&notes_path)
        .map_err(|e| e.to_string())?;

    let now = Local::now().format("%Y-%m-%d %H:%M:%S");
    writeln!(file, "- **[{}]** {}", now, note).map_err(|e| e.to_string())?;

    Ok(())
}

fn read_recent_notes() -> Vec<SearchResultItem> {
    let notes_path = get_notes_path();
    if !notes_path.exists() {
        return vec![SearchResultItem {
            id: "notes_empty".into(),
            title: "No notes saved yet".into(),
            subtitle: Some("Type /note <text> to save an instant note".into()),
            category: "Quick Notes".into(),
            item_type: "note".into(),
            card_type: None,
            path: None,
            action_hint: None,
            score: 500,
        }];
    }

    let mut items = Vec::new();
    if let Ok(content) = std::fs::read_to_string(&notes_path) {
        let lines: Vec<&str> = content.lines().rev().take(10).collect();
        for (idx, line) in lines.into_iter().enumerate() {
            let clean_line = line.trim_start_matches("- ");
            items.push(SearchResultItem {
                id: format!("note_item_{}", idx),
                title: clean_line.to_string(),
                subtitle: Some("Saved note • Click to copy".into()),
                category: "Recent Notes".into(),
                item_type: "note".into(),
                card_type: None,
                path: Some(notes_path.to_string_lossy().to_string()),
                action_hint: Some("Tab Copy • ↵ Open Notes".into()),
                score: 500 - (idx as i32 * 10),
            });
        }
    }

    items
}

fn get_notes_path() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("omni-launcher").join("notes.md")
}
