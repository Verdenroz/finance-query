//! Process-wide memoization of completed translations.
//!
//! Finite terms (exchange names, repeated headlines, sector strings missed by
//! the dictionary) recur constantly across requests; caching them gives
//! dictionary-level latency for repeat translations without re-running the
//! ML backend.

use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

/// Texts longer than this are not memoized. Sized to cover business
/// summaries (typically 1–3 KB) so repeated quote reads don't re-run the ML
/// backend; full transcripts are excluded.
const MAX_MEMO_TEXT_LEN: usize = 8192;

/// Maximum number of cached entries before the cache is reset.
const MAX_MEMO_ENTRIES: usize = 4096;

type MemoMap = HashMap<(String, String), String>;

fn memo() -> &'static RwLock<MemoMap> {
    static MEMO: OnceLock<RwLock<MemoMap>> = OnceLock::new();
    MEMO.get_or_init(|| RwLock::new(HashMap::new()))
}

/// Look up a previously translated text for a language code.
pub(crate) fn get(lang_code: &str, text: &str) -> Option<String> {
    if text.len() > MAX_MEMO_TEXT_LEN {
        return None;
    }
    memo()
        .read()
        .ok()?
        .get(&(lang_code.to_string(), text.to_string()))
        .cloned()
}

/// Store a completed translation.
pub(crate) fn insert(lang_code: &str, text: &str, translated: &str) {
    if text.len() > MAX_MEMO_TEXT_LEN {
        return;
    }
    if let Ok(mut map) = memo().write() {
        if map.len() >= MAX_MEMO_ENTRIES {
            map.clear();
        }
        map.insert(
            (lang_code.to_string(), text.to_string()),
            translated.to_string(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        insert("xx-memo-test", "Hello", "Bonjour");
        assert_eq!(get("xx-memo-test", "Hello"), Some("Bonjour".to_string()));
        assert_eq!(get("xx-memo-test", "Goodbye"), None);
        assert_eq!(get("yy-memo-test", "Hello"), None);
    }

    #[test]
    fn skips_oversized_texts() {
        let big = "a".repeat(MAX_MEMO_TEXT_LEN + 1);
        insert("xx-memo-test", &big, "translated");
        assert_eq!(get("xx-memo-test", &big), None);
    }
}
