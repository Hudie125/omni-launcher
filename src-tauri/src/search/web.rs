use super::SearchResultItem;
use std::time::Duration;

pub async fn fetch_web_suggestions(query: &str) -> Vec<SearchResultItem> {
    let q = query.trim();
    if q.is_empty() {
        return Vec::new();
    }

    let mut items = Vec::new();

    // Default primary search action
    items.push(SearchResultItem {
        id: format!("web_search_{}", q),
        title: format!("Search Google for \"{}\"", q),
        subtitle: Some("Open in default web browser".into()),
        category: "Web & AI Search".into(),
        item_type: "web".into(),
        card_type: None,
        path: Some(format!("https://www.google.com/search?q={}", urlencoding::encode(q))),
        action_hint: Some("↵ Search".into()),
        score: 150,
    });

    // Fetch live suggestions with 300ms timeout
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(350))
        .build();

    if let Ok(client) = client {
        let url = format!(
            "https://suggestqueries.google.com/complete/search?client=chrome&q={}",
            urlencoding::encode(q)
        );

        if let Ok(resp) = client.get(&url).send().await {
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                if let Some(arr) = json.get(1).and_then(|v| v.as_array()) {
                    for sug in arr.iter().take(4) {
                        if let Some(sug_str) = sug.as_str() {
                            if sug_str.to_lowercase() != q.to_lowercase() {
                                items.push(SearchResultItem {
                                    id: format!("sug_{}", sug_str),
                                    title: sug_str.to_string(),
                                    subtitle: Some("Live web suggestion".into()),
                                    category: "Web Suggestions".into(),
                                    item_type: "web".into(),
                                    card_type: None,
                                    path: Some(format!(
                                        "https://www.google.com/search?q={}",
                                        urlencoding::encode(sug_str)
                                    )),
                                    action_hint: Some("↵ Open".into()),
                                    score: 120,
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
