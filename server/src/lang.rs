//! Target-language resolution for translated responses.
//!
//! The explicit `lang` query parameter wins over the `Accept-Language`
//! header. English, absent, and unparseable tags all resolve to `None`
//! (no translation), so callers can use the resolved code directly in
//! cache keys.

use axum::http::HeaderMap;

/// Resolve the target translation language for a request.
///
/// Returns a canonical language code (e.g. `ja`, `zh-Hans`) or `None`
/// when no translation should be applied.
#[cfg(feature = "translation")]
pub fn resolve_lang(query_lang: Option<&str>, headers: &HeaderMap) -> Option<String> {
    let tag = query_lang
        .map(str::to_string)
        .or_else(|| accept_language(headers))?;
    let lang = finance_query::translation::Lang::parse(&tag).ok()?;
    if lang.is_english() {
        None
    } else {
        Some(lang.code())
    }
}

#[cfg(not(feature = "translation"))]
pub fn resolve_lang(_query_lang: Option<&str>, _headers: &HeaderMap) -> Option<String> {
    None
}

/// Pick the highest-quality language tag from an `Accept-Language` header.
#[cfg(feature = "translation")]
fn accept_language(headers: &HeaderMap) -> Option<String> {
    let raw = headers
        .get(axum::http::header::ACCEPT_LANGUAGE)?
        .to_str()
        .ok()?;
    raw.split(',')
        .filter_map(|part| {
            let mut pieces = part.trim().splitn(2, ';');
            let tag = pieces.next()?.trim();
            if tag.is_empty() || tag == "*" {
                return None;
            }
            let quality = pieces
                .next()
                .and_then(|p| p.trim().strip_prefix("q="))
                .and_then(|v| v.trim().parse::<f32>().ok())
                .unwrap_or(1.0);
            Some((tag.to_string(), quality))
        })
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(tag, _)| tag)
}

#[cfg(all(test, feature = "translation"))]
mod tests {
    use super::*;
    use axum::http::header::ACCEPT_LANGUAGE;

    fn headers_with(value: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT_LANGUAGE, value.parse().unwrap());
        headers
    }

    #[test]
    fn query_param_wins_over_header() {
        let headers = headers_with("de-DE");
        assert_eq!(resolve_lang(Some("ja"), &headers), Some("ja".to_string()));
    }

    #[test]
    fn header_used_when_no_query_param() {
        let headers = headers_with("fr-FR,fr;q=0.9,en;q=0.8");
        assert_eq!(resolve_lang(None, &headers), Some("fr".to_string()));
    }

    #[test]
    fn quality_ordering_is_respected() {
        let headers = headers_with("en;q=0.5,ja;q=0.9");
        assert_eq!(resolve_lang(None, &headers), Some("ja".to_string()));
    }

    #[test]
    fn english_resolves_to_none() {
        let headers = headers_with("en-US,en;q=0.9");
        assert_eq!(resolve_lang(None, &headers), None);
        assert_eq!(resolve_lang(Some("en-GB"), &headers), None);
    }

    #[test]
    fn wildcard_and_absent_resolve_to_none() {
        let headers = headers_with("*");
        assert_eq!(resolve_lang(None, &headers), None);
        assert_eq!(resolve_lang(None, &HeaderMap::new()), None);
    }

    #[test]
    fn chinese_script_is_canonicalized() {
        let headers = HeaderMap::new();
        assert_eq!(
            resolve_lang(Some("zh-TW"), &headers),
            Some("zh-Hant".to_string())
        );
        assert_eq!(
            resolve_lang(Some("zh"), &headers),
            Some("zh-Hans".to_string())
        );
    }
}
