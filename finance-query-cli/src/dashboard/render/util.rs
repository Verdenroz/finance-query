use ratatui::style::Color;

/// Get color based on percentage change (green gradient for positive, red for negative)
pub(super) fn get_change_color(change: f64) -> Color {
    if change >= 3.0 {
        Color::Rgb(0, 255, 0) // Bright green
    } else if change >= 2.0 {
        Color::Rgb(50, 205, 50) // Lime green
    } else if change >= 1.0 {
        Color::Rgb(144, 238, 144) // Light green
    } else if change >= 0.0 {
        Color::Rgb(152, 251, 152) // Pale green
    } else if change >= -1.0 {
        Color::Rgb(255, 182, 193) // Light pink
    } else if change >= -2.0 {
        Color::Rgb(255, 99, 71) // Tomato
    } else if change >= -3.0 {
        Color::Rgb(220, 20, 60) // Crimson
    } else {
        Color::Rgb(139, 0, 0) // Dark red
    }
}

pub(super) fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}…", &s[..max_len - 1])
    }
}

pub(super) fn format_market_cap(mcap: i64) -> String {
    match mcap {
        v if v >= 1_000_000_000_000 => format!("${:.2}T", v as f64 / 1_000_000_000_000.0),
        v if v >= 1_000_000_000 => format!("${:.2}B", v as f64 / 1_000_000_000.0),
        v if v >= 1_000_000 => format!("${:.2}M", v as f64 / 1_000_000.0),
        _ => format!("${}", mcap),
    }
}

pub(super) fn format_volume(volume: i64) -> String {
    match volume {
        v if v >= 1_000_000_000 => format!("{:.2}B", v as f64 / 1_000_000_000.0),
        v if v >= 1_000_000 => format!("{:.2}M", v as f64 / 1_000_000.0),
        v if v >= 1_000 => format!("{:.2}K", v as f64 / 1_000.0),
        _ => volume.to_string(),
    }
}

/// Check if current time is during overnight trading session (8 PM - 4 AM ET)
pub(super) fn is_overnight_session() -> bool {
    use chrono::{Timelike, Utc};
    use chrono_tz::America::New_York;

    let now_et = Utc::now().with_timezone(&New_York);
    let hour = now_et.hour();

    // Overnight: 8 PM (20:00) to 4 AM (04:00) ET
    !(4..20).contains(&hour)
}
