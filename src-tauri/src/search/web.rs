use super::SearchResultItem;
use std::time::Duration;

pub async fn fetch_web_suggestions(query: &str) -> Vec<SearchResultItem> {
    let q = query.trim();
    if q.is_empty() {
        return Vec::new();
    }

    let mut items = Vec::new();
    let q_encoded = urlencoding::encode(q);

    // 1. Google search
    items.push(SearchResultItem {
        id: format!("web_search_{}", q),
        title: format!("Google: \"{}\"", q),
        subtitle: Some("Search the web via Google".into()),
        category: "Web & AI".into(),
        item_type: "web".into(),
        card_type: None,
        path: Some(format!("https://www.google.com/search?q={}", q_encoded)),
        action_hint: Some("↵ Search".into()),
        score: 160,
        snippet: None,
        admin_path: None,
    });

    // 2. Google Gemini
    items.push(SearchResultItem {
        id: format!("ai_gemini_{}", q),
        title: format!("Gemini: \"{}\"", q),
        subtitle: Some("Ask Google Gemini AI in browser".into()),
        category: "Web & AI".into(),
        item_type: "ai".into(),
        card_type: None,
        path: Some(format!("https://gemini.google.com/app?q={}", q_encoded)),
        action_hint: Some("↵ Open Gemini".into()),
        score: 155,
        snippet: None,
        admin_path: None,
    });

    // 3. ChatGPT
    items.push(SearchResultItem {
        id: format!("ai_chatgpt_{}", q),
        title: format!("ChatGPT: \"{}\"", q),
        subtitle: Some("Ask OpenAI ChatGPT".into()),
        category: "Web & AI".into(),
        item_type: "ai".into(),
        card_type: None,
        path: Some(format!("https://chatgpt.com/?q={}", q_encoded)),
        action_hint: Some("↵ Open ChatGPT".into()),
        score: 150,
        snippet: None,
        admin_path: None,
    });

    // 4. Quick match for web service bookmarks/commands:
    let q_lower = q.to_lowercase();
    let web_services = [
        ("My IP Address", "ip", "https://2ip.ru", "Check external IP address"),
        ("YouTube", "yt", "https://www.youtube.com", "Video streaming platform"),
        ("Wikipedia", "wiki", "https://en.wikipedia.org", "Free online encyclopedia"),
        ("Gmail", "gmail", "https://mail.google.com", "Google email inbox"),
        ("Telegram Web", "tg", "https://web.telegram.org", "Telegram messenger web client"),
        ("GitHub", "gh", "https://github.com", "Developer code repository"),
        ("Claude AI", "claude", "https://claude.ai", "Anthropic Claude AI chat"),
        ("Steam", "steam", "steam://open/main", "Steam gaming client"),
        ("Spotify", "spotify", "spotify:", "Spotify music player"),
    ];

    for (name, key, url, desc) in web_services {
        if q_lower == key || q_lower == name.to_lowercase() || name.to_lowercase().contains(&q_lower) {
            items.push(SearchResultItem {
                id: format!("svc_{}", key),
                title: format!("{} (>{})", name, key),
                subtitle: Some(desc.into()),
                category: "Web & AI".into(),
                item_type: "web".into(),
                card_type: None,
                path: Some(url.into()),
                action_hint: Some("↵ Open".into()),
                score: 300,
                snippet: None,
                admin_path: None,
            });
        }
    }

    // 5. Fetch live Google suggestions
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(400))
        .build();

    if let Ok(client) = client {
        let url = format!(
            "https://suggestqueries.google.com/complete/search?client=chrome&q={}",
            q_encoded
        );

        if let Ok(resp) = client.get(&url).send().await {
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                if let Some(arr) = json.get(1).and_then(|v| v.as_array()) {
                    for sug in arr.iter().take(5) {
                        if let Some(sug_str) = sug.as_str() {
                            if sug_str.to_lowercase() != q_lower {
                                items.push(SearchResultItem {
                                    id: format!("sug_{}", sug_str),
                                    title: sug_str.to_string(),
                                    subtitle: Some("Google live suggestion".into()),
                                    category: "Web Suggestions".into(),
                                    item_type: "web".into(),
                                    card_type: None,
                                    path: Some(format!(
                                        "https://www.google.com/search?q={}",
                                        urlencoding::encode(sug_str)
                                    )),
                                    action_hint: Some("↵ Search".into()),
                                    score: 120,
                                    snippet: None,
                                    admin_path: None,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    items
}

mod urlencoding {
    pub fn encode(s: &str) -> String {
        let mut encoded = String::new();
        for b in s.bytes() {
            if b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.' || b == b'~' {
                encoded.push(b as char);
            } else {
                encoded.push_str(&format!("%{:02X}", b));
            }
        }
        encoded
    }
}
