//! Numeric parsing shared by the macro-data adapters.

/// Parse a string-encoded observation value.
///
/// These APIs encode "no figure published" in-band as a sentinel string rather
/// than as JSON `null`; `missing` lists this API's sentinels, matched
/// case-insensitively. Anything else unparseable is also `None`.
pub(crate) fn parse_number(raw: &str, missing: &[&str]) -> Option<f64> {
    let raw = raw.trim();
    if raw.is_empty() || missing.iter().any(|s| raw.eq_ignore_ascii_case(s)) {
        return None;
    }
    raw.parse::<f64>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sentinels_and_junk_parse_to_none() {
        assert_eq!(parse_number(" 3.5 ", &["-"]), Some(3.5));
        assert_eq!(parse_number("-", &["-"]), None);
        assert_eq!(parse_number("NULL", &["null"]), None);
        assert_eq!(parse_number("", &["null"]), None);
        assert_eq!(parse_number("n/a", &["-"]), None);
    }
}
