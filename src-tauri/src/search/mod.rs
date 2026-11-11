pub mod ai;
pub mod apps;
pub mod calc;
pub mod files;
pub mod notes;
pub mod web;

use apps::AppIndex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultItem {
    pub id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub category: String,
    pub item_type: String, // "app", "file", "calc", "ai", "cmd", "note", "web"
    pub card_type: Option<String>, // "math", "ai_response"
    pub path: Option<String>,
    pub action_hint: Option<String>,
    pub score: i32,
}

pub struct SearchCoordinator {
    pub app_index: Arc<AppIndex>,
    current_cancel_token: Mutex<Option<CancellationToken>>,
}

impl SearchCoordinator {
    pub fn new() -> Self {
        Self {
            app_index: Arc::new(AppIndex::new()),
            current_cancel_token: Mutex::new(None),
        }
    }

    pub async fn search(
        &self,
        query: &str,
        search_dirs: &[String],
        exclude_patterns: &[String],
        ollama_model: &str,
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

        let mut results = Vec::new();

        // 2. Calculator & Unit Converter (Instant)
        if let Some(calc_res) = calc::evaluate_calc_or_convert(q) {
            results.push(calc_res);
        }

        // 3. Scratchpad / Quick Notes
        let note_items = notes::handle_notes(q);
        results.extend(note_items);

        // 4. System Commands check
        if let Some(cmd_item) = check_system_command_match(q) {
            results.push(cmd_item);
        }

        // 5. Applications Search
        let app_results = self.app_index.search(q);
        results.extend(app_results);

        // 6. AI Assistant Queries (/ai or question words)
        if q.starts_with("/ai ") || is_ai_question(q) {
            let ai_items = ai::query_ai(q, ollama_model).await;
            results.extend(ai_items);
        }

        // 7. Filesystem Search (async with cancellation token)
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

        // 8. Live Web Suggestions (if not a pure math/system/note command)
        if !q.is_empty() && results.len() < 12 && !cancel_token.is_cancelled() {
            let web_items = web::fetch_web_suggestions(q).await;
            if !cancel_token.is_cancelled() {
                results.extend(web_items);
            }
        }

        // Sort items by score descending
        results.sort_by(|a, b| b.score.cmp(&a.score));
        results
    }
}

fn check_system_command_match(q: &str) -> Option<SearchResultItem> {
    let lower = q.to_lowercase();
    let cmd = lower.trim();

    match cmd {
        "lock" => Some(SearchResultItem {
            id: "sys_lock".into(),
            title: "Lock Screen".into(),
            subtitle: Some("Lock computer workstation".into()),
            category: "System Commands".into(),
            item_type: "cmd".into(),
            card_type: None,
            path: Some("cmd://lock".into()),
            action_hint: Some("↵ Lock".into()),
            score: 900,
        }),
        "sleep" | "suspend" => Some(SearchResultItem {
            id: "sys_sleep".into(),
            title: "Sleep Mode".into(),
            subtitle: Some("Put computer into low power suspend mode".into()),
            category: "System Commands".into(),
            item_type: "cmd".into(),
            card_type: None,
            path: Some("cmd://sleep".into()),
            action_hint: Some("↵ Sleep".into()),
            score: 900,
        }),
        "restart" | "reboot" => Some(SearchResultItem {
            id: "sys_restart".into(),
            title: "Restart System".into(),
            subtitle: Some("Reboot operating system immediately".into()),
            category: "System Commands".into(),
            item_type: "cmd".into(),
            card_type: None,
            path: Some("cmd://restart".into()),
            action_hint: Some("↵ Reboot".into()),
            score: 900,
        }),
        "shutdown" | "poweroff" => Some(SearchResultItem {
            id: "sys_shutdown".into(),
            title: "Shutdown System".into(),
            subtitle: Some("Turn off the computer".into()),
            category: "System Commands".into(),
            item_type: "cmd".into(),
            card_type: None,
            path: Some("cmd://shutdown".into()),
            action_hint: Some("↵ Shutdown".into()),
            score: 900,
        }),
        "empty trash" | "empty bin" => Some(SearchResultItem {
            id: "sys_empty_trash".into(),
            title: "Empty Trash / Recycle Bin".into(),
            subtitle: Some("Permanently delete recycled items".into()),
            category: "System Commands".into(),
            item_type: "cmd".into(),
            card_type: None,
            path: Some("cmd://empty trash".into()),
            action_hint: Some("↵ Empty".into()),
            score: 900,
        }),
        _ if cmd.starts_with("kill ") => Some(SearchResultItem {
            id: "sys_kill".into(),
            title: format!("Kill Process: {}", &cmd[5..]),
            subtitle: Some("Terminate matching processes".into()),
            category: "System Commands".into(),
            item_type: "cmd".into(),
            card_type: None,
            path: Some(format!("cmd://{}", cmd)),
            action_hint: Some("↵ Terminate".into()),
            score: 950,
        }),
        _ => None,
    }
}

fn is_ai_question(q: &str) -> bool {
    let lower = q.to_lowercase();
    lower.starts_with("what is")
        || lower.starts_with("how to")
        || lower.starts_with("why is")
        || lower.starts_with("who is")
        || lower.starts_with("explain ")
}
