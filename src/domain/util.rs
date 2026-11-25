//! Small helpers that reproduce JavaScript semantics the original code relied on
//! (rounding, number formatting, string slicing), so output is identical.

/// `Math.round`: rounds half toward +∞ (Rust's `round` rounds half away from zero).
pub fn js_round(x: f64) -> f64 {
    (x + 0.5).floor()
}

/// `Math.round(x * 10) / 10`
pub fn round1(x: f64) -> f64 {
    js_round(x * 10.0) / 10.0
}

/// `Math.round(x * 100) / 100`
pub fn round2(x: f64) -> f64 {
    js_round(x * 100.0) / 100.0
}

/// `x.toFixed(digits)` (ties round up, like JS for exactly representable halves).
pub fn to_fixed(x: f64, digits: usize) -> String {
    let f = 10f64.powi(digits as i32);
    format!("{:.*}", digits, js_round(x * f) / f)
}

/// Formats a number the way JS template literals do for the values used here
/// (shortest round-trip representation, no trailing `.0`).
pub fn num(x: f64) -> String {
    if x.is_finite() && x == x.trunc() && x.abs() < 1e15 {
        format!("{}", x as i64)
    } else {
        format!("{x}")
    }
}

/// Inserts thousands separators: 1234567 -> "1,234,567".
pub fn group_thousands(int_part: u64) -> String {
    let s = int_part.to_string();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

/// `n.toLocaleString("en-US", { maximumFractionDigits: d, minimumFractionDigits: d })`
pub fn fixed_grouped(n: f64, digits: usize) -> String {
    let neg = n < 0.0;
    let factor = 10f64.powi(digits as i32);
    let scaled = js_round(n.abs() * factor);
    let int_part = (scaled / factor).trunc() as u64;
    let frac = (scaled - (int_part as f64) * factor).round() as u64;
    let mut s = group_thousands(int_part);
    if digits > 0 {
        s.push('.');
        s.push_str(&format!("{:0width$}", frac, width = digits));
    }
    if neg && scaled != 0.0 {
        format!("-{s}")
    } else {
        s
    }
}

/// `n.toLocaleString("en-US")` for integers / short decimals (max 3 fraction digits).
pub fn locale(n: f64) -> String {
    let r = js_round(n * 1000.0) / 1000.0;
    if r == r.trunc() {
        fixed_grouped(r, 0)
    } else {
        let s = format!("{:.3}", r.abs());
        let frac = s.split('.').nth(1).unwrap_or("").trim_end_matches('0').to_string();
        let int_part = r.abs().trunc() as u64;
        let sign = if r < 0.0 { "-" } else { "" };
        format!("{sign}{}.{frac}", group_thousands(int_part))
    }
}

/// `n.toLocaleString("en-US", { style: "currency", currency: "USD", ... })`
pub fn money(n: f64, digits: usize) -> String {
    let s = fixed_grouped(n.abs(), digits);
    if n < 0.0 && js_round(n.abs() * 10f64.powi(digits as i32)) != 0.0 {
        format!("-${s}")
    } else {
        format!("${s}")
    }
}

/// Largest byte index `<= i` that is a char boundary.
pub fn floor_boundary(s: &str, mut i: usize) -> usize {
    if i >= s.len() {
        return s.len();
    }
    while !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

/// Char-based `String.prototype.slice(a, b)` with clamping (no panics on short strings).
pub fn slice(s: &str, start: usize, end: usize) -> String {
    s.chars().skip(start).take(end.saturating_sub(start)).collect()
}

/// Char-based `slice(start)`.
pub fn slice_from(s: &str, start: usize) -> String {
    s.chars().skip(start).collect()
}

/// Truncates to at most `n` chars (`s.slice(0, n)`).
pub fn take_chars(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

/// Splits text into sentences on whitespace that follows `.`, `!` or `?`
/// (the original used the look-behind regex `/(?<=[.!?])\s+/`).
pub fn split_sentences(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let bytes = text.as_bytes();
    let mut start = 0;
    let mut i = 0;
    while i < bytes.len() {
        if matches!(bytes[i], b'.' | b'!' | b'?') {
            let mut j = i + 1;
            while j < bytes.len() && (bytes[j] as char).is_ascii_whitespace() {
                j += 1;
            }
            if j > i + 1 {
                out.push(&text[start..i + 1]);
                start = j;
                i = j;
                continue;
            }
        }
        i += 1;
    }
    out.push(&text[start..]);
    out
}

/// `Number(s)`: whitespace-trimmed decimal parse; "" -> 0, invalid -> None (NaN).
pub fn js_number(s: &str) -> Option<f64> {
    let t = s.trim();
    if t.is_empty() {
        return Some(0.0);
    }
    t.parse::<f64>().ok()
}

/// JSON number that serializes like JavaScript (`7`, not `7.0`).
pub fn json_num(x: f64) -> serde_json::Value {
    if x.is_finite() && x == x.trunc() && x.abs() < 9e15 {
        serde_json::Value::from(x as i64)
    } else {
        serde_json::Number::from_f64(x).map(serde_json::Value::Number).unwrap_or(serde_json::Value::Null)
    }
}

/// Current time as an ISO-8601 string with milliseconds (`new Date().toISOString()`).
pub fn now_iso() -> String {
    #[cfg(feature = "ssr")]
    {
        chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
    }
    #[cfg(not(feature = "ssr"))]
    {
        String::from(js_sys::Date::new_0().to_iso_string())
    }
}

/// Local time as `YYYYMMDDHHMMSS` (HL7 v2 timestamp).
pub fn now_hl7_ts() -> String {
    #[cfg(feature = "ssr")]
    {
        chrono::Local::now().format("%Y%m%d%H%M%S").to_string()
    }
    #[cfg(not(feature = "ssr"))]
    {
        let d = js_sys::Date::new_0();
        format!(
            "{:04}{:02}{:02}{:02}{:02}{:02}",
            d.get_full_year(),
            d.get_month() + 1,
            d.get_date(),
            d.get_hours(),
            d.get_minutes(),
            d.get_seconds()
        )
    }
}

/// High-resolution milliseconds (`performance.now()`), for timing measurements.
pub fn now_ms() -> f64 {
    #[cfg(feature = "ssr")]
    {
        use std::sync::OnceLock;
        use std::time::Instant;
        static START: OnceLock<Instant> = OnceLock::new();
        START.get_or_init(Instant::now).elapsed().as_secs_f64() * 1000.0
    }
    #[cfg(not(feature = "ssr"))]
    {
        web_sys::window().and_then(|w| w.performance()).map(|p| p.now()).unwrap_or(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn formats_like_js() {
        assert_eq!(money(1234.5, 0), "$1,235");
        assert_eq!(money(2480.0, 0), "$2,480");
        assert_eq!(money(-12.5, 2), "-$12.50");
        assert_eq!(num(2.6), "2.6");
        assert_eq!(num(10.0), "10");
        assert_eq!(locale(1234567.0), "1,234,567");
        assert_eq!(js_round(-2.5), -2.0);
        assert_eq!(split_sentences("A b. C d! E"), vec!["A b.", "C d!", "E"]);
    }
}
