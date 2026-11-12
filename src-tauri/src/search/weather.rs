use super::SearchResultItem;
use serde_json::Value;
use std::time::Duration;

pub struct WeatherEngine;

impl WeatherEngine {
    pub async fn fetch_weather(city: &str) -> Option<SearchResultItem> {
        let city_clean = city.trim();
        let target_city = if city_clean.is_empty() {
            "Moscow"
        } else {
            city_clean
        };

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .ok()?;

        // 1. Geocode
        let geo_url = format!(
            "https://geocoding-api.open-meteo.com/v1/search?name={}&count=1&language=en&format=json",
            url_encode(target_city)
        );

        let geo_res = client.get(&geo_url).send().await.ok()?;
        let geo_json: Value = geo_res.json().await.ok()?;

        let result = geo_json.get("results")?.as_array()?.first()?;
        let name = result.get("name")?.as_str().unwrap_or(target_city);
        let country = result.get("country").and_then(|c| c.as_str()).unwrap_or("");
        let lat = result.get("latitude")?.as_f64()?;
        let lon = result.get("longitude")?.as_f64()?;

        // 2. Weather forecast
        let wx_url = format!(
            "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,wind_speed_10m,weather_code",
            lat, lon
        );

        let wx_res = client.get(&wx_url).send().await.ok()?;
        let wx_json: Value = wx_res.json().await.ok()?;

        let cur = wx_json.get("current")?;
        let temp = cur.get("temperature_2m")?.as_f64()?;
        let wind = cur.get("wind_speed_10m").and_then(|w| w.as_f64()).unwrap_or(0.0);
        let code = cur.get("weather_code").and_then(|c| c.as_i64()).unwrap_or(0);

        let desc = weather_code_to_desc(code);

        let title = format!("{}: {:.1}°C", name, temp);
        let subtitle = format!("{}, Wind {:.1} km/h • {}, {}", desc, wind, name, country);

        Some(SearchResultItem {
            id: format!("weather_{}", name),
            title,
            subtitle: Some(subtitle),
            category: "Weather".into(),
            item_type: "calc".into(),
            card_type: Some("weather".into()),
            path: None,
            action_hint: Some("↵ Copy Weather Info".into()),
            score: 1150,
            snippet: None,
            admin_path: None,
        })
    }
}

fn url_encode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

fn weather_code_to_desc(code: i64) -> &'static str {
    match code {
        0 => "☀️ Clear sky",
        1..=3 => "⛅ Partly cloudy",
        45 | 48 => "🌫️ Foggy",
        51..=55 => "🌧️ Drizzle",
        61..=65 => "🌧️ Rain",
        71..=75 => "❄️ Snowfall",
        80..=82 => "🌦️ Rain showers",
        95..=99 => "⛈️ Thunderstorm",
        _ => "🌡️ Weather",
    }
}
