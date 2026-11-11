use super::SearchResultItem;
use std::time::Duration;

pub async fn query_ai(query: &str, preferred_model: &str) -> Vec<SearchResultItem> {
    let q = query.trim();
    if q.is_empty() {
        return Vec::new();
    }

    let prompt = if q.starts_with("/ai ") {
        q[4..].trim()
    } else {
        q
    };

    let mut items = Vec::new();

    // 1. Direct Web AI links
    items.push(SearchResultItem {
        id: format!("ai_gemini_{}", prompt),
        title: format!("Ask Google Gemini: \"{}\"", prompt),
        subtitle: Some("Launch Gemini web assistant".into()),
        category: "AI Assistants".into(),
        item_type: "ai".into(),
        card_type: None,
        path: Some(format!("https://gemini.google.com/app?q={}", url_encode(prompt))),
        action_hint: Some("↵ Ask Gemini".into()),
        score: 350,
    });

    items.push(SearchResultItem {
        id: format!("ai_chatgpt_{}", prompt),
        title: format!("Ask ChatGPT: \"{}\"", prompt),
        subtitle: Some("Launch OpenAI ChatGPT".into()),
        category: "AI Assistants".into(),
        item_type: "ai".into(),
        card_type: None,
        path: Some(format!("https://chatgpt.com/?q={}", url_encode(prompt))),
        action_hint: Some("↵ Ask ChatGPT".into()),
        score: 340,
    });

    // 2. Query Local Ollama if user specifically typed "/ai <question>"
    if q.starts_with("/ai ") && !prompt.is_empty() {
        if let Some(ollama_resp) = query_local_ollama(prompt, preferred_model).await {
            items.insert(
                0,
                SearchResultItem {
                    id: format!("ollama_{}", prompt),
                    title: ollama_resp,
                    subtitle: Some(format!("Local Ollama • Model: {}", preferred_model)),
                    category: "Local LLM".into(),
                    item_type: "ai".into(),
                    card_type: Some("ai_response".into()),
                    path: None,
                    action_hint: Some("Tab Copy Answer".into()),
                    score: 2000,
                },
            );
        }
    }

    items
}

async fn query_local_ollama(prompt: &str, model: &str) -> Option<String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(12))
        .build()
        .ok()?;

    let target_model = if model == "auto" { "llama3" } else { model };

    let payload = serde_json::json!({
        "model": target_model,
        "prompt": prompt,
        "stream": false
    });

    let resp = client
        .post("http://127.0.0.1:11434/api/generate")
        .json(&payload)
        .send()
        .await
        .ok()?;

    if resp.status().is_success() {
        let json: serde_json::Value = resp.json().await.ok()?;
        return json.get("response").and_then(|v| v.as_str()).map(|s| s.to_string());
    }

    None
}

fn url_encode(s: &str) -> String {
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
