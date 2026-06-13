//! Compile and runtime tests for docs/library/translation.md
//!
//! Requires the `translation` feature flag:
//!   cargo test --test doc_translation --features translation
//!   cargo test --test doc_translation --features translation -- --ignored   (network tests)

#![cfg(feature = "translation")]

use std::sync::Arc;

use finance_query::translation::{self, Lang, Translatable, TranslationBackend};

// ---------------------------------------------------------------------------
// Language Tags — from translation.md "Language Tags" section
// ---------------------------------------------------------------------------

#[test]
fn test_lang_parsing_and_normalization() {
    let lang = Lang::parse("zh-TW").unwrap();
    assert_eq!(lang.code(), "zh-Hant"); // region implies Traditional script
    assert!(!lang.is_english());

    assert!(Lang::parse("en-US").unwrap().is_english());
    assert_eq!(Lang::parse("pt_BR").unwrap().code(), "pt");
}

#[test]
fn test_invalid_lang_tags_error() {
    assert!(Lang::parse("").is_err());
    assert!(Lang::parse("1234").is_err());
}

#[tokio::test]
async fn test_providers_invalid_lang_fails_fast() {
    // Lang validation happens before provider initialisation — no network.
    let result = finance_query::Providers::builder()
        .lang("1234")
        .build()
        .await;
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// Translatable + dictionary tier — no backend, no network
// ---------------------------------------------------------------------------

struct Profile {
    sector: Option<String>,
}

impl Translatable for Profile {
    fn visit_translatable(&mut self, visit: &mut dyn FnMut(&mut String)) {
        if let Some(s) = &mut self.sector {
            visit(s);
        }
    }
}

#[tokio::test]
async fn test_dictionary_terms_translate_without_backend() {
    let mut p = Profile {
        sector: Some("Technology".into()),
    };
    translation::translate(&mut p, "ja").await.unwrap();
    assert_eq!(p.sector.as_deref(), Some("テクノロジー"));
}

#[tokio::test]
async fn test_english_target_is_a_no_op() {
    let mut p = Profile {
        sector: Some("Technology".into()),
    };
    translation::translate(&mut p, "en-US").await.unwrap();
    assert_eq!(p.sector.as_deref(), Some("Technology"));
}

// ---------------------------------------------------------------------------
// Custom Backend — from translation.md "Custom Backend" section
// ---------------------------------------------------------------------------

struct MyBackend;

#[async_trait::async_trait]
impl TranslationBackend for MyBackend {
    fn id(&self) -> &'static str {
        "my-backend"
    }

    async fn translate_batch(
        &self,
        texts: &[String],
        target: &Lang,
    ) -> finance_query::Result<Vec<String>> {
        // Call your translation service; return one string per input, in order.
        let _ = target;
        Ok(texts.to_vec())
    }
}

/// Compile-time verification of the custom-backend example. Never called:
/// `set_backend` is process-wide and would leak into the other tests in this
/// binary, which exercise the no-backend dictionary/passthrough behavior.
#[allow(dead_code)]
fn _register_custom_backend() {
    translation::set_backend(Arc::new(MyBackend));
}

// ---------------------------------------------------------------------------
// Builder examples — compile-time verification (network paths are ignored)
// ---------------------------------------------------------------------------

/// Verifies the Ticker / Tickers / Providers / standalone examples from
/// translation.md type-check. Never called; exists only for the compiler.
#[allow(dead_code)]
async fn _verify_builder_examples() -> Result<(), Box<dyn std::error::Error>> {
    use finance_query::{Capability, Provider, Providers, SearchOptions, Ticker, Tickers, finance};

    let ticker = Ticker::builder("7203.T").lang("ja").build().await?;
    let _quote = ticker.quote::<finance_query::format::Raw>().await?;

    let tickers = Tickers::builder(["AAPL", "NVDA"])
        .lang("de")
        .build()
        .await?;
    let _news = tickers.news().await?;

    let providers = Providers::builder()
        .route(Capability::QUOTE, &[Provider::Yahoo])
        .lang("ja")
        .build()
        .await?;
    let ticker = providers.ticker("7203.T").build().await?;
    let _quote = ticker.quote::<finance_query::format::Raw>().await?;
    let _en = providers.ticker("AAPL").lang("en-US").build().await?;

    let mut results = finance::search("toyota", &SearchOptions::default()).await?;
    translation::translate(&mut results, "ja").await?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Network tests
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_ticker_lang_translates_dictionary_fields() {
    use finance_query::Ticker;

    // Toyota: sector "Consumer Cyclical" has a dictionary entry in Japanese,
    // so this works with no ML backend configured.
    let ticker = Ticker::builder("7203.T").lang("ja").build().await.unwrap();
    let quote = ticker.quote::<finance_query::format::Raw>().await.unwrap();

    if let Some(sector) = &quote.sector_disp {
        println!("sector_disp (ja): {sector}");
        assert!(
            !sector.is_ascii(),
            "expected a translated (non-ASCII) sector, got '{sector}'"
        );
    }
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_standalone_search_translation() {
    use finance_query::{SearchOptions, finance};

    let mut results = finance::search("toyota", &SearchOptions::default())
        .await
        .unwrap();
    translation::translate(&mut results, "ja").await.unwrap();
    println!("translated search results: {} quotes", results.quotes.len());
}
