use super::SearchResultItem;
use std::time::Duration;

pub async fn query_ai(query: &str, preferred_model: &str) -> Vec<SearchResultItem> {
    let q = query.trim();
    if q.is_empty() {
        return Vec::new();
    }

    let is_explicit_ai = q.starts_with(">ask ")
        || q.starts_with("? ")
        || q.starts_with("/ai ")
        || q.starts_with("?ai ");

    let prompt = if q.starts_with(">ask ") {
        q[5..].trim()
    } else if q.starts_with("? ") {
        q[2..].trim()
    } else if q.starts_with("/ai ") {
        q[4..].trim()
    } else if q.starts_with("?ai ") {
        q[4..].trim()
    } else {
        q
    };

    let mut items = Vec::new();
    let prompt_encoded = url_encode(prompt);

    // If explicit AI command, try local LLM (Ollama) or API first
    if is_explicit_ai && !prompt.is_empty() {
        if let Some(ollama_resp) = query_local_ollama(prompt, preferred_model).await {
            items.push(SearchResultItem {
                id: format!("ollama_{}", prompt),
                title: ollama_resp,
                subtitle: Some(format!("Ollama AI • Model: {}", preferred_model)),
                category: "AI Assistant".into(),
                item_type: "ai".into(),
                card_type: Some("ai_response".into()),
                path: None,
                action_hint: Some("↵ Copy Answer".into()),
                score: 2500,
                snippet: None,
                admin_path: None,
            });
        }
    }

    // Direct Web AI assistant shortcuts
    items.push(SearchResultItem {
        id: format!("ai_gemini_{}", prompt),
        title: format!("Gemini: \"{}\"", prompt),
        subtitle: Some("Launch Google Gemini in web browser".into()),
        category: "Web & AI".into(),
        item_type: "ai".into(),
        card_type: None,
        path: Some(format!("https://gemini.google.com/app?q={}", prompt_encoded)),
        action_hint: Some("↵ Ask Gemini".into()),
        score: if is_explicit_ai { 1500 } else { 160 },
        snippet: None,
        admin_path: None,
    });

    items.push(SearchResultItem {
        id: format!("ai_chatgpt_{}", prompt),
        title: format!("ChatGPT: \"{}\"", prompt),
        subtitle: Some("Launch OpenAI ChatGPT in web browser".into()),
        category: "Web & AI".into(),
        item_type: "ai".into(),
        card_type: None,
        path: Some(format!("https://chatgpt.com/?q={}", prompt_encoded)),
        action_hint: Some("↵ Ask ChatGPT".into()),
        score: if is_explicit_ai { 1450 } else { 155 },
        snippet: None,
        admin_path: None,
    });

    items.push(SearchResultItem {
        id: format!("ai_claude_{}", prompt),
        title: format!("Claude: \"{}\"", prompt),
        subtitle: Some("Launch Anthropic Claude AI in browser".into()),
        category: "Web & AI".into(),
        item_type: "ai".into(),
        card_type: None,
        path: Some(format!("https://claude.ai/new?q={}", prompt_encoded)),
        action_hint: Some("↵ Ask Claude".into()),
        score: if is_explicit_ai { 1400 } else { 150 },
        snippet: None,
        admin_path: None,
    });

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
