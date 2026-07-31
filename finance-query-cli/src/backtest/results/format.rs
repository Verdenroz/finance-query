use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

/// Linear interpolation percentile on a pre-sorted slice.
/// `p` is in [0.0, 1.0]. Uses the "inclusive" (C=1) method.
pub(super) fn percentile(sorted: &[f64], p: f64) -> f64 {
    debug_assert!(
        (0.0..=1.0).contains(&p),
        "percentile p must be in [0, 1], got {p}"
    );
    let n = sorted.len();
    if n == 0 {
        return 0.0;
    }
    if n == 1 {
        return sorted[0];
    }
    let rank = p * (n - 1) as f64;
    let lo = rank.floor() as usize;
    let hi = lo + 1;
    let frac = rank - lo as f64;
    if hi >= n {
        sorted[n - 1]
    } else {
        sorted[lo] + frac * (sorted[hi] - sorted[lo])
    }
}

pub(super) fn metric_line(label: &str, value: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("  {:<18}", label),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(value.to_string(), Style::default().fg(Color::White)),
    ])
}

/// Returns `true` when consecutive equity curve bars are less than one trading day apart,
/// which indicates an intraday backtest (1m, 5m, 15m, 30m, 1h intervals).
pub(super) fn is_intraday(equity_curve: &[finance_query::backtesting::EquityPoint]) -> bool {
    const ONE_DAY_SECS: i64 = 86_400;
    equity_curve
        .windows(2)
        .any(|w| (w[1].timestamp - w[0].timestamp).abs() < ONE_DAY_SECS)
}

pub(super) fn format_timestamp(ts: i64) -> String {
    use chrono::DateTime;
    if let Some(dt) = DateTime::from_timestamp(ts, 0) {
        dt.format("%Y-%m-%d").to_string()
    } else {
        ts.to_string()
    }
}

pub(super) fn format_timestamp_with_precision(ts: i64, intraday: bool) -> String {
    use chrono::DateTime;
    if let Some(dt) = DateTime::from_timestamp(ts, 0) {
        if intraday {
            dt.format("%Y-%m-%d %H:%M").to_string()
        } else {
            dt.format("%Y-%m-%d").to_string()
        }
    } else {
        ts.to_string()
    }
}

pub(super) fn format_duration_secs(secs: f64) -> String {
    if secs <= 0.0 {
        return "0s".to_string();
    }
    let days = secs / 86400.0;
    if days >= 1.0 {
        format!("{:.1}d", days)
    } else {
        let hours = secs / 3600.0;
        if hours >= 1.0 {
            format!("{:.1}h", hours)
        } else {
            format!("{:.0}m", secs / 60.0)
        }
    }
}

pub(super) fn format_ratio(ratio: f64) -> String {
    if ratio.is_nan() {
        "-".to_string()
    } else if ratio == f64::MAX {
        "∞".to_string()
    } else if ratio.is_infinite() {
        if ratio.is_sign_negative() {
            "-∞".to_string()
        } else {
            "∞".to_string()
        }
    } else {
        format!("{:.2}", ratio)
    }
}

pub(super) fn return_color(value: f64) -> Color {
    if value > 0.0 {
        Color::Green
    } else if value < 0.0 {
        Color::Red
    } else {
        Color::DarkGray
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_ratio_handles_max_sentinel() {
        assert_eq!(format_ratio(f64::MAX), "∞");
    }
}
