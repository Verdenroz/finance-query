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
    - `translation-offline` — fully local opus-mt bilingual models (~48 languages, no API key); a small per-language model (~80–210 MB) is downloaded on first use and cached
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

The `translation-offline` feature bundles a fully local CPU backend built on
**opus-mt bilingual** models (one English→target model per language) run through
CTranslate2 with int8 weights. Models are distributed as Argos Translate
packages — the same models LibreTranslate uses:

- A small per-language package (~80–210 MB: a CTranslate2 model directory plus a
  SentencePiece tokenizer) is downloaded from the Argos package server on first
  use of that language and cached; every subsequent run is fully offline. No API
  key is required.
- The cache root is `$FINANCE_QUERY_TRANSLATION_CACHE`, falling back to
  `$HF_HOME/argos`, then `~/.cache/huggingface/argos` — reusing the same
  persistent volume as other cached model data.
- Intra-op threads per model default to `min(cores, 8)`; override with
  `FINANCE_QUERY_TRANSLATION_THREADS`.
- Models load lazily on the first request for each language. Servers and CLIs can
  instead warm a set up front (avoiding the one-time load on the first translated
  request) by listing primary subtags in `FINANCE_QUERY_TRANSLATION_PRELOAD`
  (e.g. `es,ja,de`) and calling:

```rust
finance_query::translation::preload().await?;
```

### Language coverage

The offline backend covers **~48 languages** — the major world languages with a
published opus-mt package (Simplified and Traditional Chinese are distinct
packages). This is narrower than a single massively-multilingual model, and is a
deliberate trade: per-language bilingual models are an order of magnitude smaller
and give far tighter cross-language latency parity than one shared decoder.

A target language with no package degrades gracefully rather than erroring the
response: free-form text stays English while the **built-in dictionary tier still
applies** (sector names, security types, officer titles in its 11 languages).
Only an explicit standalone `translate()` call to an unsupported language returns
the "not supported by the offline translation model" error.

### Performance

Inference runs on the CPU (int8, oneDNN). Each model call sorts its sentences by
length and batches them by token budget, so latency tracks real output tokens
rather than padding — keeping the spread between languages tight. On a modern
8-core machine a typical response payload — ~20 news titles plus a couple of
multi-sentence business summaries — translates in roughly 2–3 seconds cold;
memoization makes repeated fields free within a process.

Two characteristics are inherent to the model set, not tuning gaps:

- **The spread is bounded by model size.** German and Russian sit at the slow end
  purely because their opus-mt decoders are the largest in the index (~160–210 MB
  vs ~80 MB for the smallest); there is no smaller published conversion to swap
  in. All languages otherwise share the same int8 dnnl GEMM path.
- **CTranslate2 (`ct2rs`) runs somewhat slower than the reference Python
  CTranslate2** for the same model, a property of the vendored native build's GEMM
  backend rather than this crate.

Earnings **transcripts** are the deliberate exception: a full call transcript is
hundreds of kilobytes of prose and takes tens of seconds to translate, and the
`/transcripts/{symbol}/all` server endpoint translates every transcript it
returns. The server caches translated transcripts for 7 days (transcripts are
immutable), so the cost is paid once per symbol/quarter/language. If you need
guaranteed-fast transcript responses, request them in English and translate
client-side, or warm the cache ahead of time.

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
