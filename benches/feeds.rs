//! RSS/Atom feed-parsing benchmark (feature: `rss`).
//!
//! Mirrors the parse path in `src/feeds/mod.rs` (`feed_rs::parser::parse` over
//! the fetched bytes) without any network: the fixtures are offline XML in
//! `benches/fixtures/`. Measures the cost that dominates `feeds::fetch` once the
//! bytes are in hand.
//!
//! ```text
//! cargo bench --bench feeds --features finance-query/rss
//! ```

use criterion::{Criterion, Throughput, black_box, criterion_group, criterion_main};
use feed_rs::parser;

static RSS: &[u8] = include_bytes!("fixtures/feed_rss.xml");

fn bench_parse(c: &mut Criterion) {
    let mut g = c.benchmark_group("feed_parse");
    g.throughput(Throughput::Bytes(RSS.len() as u64));
    g.bench_function("rss", |b| {
        b.iter(|| {
            let feed = parser::parse(black_box(RSS)).unwrap();
            black_box(feed);
        })
    });
    g.finish();
}

criterion_group!(feeds_benches, bench_parse);
criterion_main!(feeds_benches);
