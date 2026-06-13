//! Global target-language state for translated output (`--lang` / `FQ_LANG`).
//!
//! Commands construct tickers through [`ticker`]/[`tickers`] so the target
//! language flows into the library, which translates human-readable fields
//! (names, sectors, summaries, news titles) while leaving symbols, codes,
//! and numbers untouched.

use std::sync::OnceLock;

use finance_query::translation::{Lang, Translatable};
use finance_query::{Result, Ticker, Tickers};

static TARGET: OnceLock<Option<String>> = OnceLock::new();

/// Resolve and store the target language once at startup.
///
/// English, absent, and unparseable tags resolve to no translation;
/// invalid tags print a warning to stderr.
pub fn init(lang: Option<&str>) {
    let resolved = lang.and_then(|tag| match Lang::parse(tag) {
        Ok(parsed) if parsed.is_english() => None,
        Ok(parsed) => Some(parsed.code()),
        Err(_) => {
            eprintln!("warning: ignoring invalid language tag '{tag}'");
            None
        }
    });
    let _ = TARGET.set(resolved);
}

/// The resolved target language code, if translation is active.
pub fn target() -> Option<&'static str> {
    TARGET.get().and_then(|t| t.as_deref())
}

/// Build a [`Ticker`] honoring the global target language.
pub async fn ticker(symbol: impl Into<String>) -> Result<Ticker> {
    match target() {
        Some(lang) => Ticker::builder(symbol).lang(lang).build().await,
        None => Ticker::new(symbol).await,
    }
}

/// Build a [`Tickers`] batch honoring the global target language.
pub async fn tickers<S, I>(symbols: I) -> Result<Tickers>
where
    S: Into<String>,
    I: IntoIterator<Item = S>,
{
    match target() {
        Some(lang) => Tickers::builder(symbols).lang(lang).build().await,
        None => Tickers::new(symbols).await,
    }
}

/// Translate a typed library response in place when a target language is set.
pub async fn translate<T: Translatable>(value: &mut T) -> Result<()> {
    if let Some(lang) = target() {
        finance_query::translation::translate(value, lang).await?;
    }
    Ok(())
}
