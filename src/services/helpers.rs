// Small formatting/utility helpers shared across handlers and templates.
use chrono::{DateTime, Utc, NaiveDateTime};

/// Format an integer amount as Indonesian Rupiah, e.g. 1500000 -> "1.500.000".
pub fn format_rupiah(amount: i64) -> String {
    let neg = amount < 0;
    let digits = amount.abs().to_string();
    let mut out = String::new();
    let bytes = digits.as_bytes();
    let len = bytes.len();
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            out.push('.');
        }
        out.push(*b as char);
    }
    if neg {
        format!("-{out}")
    } else {
        out
    }
}

/// Full price with currency prefix, e.g. "Rp 1.500.000".
pub fn price(amount: i64, currency: &str) -> String {
    format!("{} {}", currency, format_rupiah(amount))
}

/// Generate a URL-safe slug from arbitrary text.
pub fn make_slug(text: &str) -> String {
    let s = slug::slugify(text);
    if s.is_empty() {
        format!("item-{}", &uuid::Uuid::new_v4().to_string()[..8])
    } else {
        s
    }
}

/// Generate a human-friendly order number like INV-20260729-A1B2.
pub fn generate_order_number() -> String {
    let now = Utc::now();
    let date = now.format("%Y%m%d");
    let suffix: String = uuid::Uuid::new_v4().to_string()[..4].to_uppercase();
    format!("INV-{date}-{suffix}")
}

/// Parse a stored SQLite datetime string (UTC) into a friendly local-ish label.
pub fn format_datetime(s: &str) -> String {
    // Stored as "YYYY-MM-DD HH:MM:SS"
    if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return dt.format("%d %b %Y, %H:%M").to_string();
    }
    s.to_string()
}

pub fn format_date(s: &str) -> String {
    if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return dt.format("%d %b %Y").to_string();
    }
    s.to_string()
}

/// Truncate text to n chars adding an ellipsis.
pub fn truncate(text: &str, n: usize) -> String {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= n {
        text.to_string()
    } else {
        let s: String = chars.into_iter().take(n).collect();
        format!("{s}...")
    }
}

/// Very small HTML sanitizer for user-provided rich text (whitelist tags).
/// Not a full sanitizer; escapes everything then re-allows a few safe tags.
pub fn safe_html(input: &str) -> String {
    let escaped = input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    // allow paragraph breaks from double newlines
    escaped
        .replace("\r\n", "\n")
        .split("\n\n")
        .map(|p| format!("<p>{}</p>", p.replace('\n', "<br>")))
        .collect::<Vec<_>>()
        .join("")
}

/// Current UTC timestamp string, used occasionally.
pub fn now_iso() -> String {
    let now: DateTime<Utc> = Utc::now();
    now.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Build a WhatsApp click-to-chat URL with a pre-filled message.
pub fn whatsapp_link(number: &str, message: &str) -> String {
    let digits: String = number.chars().filter(|c| c.is_ascii_digit()).collect();
    let normalized = if digits.starts_with('0') {
        format!("62{}", &digits[1..])
    } else {
        digits
    };
    format!("https://wa.me/{}?text={}", normalized, urlencoding::encode(message))
}
