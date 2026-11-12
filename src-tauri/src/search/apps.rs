use super::SearchResultItem;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::path::PathBuf;
use std::sync::RwLock;

#[derive(Debug, Clone)]
pub struct AppEntry {
    pub name: String,
    pub exec: String,
    pub path: PathBuf,
    pub comment: Option<String>,
}

pub struct AppIndex {
    apps: RwLock<Vec<AppEntry>>,
    matcher: SkimMatcherV2,
}

impl AppIndex {
    pub fn new() -> Self {
        let index = Self {
            apps: RwLock::new(Vec::new()),
            matcher: SkimMatcherV2::default(),
        };
        index.refresh();
        index
    }

    pub fn refresh(&self) {
        let mut found = Vec::new();

        #[cfg(target_os = "windows")]
        {
            Self::scan_windows_apps(&mut found);
            Self::add_windows_system_tools(&mut found);
        }

        #[cfg(target_os = "linux")]
        {
            Self::scan_linux_desktop_entries(&mut found);
        }

        if let Ok(mut lock) = self.apps.write() {
            *lock = found;
        }
    }

    pub fn search(&self, query: &str) -> Vec<SearchResultItem> {
        let q = query.trim().to_lowercase();
        let apps = match self.apps.read() {
            Ok(guard) => guard,
            Err(_) => return Vec::new(),
        };

        let mut results = Vec::new();

        // 1. Search Active Windows (WinAPI EnumWindows)
        let active_windows = crate::system::windows::get_running_windows(&q);
        for win in active_windows {
            results.push(SearchResultItem {
                id: format!("win_{}", win.hwnd),
                title: format!("[Window] {}", win.title),
                subtitle: Some("Active running application window".into()),
                category: "Programs".into(),
                item_type: "window".into(),
                card_type: None,
                path: Some(format!("window://{}", win.hwnd)),
                action_hint: Some("↵ Switch Window".into()),
                score: 150,
                snippet: None,
                admin_path: None,
            });
        }

        if q.is_empty() {
            // Return top frequent/default apps
            let defaults: Vec<SearchResultItem> = apps
                .iter()
                .take(8)
                .map(|app| {
                    let path_str = app.path.to_string_lossy().to_string();
                    SearchResultItem {
                        id: format!("app_{}", app.name),
                        title: app.name.clone(),
                        subtitle: app.comment.clone().or_else(|| Some(app.exec.clone())),
                        category: "Programs".into(),
                        item_type: "app".into(),
                        card_type: None,
                        path: Some(path_str.clone()),
                        action_hint: Some("↵ Launch • Shift+↵ Admin".into()),
                        score: 100,
                        snippet: None,
                        admin_path: Some(path_str),
                    }
                })
                .collect();
            results.extend(defaults);
            return results;
        }

        let mut scored: Vec<(i64, &AppEntry)> = Vec::new();
        for app in apps.iter() {
            let name_lower = app.name.to_lowercase();
            // Prefix match bonus
            if name_lower.starts_with(&q) {
                scored.push((1000 + (name_lower.len() as i64 * -2), app));
            } else if let Some(score) = self.matcher.fuzzy_match(&app.name, &q) {
                scored.push((score, app));
            } else if let Some(comment) = &app.comment {
                if let Some(c_score) = self.matcher.fuzzy_match(comment, &q) {
                    scored.push((c_score - 50, app));
                }
            }
        }

        scored.sort_by(|a, b| b.0.cmp(&a.0));

        let matched_apps = scored.into_iter().take(12).map(|(score, app)| {
            let path_str = app.path.to_string_lossy().to_string();
            SearchResultItem {
                id: format!("app_{}", app.name),
                title: app.name.clone(),
                subtitle: app.comment.clone().or_else(|| Some(app.exec.clone())),
                category: "Programs".into(),
                item_type: "app".into(),
                card_type: None,
                path: Some(path_str.clone()),
                action_hint: Some("↵ Launch • Shift+↵ Admin".into()),
                score: score as i32,
                snippet: None,
                admin_path: Some(path_str),
            }
        });

        results.extend(matched_apps);
        results
    }

    #[cfg(target_os = "windows")]
    fn scan_windows_apps(out: &mut Vec<AppEntry>) {
        let mut search_dirs = Vec::new();
        if let Ok(app_data) = std::env::var("APPDATA") {
            search_dirs.push(PathBuf::from(app_data).join(r"Microsoft\Windows\Start Menu\Programs"));
        }
        if let Ok(program_data) = std::env::var("ProgramData") {
            search_dirs
                .push(PathBuf::from(program_data).join(r"Microsoft\Windows\Start Menu\Programs"));
        }

        for dir in search_dirs {
            if !dir.exists() {
                continue;
            }
            for entry in walkdir::WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
                let p = entry.path();
                if let Some(ext) = p.extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    if ext_str == "lnk" || ext_str == "url" {
                        let name = p
                            .file_stem()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_default();
                        if !name.is_empty() {
                            out.push(AppEntry {
                                name,
                                exec: p.to_string_lossy().to_string(),
                                path: p.to_path_buf(),
                                comment: None,
                            });
                        }
                    }
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    fn add_windows_system_tools(out: &mut Vec<AppEntry>) {
        let tools = [
            ("Command Prompt", "cmd.exe", "Windows command interpreter"),
            ("Task Manager", "taskmgr.exe", "View running processes and performance"),
            ("Calculator", "calc.exe", "System arithmetic calculator"),
            ("Notepad", "notepad.exe", "Simple plain text editor"),
            ("Settings", "ms-settings:", "System settings and preferences"),
            ("Explorer", "explorer.exe", "File management and disk navigator"),
            ("Snipping Tool", "snippingtool.exe", "Screen capture utility"),
            ("Recycle Bin", "shell:RecycleBinFolder", "Deleted files and restore bin"),
            ("PowerShell", "powershell.exe", "Windows PowerShell automation console"),
            ("Windows Terminal", "wt.exe", "Modern multi-tab terminal console"),
            ("Registry Editor", "regedit.exe", "Windows registry management"),
            ("Services", "services.msc", "Windows background services manager"),
            ("Disk Management", "diskmgmt.msc", "Partition and disk drive tool"),
            ("Device Manager", "devmgmt.msc", "Hardware drivers and peripherals"),
            ("Event Viewer", "eventvwr.msc", "System diagnostics and event logs"),
            ("System Info", "msinfo32.exe", "Hardware specifications and OS info"),
            ("Paint", "mspaint.exe", "Raster graphics editor"),
            ("WordPad", "wordpad.exe", "Rich text document editor"),
            ("Character Map", "charmap.exe", "Unicode symbols and glyphs"),
            ("On-Screen Keyboard", "osk.exe", "Virtual accessibility keyboard"),
        ];

        for (name, exec, comment) in tools {
            out.push(AppEntry {
                name: name.to_string(),
                exec: exec.to_string(),
                path: PathBuf::from(exec),
                comment: Some(comment.to_string()),
            });
        }
    }

    #[cfg(target_os = "linux")]
    fn scan_linux_desktop_entries(out: &mut Vec<AppEntry>) {
        let mut dirs = vec![
            PathBuf::from("/usr/share/applications"),
            PathBuf::from("/usr/local/share/applications"),
        ];
        if let Some(home) = dirs::home_dir() {
            dirs.push(home.join(".local/share/applications"));
        }

        for dir in dirs {
            if !dir.exists() {
                continue;
            }
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|e| e.to_str()) == Some("desktop") {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if let Some(app) = parse_desktop_entry(&path, &content) {
                                out.push(app);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(target_os = "linux")]
fn parse_desktop_entry(path: &std::path::Path, content: &str) -> Option<AppEntry> {
    let mut name = None;
    let mut exec = None;
    let mut comment = None;
    let mut no_display = false;

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("Name=") && name.is_none() {
            name = Some(line["Name=".len()..].to_string());
        } else if line.starts_with("Exec=") && exec.is_none() {
            exec = Some(line["Exec=".len()..].to_string());
        } else if line.starts_with("Comment=") && comment.is_none() {
            comment = Some(line["Comment=".len()..].to_string());
        } else if line == "NoDisplay=true" {
            no_display = true;
        }
    }

    if no_display {
        return None;
    }

    if let (Some(name), Some(exec)) = (name, exec) {
        let clean_exec = exec.split_whitespace().next().unwrap_or(&exec).to_string();
        Some(AppEntry {
            name,
            exec: clean_exec,
            path: path.to_path_buf(),
            comment,
        })
    } else {
        None
    }
}
