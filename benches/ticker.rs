/// Benchmarks for the single-Ticker hot paths.
///
/// Since Ticker's cache fields are private, these benchmarks target the
/// observable components: Arc<str> symbol handling, RwLock read contention
/// under concurrent access, and the OnceLock module-list construction.
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::collections::HashMap;
use std::hint::black_box;
use std::sync::Arc;
use tokio::sync::RwLock;

// ── Arc<str> symbol operations ────────────────────────────────────────────────

fn bench_symbol_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("symbol_ops");

    // Cost of constructing Arc<str> from a symbol string (builder pattern does this once)
    group.bench_function("arc_str_from_literal", |b| {
        b.iter(|| {
            let s: Arc<str> = black_box("AAPL").into();
            black_box(s)
        })
    });

    // Cost of cloning Arc<str> — what the cache read path does for each field
    let symbol: Arc<str> = "AAPL".into();
    group.bench_function("arc_str_clone", |b| {
        b.iter(|| black_box(Arc::clone(&symbol)))
    });

    // Cost of Arc<str> → String conversion (the hot path we're eliminating)
    group.bench_function("arc_str_to_string", |b| {
        b.iter(|| black_box(symbol.to_string()))
    });

    // Cost of Arc<str> → &str → String (alternative — same alloc but avoids Arc deref)
    group.bench_function("arc_str_via_ref", |b| {
        b.iter(|| black_box(String::from(&*symbol)))
    });

    group.finish();
}

// ── RwLock read path (warm cache) ────────────────────────────────────────────

fn bench_rwlock_read(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let cache: Arc<RwLock<Option<f64>>> = Arc::new(RwLock::new(Some(123.45)));

    let mut group = c.benchmark_group("rwlock_cache_read");

    // Single read — models a single quote field accessor
    group.bench_function("single_read", |b| {
        b.iter(|| {
            rt.block_on(async {
                let guard = cache.read().await;
                black_box(*guard)
            })
        })
    });

    // 30 sequential reads — models reading all quote modules in one cached call
    group.bench_function("30_sequential_reads", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut sum = 0.0_f64;
                for _ in 0..30 {
                    let guard = cache.read().await;
                    if let Some(v) = *guard {
                        sum += v;
                    }
                }
                black_box(sum)
            })
        })
    });

    group.finish();
}

// ── HashMap cache lookup (MapCache hot path) ─────────────────────────────────

type ChartCache = Arc<RwLock<HashMap<(Arc<str>, u8, u8), f64>>>;

fn bench_map_cache_lookup(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Simulate the chart MapCache: keyed by (Arc<str>, u8, u8) tuples
    let cache: ChartCache = Arc::new(RwLock::new(HashMap::new()));

    // Pre-populate with entries
    rt.block_on(async {
        let mut map = cache.write().await;
        for i in 0..10u8 {
            let key: Arc<str> = format!("SYM{i}").into();
            map.insert((key, i, i), 100.0 + i as f64);
        }
    });

    let lookup_key: Arc<str> = "SYM5".into();

    let mut group = c.benchmark_group("map_cache_lookup");

    group.bench_function("hit", |b| {
        b.iter(|| {
            rt.block_on(async {
                let guard = cache.read().await;
                black_box(guard.get(&(Arc::clone(&lookup_key), 5u8, 5u8)).copied())
            })
        })
    });

    group.bench_function("miss", |b| {
        let miss_key: Arc<str> = "MISSING".into();
        b.iter(|| {
            rt.block_on(async {
                let guard = cache.read().await;
                black_box(guard.get(&(Arc::clone(&miss_key), 0u8, 0u8)).copied())
            })
        })
    });

    group.finish();
}

// ── Symbol list operations (used in Ticker builder) ──────────────────────────

fn bench_symbol_list(c: &mut Criterion) {
    let mut group = c.benchmark_group("symbol_list");

    // Building a &[&str] ref slice from Arc<str> vec — done before every batch HTTP call
    for n in [10usize, 50, 100] {
        let symbols: Vec<Arc<str>> = (0..n).map(|i| format!("SYM{i:04}").into()).collect();

        group.bench_with_input(BenchmarkId::new("refs_slice", n), &symbols, |b, syms| {
            b.iter(|| {
                let refs: Vec<&str> = syms.iter().map(|s| &**s).collect();
                black_box(refs)
            })
        });

        group.bench_with_input(BenchmarkId::new("join_comma", n), &symbols, |b, syms| {
            b.iter(|| {
                let joined: String = syms.iter().map(|s| &**s).collect::<Vec<_>>().join(",");
                black_box(joined)
            })
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_symbol_operations,
    bench_rwlock_read,
    bench_map_cache_lookup,
    bench_symbol_list,
);
criterion_main!(benches);
