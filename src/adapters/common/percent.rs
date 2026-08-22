//! Percent-string parsing shared by adapters that report values as `"N%"`.

/// Parse a percent-formatted string (e.g. `"29.50%"` or `"29.50"`) into its
/// raw numeric value (`29.5`). Callers convert to a fraction themselves when
/// their canonical field expects one.
pub(crate) fn strip_percent(s: &str) -> Option<f64> {
    s.trim().trim_end_matches('%').trim().parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_with_and_without_percent_sign() {
        assert_eq!(strip_percent("29.50%"), Some(29.5));
        assert_eq!(strip_percent(" -9.91% "), Some(-9.91));
        assert_eq!(strip_percent("0"), Some(0.0));
        assert_eq!(strip_percent("not-a-number"), None);
    }
}
