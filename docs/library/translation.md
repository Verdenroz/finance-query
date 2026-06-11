# Translations

!!! abstract "Cargo Docs"
    [docs.rs/finance-query — translation](https://docs.rs/finance-query/latest/finance_query/translation/index.html)

!!! info "Feature flag required"
    ```toml
    finance-query = { version = "...", features = ["translation"] }
    ```
    Add `translation-offline` for the fully local machine-translation backend
    (compiles CTranslate2 from source — requires `cmake` and a C++ toolchain).

Yahoo Finance returns natural-language fields (company summaries, sector names, news titles) in English regardless of the `lang`/`region` request parameters. The `translation` module post-processes responses so the existing `.lang()` builder surface actually localizes text.

Only human-readable fields are translated — names, sector/industry display strings, business summaries, news titles, officer titles, transcripts. Symbols, codes, URLs, and numbers are never touched.

## Translation Tiers

1. **Built-in dictionary** (always available with `translation`): exact translations for the finite vocabulary of sector names, security types, and officer titles in 11 languages — German, Spanish, French, Italian, Portuguese, Dutch, Japanese, Korean, Simplified Chinese, Traditional Chinese, Russian. Zero latency, deterministic.
2. **Machine translation backend** for free-form text (business summaries, news titles), with process-wide memoization:
    - `translation-offline` — fully local NLLB-200 model (40+ languages, no API key)
    - or a custom backend registered via `set_backend`

Without any backend, free-form fields are left in English while dictionary terms are still translated — enabling `translation` alone never breaks responses.

## Via Ticker

Setting a non-English language on the builder translates text fields automatically:

```rust
use finance_query::Ticker;

let ticker = Ticker::builder("7203.T").lang("ja").build().await?;
let quote = ticker.quote::<finance_query::format::Raw>().await?;
// quote.sector_disp / long_business_summary are now Japanese.
```

Batch tickers work the same way:

```rust
use finance_query::Tickers;

let tickers = Tickers::builder(["AAPL", "NVDA"]).lang("de").build().await?;
let news = tickers.news().await?;  // titles translated to German
```

## Via Providers

Multi-provider setups configure the language once on `ProvidersBuilder` — every
`Ticker`/`Tickers` handle created from it inherits the language (and translates
automatically when it is non-English):

```rust
use finance_query::{Capability, Provider, Providers};

let providers = Providers::builder()
    .route(Capability::QUOTE, &[Provider::Yahoo])
    .lang("ja")
    .build().await?;

let ticker = providers.ticker("7203.T").build().await?;
let quote = ticker.quote::<finance_query::format::Raw>().await?;  // Japanese

// Override per handle if needed:
let en = providers.ticker("AAPL").lang("en-US").build().await?;
```

`.region()` on `ProvidersBuilder` also sets the language (e.g. `Region::Japan`
→ `ja-JP`).

## Standalone Values

Results from `finance::*` functions can be translated explicitly:

```rust
use finance_query::{SearchOptions, finance, translation};

let mut results = finance::search("toyota", &SearchOptions::default()).await?;
translation::translate(&mut results, "ja").await?;
```

`translate` accepts any type implementing the `Translatable` trait — all text-bearing response models implement it, and `Vec<T>` / `Option<T>` compose.

## Language Tags

Targets are BCP 47 language tags, parsed and normalized by `Lang`:

```rust
use finance_query::translation::Lang;

let lang = Lang::parse("zh-TW")?;
assert_eq!(lang.code(), "zh-Hant");   // region implies Traditional script
assert!(!lang.is_english());

assert!(Lang::parse("en-US")?.is_english());  // English → translation is a no-op
assert_eq!(Lang::parse("pt_BR")?.code(), "pt");  // underscores accepted
```

English targets are always a no-op; structurally invalid tags return an error.

## Offline Backend

The `translation-offline` feature bundles a fully local CPU backend (NLLB-200 distilled 600M, int8, via CTranslate2):

- Model files (~600 MB) are downloaded from the Hugging Face Hub on first use and cached (respects `HF_HOME`); every subsequent run is fully offline.
- Override the model repository with the `FINANCE_QUERY_TRANSLATION_MODEL` env var (any CTranslate2 NLLB conversion with a `tokenizer.json`).
- Servers and CLIs can warm the model up front instead of paying the one-time load on the first translated request:

```rust
finance_query::translation::preload().await?;
```

## Custom Backend

Plug any translation engine (e.g. a hosted API) by implementing `TranslationBackend` (requires the [`async-trait`](https://crates.io/crates/async-trait) crate) and registering it process-wide. A custom backend takes precedence over the built-in offline backend:

```rust
use std::sync::Arc;
use finance_query::translation::{self, Lang, TranslationBackend};

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

translation::set_backend(Arc::new(MyBackend));
```

Inputs are English and may contain multiple sentences; implementations must return one translated string per input, preserving order. Results are memoized process-wide, so repeated fields (e.g. the same sector name across symbols) hit the backend only once.

## Server, CLI & MCP

The same pipeline powers the other finance-query frontends:

- **HTTP server** — `lang` query parameter (e.g. `/v2/quote/AAPL?lang=ja`), falling back to the `Accept-Language` header; cache keys are language-qualified.
- **CLI** — global `--lang` flag or `FQ_LANG` env var (e.g. `fq quote 7203.T --lang ja`).
- **MCP server** — optional `lang` parameter on text-bearing tools (`get_quote`, `get_news`, `search`, ...).

## Next Steps

- [Ticker API](ticker.md) - Full Ticker method reference
- [Batch Tickers](tickers.md) - Multi-symbol operations
- [Configuration](configuration.md) - Builder knobs including `.lang()` and `.region()`
