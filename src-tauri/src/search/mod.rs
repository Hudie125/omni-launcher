pub mod ai;
pub mod apps;
pub mod bookmarks;
pub mod calc;
pub mod content;
pub mod currency;
pub mod files;
pub mod provider;
pub mod settings_pages;
pub mod timer;
pub mod weather;
pub mod web;

use apps::AppIndex;
use content::ContentSearchEngine;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::AppHandle;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultItem {
    pub id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub category: String,
    pub item_type: String, // "app", "file", "window", "bookmark", "calc", "ai", "cmd", "note", "web"
    pub card_type: Option<String>, // "math", "ai_response", "weather", "currency", "snippet"
    pub path: Option<String>,
    pub action_hint: Option<String>,
    pub score: i32,
    #[serde(default)]
    pub snippet: Option<String>,
    #[serde(default)]
    pub admin_path: Option<String>,
}

pub struct SearchCoordinator {
    pub app_index: Arc<AppIndex>,
    pub content_engine: Arc<ContentSearchEngine>,
    current_cancel_token: Mutex<Option<CancellationToken>>,
}

impl SearchCoordinator {
    pub fn new() -> Self {
        Self {
            app_index: Arc::new(AppIndex::new()),
            content_engine: Arc::new(ContentSearchEngine::new()),
            current_cancel_token: Mutex::new(None),
        }
    }

    pub async fn search(
        &self,
        query: &str,
        search_dirs: &[String],
        exclude_patterns: &[String],
        ollama_model: &str,
        app_handle: Option<AppHandle>,
    ) -> Vec<SearchResultItem> {
        let q = query.trim();

        // 1. Cancel previous pending search
        let cancel_token = CancellationToken::new();
        {
            let mut lock = self.current_cancel_token.lock().await;
            if let Some(old_token) = lock.take() {
                old_token.cancel();
            }
            *lock = Some(cancel_token.clone());
        }

        // Empty query: Show FREQUENT top used + RECENT history
        if q.is_empty() {
            return self.get_empty_state_results();
        }

        let mut results = Vec::new();

        // 2. Calculator & Unit Converter & Random (Instant)
        if let Some(calc_res) = calc::evaluate_calc_or_convert(q) {
            results.push(calc_res);
        }

        // 3. Currency Conversion (100 usd to eur)
        if let Some(curr_res) = currency::CurrencyConverter::convert(q).await {
            results.push(curr_res);
        }

        // 4. Weather (>weather [city])
        if q.starts_with(">weather ") || q.starts_with("weather ") {
            let city = if q.starts_with(">weather ") {
                &q[9..]
            } else {
                &q[8..]
            };
            if let Some(wx_res) = weather::WeatherEngine::fetch_weather(city).await {
                results.push(wx_res);
            }
        }

        // 5. Timers & Reminders (>timer 5m, >remind 19:30)
        if let Some(timer_res) = timer::TimerManager::parse_command(q, app_handle) {
            results.push(timer_res);
        }

        // 6. Quick System Commands (>lock, >sleep, >reboot, >kill, etc.)
        let cmd_items = self.match_quick_commands(q);
        results.extend(cmd_items);

        // 7. Windows Settings Pages (ms-settings:...)
        let settings_items = settings_pages::SettingsPagesScanner::search(q);
        results.extend(settings_items);

        // 8. Chrome Bookmarks
        let bookmark_items = bookmarks::ChromeBookmarksScanner::search(q);
        results.extend(bookmark_items);

        // 9. Applications & Active Running Windows
        let app_results = self.app_index.search(q);
        results.extend(app_results);

        // 10. AI Assistant (>ask, ?, /ai, or questions)
        if q.starts_with(">ask ") || q.starts_with("? ") || q.starts_with("/ai ") || is_ai_question(q) {
            let ai_items = ai::query_ai(q, ollama_model).await;
            results.extend(ai_items);
        }

        // 11. Filesystem Search (filename crawling)
        if q.len() >= 2 && !q.starts_with('/') && !q.starts_with('>') {
            let file_items = files::search_filesystem(
                q,
                search_dirs,
                exclude_patterns,
                cancel_token.clone(),
            )
            .await;

            if !cancel_token.is_cancelled() {
                results.extend(file_items);
            }
        }

        // 12. File Content Search (grep inside files with snippet & line numbers)
        if q.len() >= 3 && !q.starts_with('/') && !q.starts_with('>') {
            let content_items = self.content_engine.search(q, &cancel_token);
            if !cancel_token.is_cancelled() {
                results.extend(content_items);
            }
        }

        // 13. Live Web Suggestions & Shortcuts
        if !q.starts_with('>') && results.len() < 12 {
            let web_items = web::fetch_web_suggestions(q).await;
            if !cancel_token.is_cancelled() {
                results.extend(web_items);
            }
        }

        // Sort by score descending
        results.sort_by(|a, b| b.score.cmp(&a.score));

        results
    }

    fn get_empty_state_results(&self) -> Vec<SearchResultItem> {
        let mut results = Vec::new();

        // 1. Top Frequent
        let top_used = crate::storage::history::HistoryStorage::get_top_used(5);
        for (key, count) in top_used {
            let label = if let Some(idx) = key.find('|') {
                &key[idx + 1..]
            } else {
                &key
            };
            results.push(SearchResultItem {
                id: format!("freq_{}", key),
                title: format!("{} ({} times)", label, count),
                subtitle: Some("Frequently used".into()),
                category: "Frequent".into(),
                item_type: "app".into(),
                card_type: None,
                path: Some(format!("search://{}", label)),
                action_hint: Some("↵ Run".into()),
                score: 500,
                snippet: None,
                admin_path: None,
            });
        }

        // 2. Recent History
        let history = crate::storage::history::HistoryStorage::load_history();
        for item in history.into_iter().take(6) {
            results.push(SearchResultItem {
                id: format!("hist_{}", item),
                title: item.clone(),
                subtitle: Some("Recent search query".into()),
                category: "Recent".into(),
                item_type: "web".into(),
                card_type: None,
                path: Some(format!("search://{}", item)),
                action_hint: Some("↵ Repeat Search".into()),
                score: 400,
                snippet: None,
                admin_path: None,
            });
        }

        // If no history, default to top apps
        if results.is_empty() {
            return self.app_index.search("");
        }

        results
    }

    fn match_quick_commands(&self, q: &str) -> Vec<SearchResultItem> {
        let q_clean = if q.starts_with('>') { &q[1..] } else { q };
        let q_lower = q_clean.trim().to_lowercase();
        if q_lower.is_empty() {
            return Vec::new();
        }

        let commands = [
            ("Lock PC", "lock", "Lock current workstation session", "System", "cmd://lock"),
            ("Sleep", "sleep", "Enter low-power sleep mode", "System", "cmd://sleep"),
            ("Reboot (10s)", "reboot", "Restart computer with 10s cancel delay", "System", "cmd://reboot"),
            ("Shutdown (10s)", "shutdown", "Power off computer with 10s cancel delay", "System", "cmd://shutdown"),
            ("Cancel Shutdown", "cancel", "Abort scheduled reboot or shutdown", "System", "cmd://cancel"),
            ("Empty Recycle Bin", "empty trash", "Permanently remove deleted files", "System", "cmd://empty trash"),
            ("Task Manager", "taskmgr", "Monitor running processes and performance", "System", "cmd://taskmgr"),
            ("Control Panel", "panel", "Legacy Windows control panel", "System", "cmd://panel"),
            ("Windows Settings", "settings", "Modern OS settings app", "System", "cmd://settings"),
            ("Snipping Tool", "snip", "Screen clipping and capture utility", "System", "cmd://snip"),
            ("PowerShell", "ps", "Windows PowerShell terminal", "System", "cmd://ps"),
            ("Windows Terminal", "wt", "Modern multi-tab terminal", "System", "cmd://wt"),
            ("Registry Editor", "regedit", "System registry configuration", "System", "cmd://regedit"),
            ("Services", "services", "Background Windows services", "System", "cmd://services"),
            ("Disk Management", "disk", "Manage partitions and hard drives", "System", "cmd://disk"),
            ("Device Manager", "devmgmt", "Hardware devices and drivers", "System", "cmd://devmgmt"),
            ("Event Viewer", "eventvwr", "System diagnostic logs", "System", "cmd://eventvwr"),
            ("System Info", "msinfo", "Detailed hardware and software specs", "System", "cmd://msinfo"),
            ("Paint", "paint", "Simple graphics editor", "System", "cmd://paint"),
            ("WordPad", "wordpad", "Rich text document editor", "System", "cmd://wordpad"),
            ("On-Screen Keyboard", "osk", "Virtual accessibility keyboard", "System", "cmd://osk"),
            ("Character Map", "charmap", "Special symbols and font glyphs", "System", "cmd://charmap"),
            ("Home Directory", "home", "User personal home folder", "Folders", "folder://home"),
            ("Downloads Folder", "downloads", "Browser downloads folder", "Folders", "folder://downloads"),
            ("Documents Folder", "docs", "Personal documents folder", "Folders", "folder://docs"),
            ("Pictures Folder", "pics", "User images and photos", "Folders", "folder://pics"),
            ("Videos Folder", "videos", "User recorded videos", "Folders", "folder://videos"),
            ("Desktop Folder", "desktop", "Desktop files directory", "Folders", "folder://desktop"),
            ("ChatGPT", "chatgpt", "OpenAI ChatGPT assistant", "AI", "https://chatgpt.com"),
            ("Gemini", "gemini", "Google Gemini AI", "AI", "https://gemini.google.com/app"),
            ("Claude AI", "claude", "Anthropic Claude assistant", "AI", "https://claude.ai"),
            ("DALL-E 3", "dalle", "OpenAI Image Generation", "AI", "https://chatgpt.com/?model=gpt-4o"),
            ("My Public IP", "ip", "Check your external IP address", "Web", "https://2ip.ru"),
            ("Google Search", "g", "Search with Google", "Web", "https://www.google.com"),
            ("YouTube", "yt", "Watch videos on YouTube", "Web", "https://www.youtube.com"),
            ("Wikipedia", "wiki", "The free encyclopedia", "Web", "https://en.wikipedia.org"),
            ("Gmail", "gmail", "Google Mail inbox", "Web", "https://mail.google.com"),
            ("Telegram Web", "tg", "Telegram messaging client", "Web", "https://web.telegram.org"),
            ("GitHub", "gh", "GitHub developer platform", "Web", "https://github.com"),
            ("Steam", "steam", "Steam gaming client", "Web", "steam://open/main"),
            ("Spotify", "spotify", "Spotify music streaming", "Web", "spotify:"),
            ("Clipboard History", "clip", "Search clipboard history", "Utils", "cmd://clip"),
        ];

        let mut matches = Vec::new();

        for (title, key, desc, cat, target) in commands {
            if q_lower == key || q_lower == title.to_lowercase() || title.to_lowercase().contains(&q_lower) {
                let path = match target {
                    "folder://home" => dirs::home_dir().map(|p| p.to_string_lossy().to_string()),
                    "folder://downloads" => dirs::download_dir().map(|p| p.to_string_lossy().to_string()),
                    "folder://docs" => dirs::document_dir().map(|p| p.to_string_lossy().to_string()),
                    "folder://pics" => dirs::picture_dir().map(|p| p.to_string_lossy().to_string()),
                    "folder://videos" => dirs::video_dir().map(|p| p.to_string_lossy().to_string()),
                    "folder://desktop" => dirs::desktop_dir().map(|p| p.to_string_lossy().to_string()),
                    _ => Some(target.to_string()),
                };

                let item_type = match cat {
                    "Web" | "AI" => "web".into(),
                    "Folders" => "file".into(),
                    _ => "cmd".into(),
                };

                matches.push(SearchResultItem {
                    id: format!("cmd_{}", key),
                    title: format!("{} (>{})", title, key),
                    subtitle: Some(desc.into()),
                    category: cat.into(),
                    item_type,
                    card_type: None,
                    path,
                    action_hint: Some(if cat == "Web" || cat == "AI" { "↵ Open in Browser".into() } else { "↵ Execute".into() }),
                    score: 950,
                    snippet: None,
                    admin_path: None,
                });
            }
        }

        matches
    }
}

fn is_ai_question(q: &str) -> bool {
    let lower = q.to_lowercase();
    lower.starts_with("who ")
        || lower.starts_with("what ")
        || lower.starts_with("how ")
        || lower.starts_with("why ")
        || lower.starts_with("when ")
        || lower.starts_with("where ")
        || lower.starts_with("кто ")
        || lower.starts_with("что ")
        || lower.starts_with("как ")
        || lower.starts_with("почему ")
        || lower.starts_with("зачем ")
}
