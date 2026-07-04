//! Per-request language handling for MCP tools.
//!
//! Tools accept an optional `lang` parameter (BCP 47). English, absent, or
//! unparseable tags resolve to no translation, matching the HTTP server.

use finance_query::translation::Lang;

/// Normalize a raw `lang` tool param: canonical code for a supported
/// non-English language, `None` otherwise.
pub fn normalize(lang: Option<&str>) -> Option<String> {
    let lang = Lang::parse(lang?.trim()).ok()?;
    if lang.is_english() {
        None
    } else {
        Some(lang.code())
    }
}

#[cfg(test)]
mod tests {
    use super::normalize;

    #[test]
    fn english_and_invalid_resolve_to_none() {
        assert_eq!(normalize(None), None);
        assert_eq!(normalize(Some("en")), None);
        assert_eq!(normalize(Some("en-US")), None);
        assert_eq!(normalize(Some("1234")), None);
        assert_eq!(normalize(Some("")), None);
    }

    #[test]
    fn supported_languages_canonicalize() {
        assert_eq!(normalize(Some("ja")).as_deref(), Some("ja"));
        assert_eq!(normalize(Some("zh-TW")).as_deref(), Some("zh-Hant"));
        assert_eq!(normalize(Some("zh")).as_deref(), Some("zh-Hans"));
    }
}
