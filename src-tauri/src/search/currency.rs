use super::SearchResultItem;
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

static RATE_CACHE: OnceLock<Mutex<HashMap<String, (Instant, HashMap<String, f64>)>>> = OnceLock::new();

fn get_cache() -> &'static Mutex<HashMap<String, (Instant, HashMap<String, f64>)>> {
    RATE_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub struct CurrencyConverter;

impl CurrencyConverter {
    pub async fn convert(query: &str) -> Option<SearchResultItem> {
        let (amt, from, to) = Self::parse_query(query)?;

        let rate = Self::fetch_rate(&from, &to).await?;
        let result = amt * rate;

        let title = format!("{:.2} {} = {:.2} {}", amt, from, result, to);
        let subtitle = format!("1 {} = {:.4} {} (exchangerate-api)", from, rate, to);

        Some(SearchResultItem {
            id: format!("currency_{}_{}_{}", amt, from, to),
            title,
            subtitle: Some(subtitle),
            category: "Currency".into(),
            item_type: "calc".into(),
            card_type: Some("currency".into()),
            path: None,
            action_hint: Some("↵ Copy Result".into()),
            score: 1150,
            snippet: None,
            admin_path: None,
        })
    }

    fn parse_query(query: &str) -> Option<(f64, String, String)> {
        let re = Regex::new(
            r"(?i)^\s*([\d\.,]+)\s*([a-zA-Zа-яА-Я]{3,10})\s+(?:to|in|в)\s+([a-zA-Zа-яА-Я]{3,10})\s*$",
        )
        .ok()?;

        let caps = re.captures(query)?;
        let amt_str = caps.get(1)?.as_str().replace(',', ".");
        let amt: f64 = amt_str.parse().ok()?;

        let from_raw = caps.get(2)?.as_str().to_lowercase();
        let to_raw = caps.get(3)?.as_str().to_lowercase();

        let from = normalize_currency(&from_raw)?;
        let to = normalize_currency(&to_raw)?;

        Some((amt, from, to))
    }

    async fn fetch_rate(from: &str, to: &str) -> Option<f64> {
        let cache = get_cache();
        {
            if let Ok(guard) = cache.lock() {
                if let Some((ts, rates)) = guard.get(from) {
                    if ts.elapsed() < Duration::from_secs(3600) {
                        if let Some(&r) = rates.get(to) {
                            return Some(r);
                        }
                    }
                }
            }
        }

        let url = format!("https://open.er-api.com/v6/latest/{}", from);
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(4))
            .build()
            .ok()?;

        let res = client.get(&url).send().await.ok()?;
        let json: Value = res.json().await.ok()?;

        let rates_obj = json.get("rates")?.as_object()?;
        let mut map = HashMap::new();
        for (k, v) in rates_obj {
            if let Some(r) = v.as_f64() {
                map.insert(k.to_uppercase(), r);
            }
        }

        let rate = map.get(to).copied();

        if let Ok(mut guard) = cache.lock() {
            guard.insert(from.to_string(), (Instant::now(), map));
        }

        rate
    }
}

fn normalize_currency(code: &str) -> Option<String> {
    let s = match code {
        "usd" | "dollar" | "dollars" | "доллар" | "доллара" | "долларов" | "бакс" | "баксов" => {
            "USD"
        }
        "eur" | "euro" | "евро" => "EUR",
        "rub" | "ruble" | "rubles" | "рубль" | "рубля" | "рублей" | "руб" => "RUB",
        "gbp" | "pound" | "pounds" | "фунт" | "фунтов" => "GBP",
        "cny" | "yuan" | "юань" | "юаней" => "CNY",
        "jpy" | "yen" | "иена" | "йена" => "JPY",
        "kzt" | "tenge" | "тенге" => "KZT",
        "uah" | "hryvnia" | "гривна" | "гривны" | "гривен" => "UAH",
        "byn" | "белрубль" => "BYN",
        "try" | "lira" | "лира" | "лиры" => "TRY",
        "chf" | "franc" | "франк" => "CHF",
        "pln" | "zloty" | "злотый" => "PLN",
        other if other.len() == 3 => return Some(other.to_uppercase()),
        _ => return None,
    };
    Some(s.to_string())
}
