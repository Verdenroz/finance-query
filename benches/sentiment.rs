//! Offline VADER sentiment-scoring latency (feature: `sentiment`).
//!
//! Scores real captured news + transcript payloads exactly as `Ticker::news()`
//! and `Transcript::overall_sentiment()` do
//!
//! ```text
//! cargo bench --bench sentiment --features finance-query/sentiment
//! ```

use criterion::{Criterion, Throughput, black_box, criterion_group, criterion_main};
use finance_query::{News, Transcript, analyze_sentiment};

static NEWS_GENERAL: &str = include_str!("fixtures/news.json");
static NEWS_SYMBOL: &str = include_str!("fixtures/news_symbol.json");
static TRANSCRIPT: &str = include_str!("fixtures/transcripts.json");

fn news_titles(json: &str) -> Vec<String> {
    let news: Vec<News> = serde_json::from_str(json).expect("valid news fixture");
    news.into_iter().map(|n| n.title).collect()
}

fn transcript_paragraphs(json: &str) -> (Transcript, usize) {
    let t: Transcript = serde_json::from_str(json).expect("valid transcript fixture");
    let chars = t
        .transcript_content
        .transcript
        .as_ref()
        .map(|d| d.text.len())
        .unwrap_or(0);
    (t, chars)
}

fn bench_news(c: &mut Criterion) {
    let general = news_titles(NEWS_GENERAL);
    let symbol = news_titles(NEWS_SYMBOL);

    let mut g = c.benchmark_group("endpoint_news");
    for (name, titles) in [("general", &general), ("symbol", &symbol)] {
        let total_bytes: usize = titles.iter().map(|t| t.len()).sum();
        g.throughput(Throughput::Bytes(total_bytes as u64));
        g.bench_function(format!("{name}_{}_titles", titles.len()), |b| {
            b.iter(|| {
                for title in titles {
                    black_box(analyze_sentiment(black_box(title)));
                }
            })
        });
    }
    g.finish();
}

/// Aggregate the whole earnings call — `Transcript::overall_sentiment()`, the
/// largest single payload sentiment ever runs over.
fn bench_transcript(c: &mut Criterion) {
    let (transcript, chars) = transcript_paragraphs(TRANSCRIPT);

    let mut g = c.benchmark_group("endpoint_transcript");
    g.throughput(Throughput::Bytes(chars as u64));
    g.bench_function(format!("overall_sentiment_{chars}_chars"), |b| {
        b.iter(|| {
            black_box(black_box(&transcript).overall_sentiment());
        })
    });
    g.finish();
}

/// Per-headline cost in isolation (the unit the feature claims is sub-millisecond).
fn bench_single(c: &mut Criterion) {
    let headline = "Apple stock surges to record high on blockbuster earnings beat";
    c.bench_function("single_headline", |b| {
        b.iter(|| black_box(analyze_sentiment(black_box(headline))))
    });
}

criterion_group!(
    sentiment_benches,
    bench_single,
    bench_news,
    bench_transcript
);
criterion_main!(sentiment_benches);
