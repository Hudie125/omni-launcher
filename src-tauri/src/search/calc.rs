use super::SearchResultItem;
use rand::Rng;
use regex::Regex;

pub fn evaluate_calc_or_convert(query: &str) -> Option<SearchResultItem> {
    let q = query.trim();
    if q.is_empty() {
        return None;
    }

    // 1. Password generator: "pw", "pw 16", "pw 24", "pw 32"
    if let Some(item) = try_password_gen(q) {
        return Some(item);
    }

    // 2. Random tools: "rand", "dice", "coin"
    if let Some(item) = try_random_tools(q) {
        return Some(item);
    }

    // 3. Unit conversion: "<num> <from> to <to>"
    if let Some(item) = try_unit_conversion(q) {
        return Some(item);
    }

    // 4. Custom rounding math: rnd2(10/3) or rnd 3 100/7
    if let Some(item) = try_custom_round_calc(q) {
        return Some(item);
    }

    // 5. Standard arithmetic expression
    try_standard_math(q)
}

fn try_password_gen(q: &str) -> Option<SearchResultItem> {
    let lower = q.to_lowercase();
    if lower == "pw" || lower.starts_with("pw ") || lower.starts_with("pw") {
        let length: usize = if lower == "pw" {
            16
        } else if lower.starts_with("pw ") {
            lower[3..].trim().parse().unwrap_or(16)
        } else if let Ok(l) = lower[2..].trim().parse::<usize>() {
            l
        } else {
            return None;
        };

        let len = length.clamp(4, 128);
        let charset = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()-_=+";
        let mut rng = rand::thread_rng();
        let password: String = (0..len)
            .map(|_| {
                let idx = rng.gen_range(0..charset.len());
                charset[idx] as char
            })
            .collect();

        return Some(SearchResultItem {
            id: format!("pw_{}", len),
            title: password.clone(),
            subtitle: Some(format!("Generated {}-character secure password (↵ Copy)", len)),
            category: "Utils".into(),
            item_type: "calc".into(),
            card_type: Some("math".into()),
            path: None,
            action_hint: Some("↵ Copy Password".into()),
            score: 1100,
            snippet: None,
            admin_path: None,
        });
    }
    None
}

fn try_random_tools(q: &str) -> Option<SearchResultItem> {
    let lower = q.to_lowercase();
    let mut rng = rand::thread_rng();

    if lower == "rand" || lower == "random" {
        let val: i32 = rng.gen_range(1..=100);
        return Some(SearchResultItem {
            id: "rand_100".into(),
            title: val.to_string(),
            subtitle: Some("Random number (1 - 100)".into()),
            category: "Utils".into(),
            item_type: "calc".into(),
            card_type: Some("math".into()),
            path: None,
            action_hint: Some("↵ Copy".into()),
            score: 1100,
            snippet: None,
            admin_path: None,
        });
    }

    if lower == "dice" {
        let val: i32 = rng.gen_range(1..=6);
        return Some(SearchResultItem {
            id: "rand_dice".into(),
            title: format!("🎲 {}", val),
            subtitle: Some("Dice roll (1 - 6)".into()),
            category: "Utils".into(),
            item_type: "calc".into(),
            card_type: Some("math".into()),
            path: None,
            action_hint: Some("↵ Copy".into()),
            score: 1100,
            snippet: None,
            admin_path: None,
        });
    }

    if lower == "coin" {
        let val = if rng.gen_bool(0.5) { "Heads" } else { "Tails" };
        return Some(SearchResultItem {
            id: "rand_coin".into(),
            title: format!("🪙 {}", val),
            subtitle: Some("Coin flip result".into()),
            category: "Utils".into(),
            item_type: "calc".into(),
            card_type: Some("math".into()),
            path: None,
            action_hint: Some("↵ Copy".into()),
            score: 1100,
            snippet: None,
            admin_path: None,
        });
    }

    None
}

fn try_custom_round_calc(q: &str) -> Option<SearchResultItem> {
    // rnd2(10/3) or rnd 3 100/7
    let re_p = Regex::new(r"(?i)^\s*rnd\s*(\d{1,3})\s*\(\s*(.+?)\s*\)\s*$").ok()?;
    let re_s = Regex::new(r"(?i)^\s*rnd\s*(\d{1,3})\s+(.+?)\s*$").ok()?;

    let (digits, expr) = if let Some(caps) = re_p.captures(q) {
        let d: usize = caps.get(1)?.as_str().parse().ok()?;
        let e = caps.get(2)?.as_str().to_string();
        (d, e)
    } else if let Some(caps) = re_s.captures(q) {
        let d: usize = caps.get(1)?.as_str().parse().ok()?;
        let e = caps.get(2)?.as_str().to_string();
        (d, e)
    } else {
        return None;
    };

    let sanitized = sanitize_math(&expr);
    let val = evalexpr::eval_number(&sanitized).ok()?;

    let res_str = format!("{:.digits$}", val, digits = digits);

    Some(SearchResultItem {
        id: format!("rnd_calc_{}", q),
        title: res_str,
        subtitle: Some(format!("= {} (rounded to {} dec)", expr, digits)),
        category: "Calculator".into(),
        item_type: "calc".into(),
        card_type: Some("math".into()),
        path: None,
        action_hint: Some("↵ Copy Result".into()),
        score: 1200,
        snippet: None,
        admin_path: None,
    })
}

fn try_standard_math(q: &str) -> Option<SearchResultItem> {
    let has_math = q.chars().any(|c| "+-*/^%()".contains(c))
        || q.starts_with("sin(")
        || q.starts_with("cos(")
        || q.starts_with("sqrt(")
        || q.starts_with("tan(");

    if !has_math {
        return None;
    }

    let sanitized = sanitize_math(q);
    match evalexpr::eval(&sanitized) {
        Ok(value) => {
            let res_str = match value {
                evalexpr::Value::Float(f) => {
                    if f.fract() == 0.0 {
                        format!("{:.0}", f)
                    } else {
                        format!("{:.8}", f)
                            .trim_end_matches('0')
                            .trim_end_matches('.')
                            .to_string()
                    }
                }
                evalexpr::Value::Int(i) => i.to_string(),
                evalexpr::Value::Boolean(b) => b.to_string(),
                _ => return None,
            };

            Some(SearchResultItem {
                id: format!("calc_{}", q),
                title: res_str,
                subtitle: Some(format!("= {}", q)),
                category: "Calculator".into(),
                item_type: "calc".into(),
                card_type: Some("math".into()),
                path: None,
                action_hint: Some("↵ Copy Result".into()),
                score: 1000,
                snippet: None,
                admin_path: None,
            })
        }
        Err(_) => None,
    }
}

fn sanitize_math(expr: &str) -> String {
    // In evalexpr, power is math::pow(a, b). For simple expressions with ^ we convert or leave
    expr.replace('^', " math::pow ")
}

fn try_unit_conversion(q: &str) -> Option<SearchResultItem> {
    let parts: Vec<&str> = q.split_whitespace().collect();
    if parts.len() != 4 || (parts[2].to_lowercase() != "to" && parts[2].to_lowercase() != "in") {
        return None;
    }

    let val: f64 = parts[0].parse().ok()?;
    let from = parts[1].to_lowercase();
    let to = parts[3].to_lowercase();

    let converted = convert_units(val, &from, &to)?;

    Some(SearchResultItem {
        id: format!("conv_{}_{}_{}", val, from, to),
        title: format!("{:.4} {}", converted, to)
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string(),
        subtitle: Some(format!("{} {} = ", val, from)),
        category: "Unit Converter".into(),
        item_type: "calc".into(),
        card_type: Some("math".into()),
        path: None,
        action_hint: Some("↵ Copy Result".into()),
        score: 1000,
        snippet: None,
        admin_path: None,
    })
}

fn convert_units(val: f64, from: &str, to: &str) -> Option<f64> {
    // Distance (base: meters)
    let dist_factor = |u: &str| match u {
        "m" | "meter" | "meters" => Some(1.0),
        "km" | "kilometer" | "kilometers" => Some(1000.0),
        "cm" | "centimeter" | "centimeters" => Some(0.01),
        "mm" | "millimeter" | "millimeters" => Some(0.001),
        "mi" | "mile" | "miles" => Some(1609.344),
        "yd" | "yard" | "yards" => Some(0.9144),
        "ft" | "foot" | "feet" => Some(0.3048),
        "in" | "inch" | "inches" => Some(0.0254),
        _ => None,
    };

    if let (Some(f1), Some(f2)) = (dist_factor(from), dist_factor(to)) {
        return Some(val * f1 / f2);
    }

    // Weight (base: grams)
    let weight_factor = |u: &str| match u {
        "g" | "gram" | "grams" => Some(1.0),
        "kg" | "kilogram" | "kilograms" => Some(1000.0),
        "mg" | "milligram" | "milligrams" => Some(0.001),
        "lb" | "lbs" | "pound" | "pounds" => Some(453.59237),
        "oz" | "ounce" | "ounces" => Some(28.3495),
        _ => None,
    };

    if let (Some(f1), Some(f2)) = (weight_factor(from), weight_factor(to)) {
        return Some(val * f1 / f2);
    }

    // Digital Storage (base: bytes)
    let storage_factor = |u: &str| match u {
        "b" | "byte" | "bytes" => Some(1.0),
        "kb" => Some(1024.0),
        "mb" => Some(1024.0 * 1024.0),
        "gb" => Some(1024.0 * 1024.0 * 1024.0),
        "tb" => Some(1024.0 * 1024.0 * 1024.0 * 1024.0),
        _ => None,
    };

    if let (Some(f1), Some(f2)) = (storage_factor(from), storage_factor(to)) {
        return Some(val * f1 / f2);
    }

    None
}
