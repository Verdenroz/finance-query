//! Latency benchmark for the offline translation backend.
//!
//! One-shot harness (not criterion): model load + cold batches dominate, so
//! statistical sampling would just re-measure the memo cache.
//!
//! Run: cargo bench --bench translation --features translation-offline
//!
//! Two workloads are measured per target language:
//!  * `quotes+news` — a batch of news headlines plus multi-sentence business
//!    summaries (a worst-case *quote* endpoint response).
//!  * `transcript`  — the paragraph bodies of a real earnings-call transcript
//!    fixture (the heaviest translatable endpoint by far).
//!
//! The goal is twofold: absolute latency, and **latency parity across
//! languages** — the spread between the fastest and slowest target language
//! for the same payload. With per-language opus-mt bilingual models the spread
//! is driven mostly by each model's size (decoder width) rather than by output
//! token count; this harness surfaces that spread directly.

use std::time::{Duration, Instant};

use finance_query::translation::{self, Lang};

const HEADLINES: [&str; 20] = [
    "Apple shares rise after record quarterly earnings beat analyst expectations.",
    "Federal Reserve signals two more rate cuts before the end of the year.",
    "Nvidia unveils next-generation AI accelerator with double the memory bandwidth.",
    "Oil prices slip as OPEC+ members debate extending production cuts into 2027.",
    "Tesla deliveries fall short of estimates amid intensifying competition in China.",
    "Microsoft expands data center footprint with $40 billion investment in Europe.",
    "Treasury yields climb after stronger-than-expected jobs report.",
    "Amazon announces stock split and accelerates share buyback program.",
    "Gold hits all-time high as investors seek safe havens amid trade tensions.",
    "Semiconductor stocks rally on easing export restrictions to key markets.",
    "JPMorgan raises S&P 500 year-end target citing resilient consumer spending.",
    "Bitcoin tops $150,000 as institutional inflows reach record levels.",
    "Boeing secures largest-ever widebody order from Gulf carriers.",
    "Eurozone inflation cools faster than forecast, pressuring the ECB to act.",
    "Berkshire Hathaway trims Apple stake while boosting energy holdings.",
    "Meta beats revenue estimates on stronger advertising demand.",
    "Japanese yen weakens past key level, prompting intervention warnings.",
    "Retail sales rebound in May as consumers shrug off tariff concerns.",
    "Alphabet announces dividend increase alongside cloud revenue acceleration.",
    "Small-cap stocks outperform as rate cut expectations broaden market rally.",
];

const SUMMARIES: [&str; 2] = [
    "Apple Inc. designs, manufactures, and markets smartphones, personal computers, \
     tablets, wearables, and accessories worldwide. The company offers iPhone, a line \
     of smartphones; Mac, a line of personal computers; iPad, a line of multi-purpose \
     tablets; and wearables, home, and accessories comprising AirPods, Apple TV, \
     Apple Watch, Beats products, and HomePod. It also provides AppleCare support and \
     cloud services; and operates various platforms, including the App Store. In \
     addition, the company offers various services, such as Apple Arcade, a game \
     subscription service; Apple Fitness+, a personalized fitness service; Apple \
     Music, which offers users a curated listening experience; Apple News+, a \
     subscription news and magazine service; Apple Pay, a cashless payment service; \
     Apple TV+, which offers exclusive original content; and Apple Card, a co-branded \
     credit card. The company was incorporated in 1977 and is headquartered in \
     Cupertino, California.",
    "NVIDIA Corporation provides graphics and compute and networking solutions in the \
     United States, Taiwan, China, Hong Kong, and internationally. The Compute & \
     Networking segment comprises Data Center computing platforms and end-to-end \
     networking platforms; NVIDIA DRIVE automated-driving platform and automotive \
     development agreements; Jetson robotics and other embedded platforms; NVIDIA AI \
     Enterprise and other software; and DGX Cloud software and services. The Graphics \
     segment offers GeForce GPUs for gaming and PCs, the GeForce NOW game streaming \
     service and related infrastructure, and solutions for gaming platforms. The \
     company's products are used in gaming, professional visualization, data center, \
     and automotive markets.",
];

/// Real earnings-call transcript fixture (paragraph bodies extracted at runtime).
const TRANSCRIPT_JSON: &str = include_str!("fixtures/transcripts.json");

/// Representative spread of target languages across scripts/families:
/// Latin (es/de/fr), Cyrillic (ru), CJK (ja/zh), Hangul (ko), RTL (ar).
const LANGS: [&str; 8] = ["es", "de", "fr", "ru", "ja", "zh", "ko", "ar"];

/// Target languages for this run. `FQ_BENCH_LANGS` (comma-separated) narrows
/// the set for fast config sweeps; unset uses the full [`LANGS`] spread.
fn langs() -> Vec<String> {
    match std::env::var("FQ_BENCH_LANGS") {
        Ok(v) if !v.trim().is_empty() => v.split(',').map(|s| s.trim().to_string()).collect(),
        _ => LANGS.iter().map(|s| s.to_string()).collect(),
    }
}

/// `FQ_BENCH_WORKLOAD=transcript|quotes` runs just one workload (sweeps only
/// care about the transcript); unset runs both.
fn workload_filter() -> Option<String> {
    std::env::var("FQ_BENCH_WORKLOAD")
        .ok()
        .filter(|s| !s.trim().is_empty())
}

fn transcript_paragraphs() -> Vec<String> {
    let value: serde_json::Value =
        serde_json::from_str(TRANSCRIPT_JSON).expect("transcript fixture parses");
    value["transcriptContent"]["transcript"]["paragraphs"]
        .as_array()
        .expect("paragraphs array")
        .iter()
        .filter_map(|p| p["text"].as_str())
        .map(str::to_string)
        .collect()
}

struct LangResult {
    code: String,
    elapsed: Duration,
    out_chars: usize,
}

async fn run_workload(name: &str, texts: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let in_chars: usize = texts.iter().map(String::len).sum();
    println!(
        "\n=== workload: {name} ({} texts, {} input chars) ===",
        texts.len(),
        in_chars
    );

    let mut results: Vec<LangResult> = Vec::new();
    for code in langs() {
        let lang = Lang::parse(&code)?;
        let t = Instant::now();
        let out = translation::translate_texts(texts, &lang).await?;
        let elapsed = t.elapsed();
        let out_chars: usize = out.iter().map(String::len).sum();
        results.push(LangResult {
            code,
            elapsed,
            out_chars,
        });
    }

    let fastest = results
        .iter()
        .map(|r| r.elapsed)
        .min()
        .unwrap_or_default()
        .as_secs_f64();
    let slowest = results
        .iter()
        .map(|r| r.elapsed)
        .max()
        .unwrap_or_default()
        .as_secs_f64();

    for r in &results {
        let secs = r.elapsed.as_secs_f64();
        println!(
            "  {:<4} {:>8.2?}  ({:>7.0} in-char/s, {:>6} out-chars, {:.2}x fastest)",
            r.code,
            r.elapsed,
            in_chars as f64 / secs.max(1e-9),
            r.out_chars,
            secs / fastest.max(1e-9),
        );
    }
    println!(
        "  spread: fastest {:.2}s … slowest {:.2}s  (Δ {:.2}s, {:.2}x)",
        fastest,
        slowest,
        slowest - fastest,
        slowest / fastest.max(1e-9),
    );
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let t = Instant::now();
    translation::preload().await?;
    println!("model load: {:?}", t.elapsed());

    let only = workload_filter();
    let run = |name: &str| only.as_deref().is_none_or(|w| name.starts_with(w));

    if run("quotes") {
        let quotes_news: Vec<String> = HEADLINES
            .iter()
            .chain(SUMMARIES.iter())
            .map(|s| s.to_string())
            .collect();
        run_workload("quotes+news", &quotes_news).await?;
    }
    if run("transcript") {
        let transcript = transcript_paragraphs();
        run_workload("transcript", &transcript).await?;
    }

    Ok(())
}
