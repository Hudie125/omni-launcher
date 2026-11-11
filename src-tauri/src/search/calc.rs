use super::SearchResultItem;

pub fn evaluate_calc_or_convert(query: &str) -> Option<SearchResultItem> {
    let q = query.trim();
    if q.is_empty() {
        return None;
    }

    // Try unit conversion first: "<num> <from> to <to>"
    if let Some(item) = try_unit_conversion(q) {
        return Some(item);
    }

    // Check if query looks like a math expression
    let has_math_tokens = q.chars().any(|c| "+-*/^%()".contains(c))
        || q.starts_with("sin(")
        || q.starts_with("cos(")
        || q.starts_with("sqrt(")
        || q.starts_with("tan(");

    if !has_math_tokens {
        return None;
    }

    // Evaluate arithmetic expression using evalexpr
    match evalexpr::eval(q) {
        Ok(value) => {
            let res_str = match value {
                evalexpr::Value::Float(f) => {
                    if f.fract() == 0.0 {
                        format!("{:.0}", f)
                    } else {
                        format!("{:.6}", f).trim_end_matches('0').trim_end_matches('.').to_string()
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
                action_hint: Some("↵ Copy".into()),
                score: 1000,
            })
        }
        Err(_) => None,
    }
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
        title: format!("{:.4} {}", converted, to).trim_end_matches('0').trim_end_matches('.').to_string(),
        subtitle: Some(format!("{} {} = ", val, from)),
        category: "Unit Converter".into(),
        item_type: "calc".into(),
        card_type: Some("math".into()),
        path: None,
        action_hint: Some("↵ Copy".into()),
        score: 1000,
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

    // Digital Storage (base: Megabytes)
    let data_factor = |u: &str| match u {
        "b" | "bytes" => Some(1.0 / (1024.0 * 1024.0)),
        "kb" => Some(1.0 / 1024.0),
        "mb" => Some(1.0),
        "gb" => Some(1024.0),
        "tb" => Some(1024.0 * 1024.0),
        _ => None,
    };

    if let (Some(f1), Some(f2)) = (data_factor(from), data_factor(to)) {
        return Some(val * f1 / f2);
    }

    // Temperature
    match (from, to) {
        ("c", "f") => Some(val * 9.0 / 5.0 + 32.0),
        ("f", "c") => Some((val - 32.0) * 5.0 / 9.0),
        ("c", "k") => Some(val + 273.15),
        ("k", "c") => Some(val - 273.15),
        _ => None,
    }
}
