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

/// Nested so `get` can borrow both keys instead of allocating an owned tuple to
/// probe the map — the whole point of the cache is that a hit costs nothing.
/// `entries` tracks the total across languages so the cap check stays O(1).
#[derive(Default)]
struct Memo {
    by_lang: HashMap<String, HashMap<String, String>>,
    entries: usize,
}

fn memo() -> &'static RwLock<Memo> {
    static MEMO: OnceLock<RwLock<Memo>> = OnceLock::new();
    MEMO.get_or_init(|| RwLock::new(Memo::default()))
}

/// Look up a previously translated text for a language code.
pub(crate) fn get(lang_code: &str, text: &str) -> Option<String> {
    if text.len() > MAX_MEMO_TEXT_LEN {
        return None;
    }
    memo()
        .read()
        .ok()?
        .by_lang
        .get(lang_code)?
        .get(text)
        .cloned()
}

/// Store a completed translation.
pub(crate) fn insert(lang_code: &str, text: &str, translated: &str) {
    if text.len() > MAX_MEMO_TEXT_LEN {
        return;
    }
    if let Ok(mut memo) = memo().write() {
        if memo.entries >= MAX_MEMO_ENTRIES {
            memo.by_lang.clear();
            memo.entries = 0;
        }
        if memo
            .by_lang
            .entry(lang_code.to_string())
            .or_default()
            .insert(text.to_string(), translated.to_string())
            .is_none()
        {
            memo.entries += 1;
        }
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

    #[test]
    fn insert_then_get_round_trips_per_language() {
        insert("zz-memo-ja", "MemoCacheRoundTrip", "テクノロジー");
        insert("zz-memo-de", "MemoCacheRoundTrip", "Technologie");
        assert_eq!(
            get("zz-memo-ja", "MemoCacheRoundTrip").as_deref(),
            Some("テクノロジー")
        );
        assert_eq!(
            get("zz-memo-de", "MemoCacheRoundTrip").as_deref(),
            Some("Technologie")
        );
        assert_eq!(get("zz-memo-fr", "MemoCacheRoundTrip"), None);
        assert_eq!(get("zz-memo-ja", "MemoCacheUnseen"), None);
    }

    #[test]
    fn oversized_text_is_not_cached() {
        let big = "x".repeat(MAX_MEMO_TEXT_LEN + 1);
        insert("zz-memo-oversized", &big, "ignored");
        assert_eq!(get("zz-memo-oversized", &big), None);
    }

    /// The memo is a process-wide static shared with every other test in the
    /// suite, but nothing else touches the `zz-cap-*` language codes used
    /// here, and this test's own MAX_MEMO_ENTRIES+ insertions are enough to
    /// force at least one clear regardless of unrelated concurrent activity.
    #[test]
    fn entry_cap_is_enforced_across_languages() {
        let first_text = "zz-cap-first-only-once";
        insert("zz-cap-a", first_text, "seed");

        let extra = 8;
        for i in 0..(MAX_MEMO_ENTRIES + extra) {
            let lang = if i % 2 == 0 { "zz-cap-a" } else { "zz-cap-b" };
            let text = format!("zz-cap-text-{i}");
            insert(lang, &text, "v");
        }

        // Crossing the cap clears the whole map, so an entry written well
        // before the crossing point cannot have survived.
        assert_eq!(get("zz-cap-a", first_text), None);

        // Entries written after the clear (the loop's tail) do survive.
        let last_i = MAX_MEMO_ENTRIES + extra - 1;
        let last_lang = if last_i.is_multiple_of(2) {
            "zz-cap-a"
        } else {
            "zz-cap-b"
        };
        let last_text = format!("zz-cap-text-{last_i}");
        assert_eq!(get(last_lang, &last_text).as_deref(), Some("v"));
    }
}
