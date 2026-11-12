use chrono::Local;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteItem {
    pub id: String,
    pub text: String,
    pub ts: String,
}

pub struct NotesStorage;

impl NotesStorage {
    fn file_path() -> PathBuf {
        let base_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        let dir = base_dir.join("Finder");
        let _ = fs::create_dir_all(&dir);
        dir.join("search_notes.json")
    }

    pub fn load_notes() -> Vec<NoteItem> {
        let path = Self::file_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(notes) = serde_json::from_str::<Vec<NoteItem>>(&content) {
                    return notes;
                }
            }
        }
        Vec::new()
    }

    pub fn save_notes(notes: &[NoteItem]) -> Result<(), String> {
        let path = Self::file_path();
        let content = serde_json::to_string_pretty(notes).map_err(|e| e.to_string())?;
        fs::write(path, content).map_err(|e| e.to_string())
    }

    pub fn add_note(text: &str) -> Result<NoteItem, String> {
        let mut notes = Self::load_notes();
        let now = Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        let item = NoteItem {
            id: format!("note_{}", Local::now().timestamp_millis()),
            text: text.trim().to_string(),
            ts: now,
        };
        notes.insert(0, item.clone());
        Self::save_notes(&notes)?;
        Ok(item)
    }

    pub fn edit_note(id: &str, new_text: &str) -> Result<(), String> {
        let mut notes = Self::load_notes();
        if let Some(n) = notes.iter_mut().find(|n| n.id == id) {
            n.text = new_text.trim().to_string();
            n.ts = Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
            Self::save_notes(&notes)?;
            Ok(())
        } else {
            Err("Note not found".into())
        }
    }

    pub fn delete_note(id: &str) -> Result<(), String> {
        let mut notes = Self::load_notes();
        notes.retain(|n| n.id != id);
        Self::save_notes(&notes)
    }

    pub fn delete_all_notes() -> Result<(), String> {
        Self::save_notes(&[])
    }

    pub fn export_notes_to_desktop() -> Result<String, String> {
        let notes = Self::load_notes();
        let desktop = dirs::desktop_dir().ok_or_else(|| "Desktop not found".to_string())?;
        let export_path = desktop.join("notes_export.txt");

        let mut out = String::new();
        for n in &notes {
            out.push_str(&format!("[{}] {}\n\n", n.ts, n.text));
        }

        fs::write(&export_path, out).map_err(|e| e.to_string())?;
        Ok(export_path.to_string_lossy().to_string())
    }
}
