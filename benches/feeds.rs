//! RSS/Atom feed-parsing benchmark.
//!
//! Mirrors the parse path in `src/feeds/mod.rs` (`feeds::parse_bytes` over the
//! fetched bytes) without any network: the fixtures are offline XML in
//! `benches/fixtures/`. Measures the cost that dominates `feeds::fetch` once the
//! bytes are in hand.
//!
//! ```text
//! cargo bench --bench feeds
//! ```

use criterion::{Criterion, Throughput, black_box, criterion_group, criterion_main};
use finance_query::feeds;

static RSS: &[u8] = include_bytes!("fixtures/feed_rss.xml");

fn bench_parse(c: &mut Criterion) {
    let mut g = c.benchmark_group("feed_parse");
    g.throughput(Throughput::Bytes(RSS.len() as u64));
    g.bench_function("rss", |b| {
        b.iter(|| {
            let entries = feeds::parse_bytes(black_box(RSS), "bench").unwrap();
            black_box(entries);
        })
    });
    g.finish();
}

criterion_group!(feeds_benches, bench_parse);
criterion_main!(feeds_benches);
