//! Offline-friendly translation of human-readable response fields.
//!
//! Yahoo Finance returns natural-language fields (company summaries, sector
//! names, news titles) in English regardless of the `lang`/`region` request
//! parameters. This module post-processes responses so the existing
//! `.lang()` / `.region()` builder surface actually localizes text.
//!
//! # Tiers
//!
//! 1. **Built-in dictionary** (always available with the `translation`
//!    feature): exact translations for the finite vocabulary of sector
//!    names, security types, and officer titles in 11 major languages.
//!    Zero latency, deterministic.
//! 2. **Machine translation backend** for free-form text (business
//!    summaries, news titles). The `translation-offline` feature provides a
//!    fully local CPU backend (opus-mt bilingual models, ~48 languages, no API
//!    key) that downloads a small per-language model on first use; a custom
//!    backend can be plugged via [`set_backend`]. Results are memoized
//!    process-wide.
//!
//! Without any backend, free-form fields are left in English while
//! dictionary terms are still translated — enabling `translation` alone
//! never breaks responses.
//!
//! # Example
//!
//! ```no_run
//! use finance_query::Ticker;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // With the `translation` feature, setting a language translates
//! // text fields automatically.
//! let ticker = Ticker::builder("7203.T").lang("ja").build().await?;
//! let quote = ticker.quote::<finance_query::format::Both>().await?;
//! // quote.sector_disp / long_business_summary are now Japanese.
//! # Ok(())
//! # }
//! ```
//!
//! Standalone values from `finance::*` functions can be translated
//! explicitly:
//!
//! ```no_run
//! use finance_query::{SearchOptions, finance, translation};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut results = finance::search("toyota", &SearchOptions::default()).await?;
//! translation::translate(&mut results, "ja").await?;
//! # Ok(())
//! # }
//! ```

mod backend;
mod dictionary;
mod lang;
mod memo;
#[cfg(feature = "translation-offline")]
mod opusmt;
#[cfg(any(feature = "translation-offline", test))]
mod split;
mod translatable;

pub use backend::{TranslationBackend, set_backend};
pub use lang::Lang;
#[cfg(feature = "translation-offline")]
pub use opusmt::preload;
pub use translatable::Translatable;

use crate::error::Result;
use dictionary::DictLang;

/// Translate the human-readable text fields of a value in place.
///
/// `lang` is a BCP 47 language tag (e.g. `"ja"`, `"de-DE"`, `"zh-Hant"`).
/// English targets are a no-op. Returns an error for structurally invalid
/// tags or when the machine-translation backend fails; fields not covered
/// by the dictionary are left in English when no backend is available.
pub async fn translate<T: Translatable + ?Sized>(value: &mut T, lang: &str) -> Result<()> {
    let lang = Lang::parse(lang)?;
    translate_with(value, &lang).await
}

/// Like [`translate`], with an already-parsed [`Lang`].
pub async fn translate_with<T: Translatable + ?Sized>(value: &mut T, lang: &Lang) -> Result<()> {
    if lang.is_english() {
        return Ok(());
    }
    let mut texts: Vec<String> = Vec::new();
    value.visit_translatable(&mut |s| texts.push(s.clone()));
    if texts.is_empty() {
        return Ok(());
    }

    let translated = translate_texts(&texts, lang).await?;

    let mut iter = translated.into_iter();
    value.visit_translatable(&mut |s| {
        if let Some(t) = iter.next() {
            *s = t;
        }
    });
    value.after_translate();
    Ok(())
}

/// Translate a batch of raw English texts, preserving order.
///
/// Applies the dictionary, the process-wide memo cache, and the active
/// machine-translation backend in that order. Texts without a dictionary
/// hit are returned unchanged when no backend is available.
pub async fn translate_texts(texts: &[String], lang: &Lang) -> Result<Vec<String>> {
    if lang.is_english() {
        return Ok(texts.to_vec());
    }
    let lang_code = lang.code();
    let dict_lang = DictLang::from_lang(lang);

    let mut out: Vec<Option<String>> = vec![None; texts.len()];
    // Unique texts that need the ML backend, with the slots they fill.
    let mut pending: Vec<String> = Vec::new();
    let mut pending_slots: std::collections::HashMap<String, Vec<usize>> =
        std::collections::HashMap::new();

    for (i, text) in texts.iter().enumerate() {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            out[i] = Some(text.clone());
            continue;
        }
        if let Some(dl) = dict_lang
            && let Some(hit) = dictionary::lookup(dl, trimmed)
        {
            out[i] = Some(hit.to_string());
            continue;
        }
        if let Some(hit) = memo::get(&lang_code, trimmed) {
            out[i] = Some(hit);
            continue;
        }
        let slots = pending_slots.entry(trimmed.to_string()).or_default();
        if slots.is_empty() {
            pending.push(trimmed.to_string());
        }
        slots.push(i);
    }

    if !pending.is_empty() {
        match backend::active_backend() {
            Some(backend) if backend.supports(lang) => {
                let translated = backend.translate_batch(&pending, lang).await?;
                for (source, translated) in pending.iter().zip(translated) {
                    memo::insert(&lang_code, source, &translated);
                    if let Some(slots) = pending_slots.get(source) {
                        for &i in slots {
                            out[i] = Some(translated.clone());
                        }
                    }
                }
            }
            Some(_) => {
                tracing::debug!(
                    lang = %lang_code,
                    count = pending.len(),
                    "translation backend does not support this language; leaving free-form text untranslated"
                );
            }
            None => {
                tracing::debug!(
                    lang = %lang_code,
                    count = pending.len(),
                    "no translation backend available; leaving free-form text untranslated"
                );
            }
        }
    }

    Ok(out
        .into_iter()
        .zip(texts)
        .map(|(translated, original)| translated.unwrap_or_else(|| original.clone()))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Profile {
        sector: Option<String>,
        summary: Option<String>,
    }

    impl Translatable for Profile {
        fn visit_translatable(&mut self, visit: &mut dyn FnMut(&mut String)) {
            if let Some(s) = &mut self.sector {
                visit(s);
            }
            if let Some(s) = &mut self.summary {
                visit(s);
            }
        }
    }

    #[tokio::test]
    async fn english_is_a_no_op() {
        let mut p = Profile {
            sector: Some("Technology".into()),
            summary: Some("A company.".into()),
        };
        translate(&mut p, "en-US").await.unwrap();
        assert_eq!(p.sector.as_deref(), Some("Technology"));
    }

    #[tokio::test]
    async fn dictionary_terms_translate_without_backend() {
        let mut p = Profile {
            sector: Some("Technology".into()),
            summary: None,
        };
        translate(&mut p, "ja").await.unwrap();
        assert_eq!(p.sector.as_deref(), Some("テクノロジー"));
    }

    #[cfg(not(feature = "translation-offline"))]
    #[tokio::test]
    async fn free_form_passes_through_without_backend() {
        let mut p = Profile {
            sector: Some("Healthcare".into()),
            summary: Some("Designs and sells devices.".into()),
        };
        translate(&mut p, "de").await.unwrap();
        assert_eq!(p.sector.as_deref(), Some("Gesundheitswesen"));
        assert_eq!(p.summary.as_deref(), Some("Designs and sells devices."));
    }

    #[tokio::test]
    async fn invalid_lang_errors() {
        let mut p = Profile {
            sector: None,
            summary: None,
        };
        assert!(translate(&mut p, "not a lang!").await.is_err());
    }

    // An uncovered language ("sw": no opus-mt package, not in the dictionary)
    // must degrade gracefully — text stays English, no error on the request.
    #[cfg(feature = "translation-offline")]
    #[tokio::test]
    async fn unsupported_offline_language_degrades_gracefully() {
        let mut p = Profile {
            sector: Some("Technology".into()),
            summary: Some("Designs and sells devices.".into()),
        };
        translate(&mut p, "sw").await.unwrap();
        assert_eq!(p.sector.as_deref(), Some("Technology"));
        assert_eq!(p.summary.as_deref(), Some("Designs and sells devices."));
    }

    #[tokio::test]
    async fn translate_texts_dedups_and_preserves_order() {
        let lang = Lang::parse("fr").unwrap();
        let texts = vec![
            "Technology".to_string(),
            "Energy".to_string(),
            "Technology".to_string(),
        ];
        let out = translate_texts(&texts, &lang).await.unwrap();
        assert_eq!(out, vec!["Technologie", "Énergie", "Technologie"]);
    }
}
