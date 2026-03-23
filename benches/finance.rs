use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use finance_query::{
    Currency, FearAndGreed, MarketHours, MarketSummaryQuote, News, SearchResults, TrendingQuote,
};

static SEARCH_JSON: &str = include_str!("fixtures/search.json");
static MARKET_SUMMARY_JSON: &str = include_str!("fixtures/market_summary.json");
static TRENDING_JSON: &str = include_str!("fixtures/trending.json");
static FEAR_AND_GREED_JSON: &str = include_str!("fixtures/fear_and_greed.json");
static HOURS_JSON: &str = include_str!("fixtures/hours.json");
static NEWS_JSON: &str = include_str!("fixtures/news.json");
static CURRENCIES_JSON: &str = include_str!("fixtures/currencies.json");

fn bench_search_deserialize(c: &mut Criterion) {
    c.bench_function("search_deserialize_6_quotes", |b| {
        b.iter(|| {
            let result: SearchResults = serde_json::from_str(black_box(SEARCH_JSON)).unwrap();
            black_box(result);
        })
    });
}

fn bench_market_summary_deserialize(c: &mut Criterion) {
    c.bench_function("market_summary_deserialize_15_items", |b| {
        b.iter(|| {
            let result: Vec<MarketSummaryQuote> =
                serde_json::from_str(black_box(MARKET_SUMMARY_JSON)).unwrap();
            black_box(result);
        })
    });
}

fn bench_trending_deserialize(c: &mut Criterion) {
    c.bench_function("trending_deserialize_20_items", |b| {
        b.iter(|| {
            let result: Vec<TrendingQuote> =
                serde_json::from_str(black_box(TRENDING_JSON)).unwrap();
            black_box(result);
        })
    });
}

fn bench_fear_and_greed_deserialize(c: &mut Criterion) {
    c.bench_function("fear_and_greed_deserialize", |b| {
        b.iter(|| {
            let result: FearAndGreed =
                serde_json::from_str(black_box(FEAR_AND_GREED_JSON)).unwrap();
            black_box(result);
        })
    });
}

fn bench_hours_deserialize(c: &mut Criterion) {
    c.bench_function("hours_deserialize", |b| {
        b.iter(|| {
            let result: MarketHours = serde_json::from_str(black_box(HOURS_JSON)).unwrap();
            black_box(result);
        })
    });
}

fn bench_news_deserialize(c: &mut Criterion) {
    c.bench_function("news_deserialize_10_items", |b| {
        b.iter(|| {
            let result: Vec<News> = serde_json::from_str(black_box(NEWS_JSON)).unwrap();
            black_box(result);
        })
    });
}

fn bench_currencies_deserialize(c: &mut Criterion) {
    c.bench_function("currencies_deserialize_168_items", |b| {
        b.iter(|| {
            let result: Vec<Currency> = serde_json::from_str(black_box(CURRENCIES_JSON)).unwrap();
            black_box(result);
        })
    });
}

fn bench_serialize(c: &mut Criterion) {
    let market_summary: Vec<MarketSummaryQuote> =
        serde_json::from_str(MARKET_SUMMARY_JSON).unwrap();
    let currencies: Vec<Currency> = serde_json::from_str(CURRENCIES_JSON).unwrap();
    let news: Vec<News> = serde_json::from_str(NEWS_JSON).unwrap();
    let search: SearchResults = serde_json::from_str(SEARCH_JSON).unwrap();

    let mut group = c.benchmark_group("serialize");

    group.bench_function(BenchmarkId::new("market_summary", "15_items"), |b| {
        b.iter(|| {
            let json = serde_json::to_string(black_box(&market_summary)).unwrap();
            black_box(json);
        })
    });

    group.bench_function(BenchmarkId::new("currencies", "168_items"), |b| {
        b.iter(|| {
            let json = serde_json::to_string(black_box(&currencies)).unwrap();
            black_box(json);
        })
    });

    group.bench_function(BenchmarkId::new("news", "10_items"), |b| {
        b.iter(|| {
            let json = serde_json::to_string(black_box(&news)).unwrap();
            black_box(json);
        })
    });

    group.bench_function(BenchmarkId::new("search_results", "6_quotes"), |b| {
        b.iter(|| {
            let json = serde_json::to_string(black_box(&search)).unwrap();
            black_box(json);
        })
    });

    group.finish();
}

criterion_group!(
    finance_benches,
    bench_search_deserialize,
    bench_market_summary_deserialize,
    bench_trending_deserialize,
    bench_fear_and_greed_deserialize,
    bench_hours_deserialize,
    bench_news_deserialize,
    bench_currencies_deserialize,
    bench_serialize,
);
criterion_main!(finance_benches);
