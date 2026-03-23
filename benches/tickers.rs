/// Benchmarks for batch Tickers operations.
///
/// Focused on the response-building hot path where Arc<str> symbols are
/// converted to String for HashMap keys — the primary optimization target.
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::collections::HashMap;
use std::hint::black_box;
use std::sync::Arc;

// ── Symbol key allocation: Arc<str> → String ─────────────────────────────────
//
// This is the hot path in tickers/core.rs lines 470, 707, 751, etc.
// Every cached symbol gets `.to_string()` when building the BatchXxxResponse.
//
// After the fix, we'll use the Arc<str> directly or avoid the conversion.

fn bench_response_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_response_build");

    for n in [10usize, 50, 100] {
        // Simulate the Arc<str> symbols stored in Tickers
        let symbols: Vec<Arc<str>> = (0..n).map(|i| format!("SYM{i:04}").into()).collect();
        let values: Vec<f64> = (0..n).map(|i| 100.0 + i as f64).collect();

        // CURRENT: symbol.to_string() per insert — what the code does now
        group.bench_with_input(
            BenchmarkId::new("arc_to_string_hashmap", n),
            &(&symbols, &values),
            |b, (syms, vals)| {
                b.iter(|| {
                    let mut map: HashMap<String, f64> = HashMap::with_capacity(syms.len());
                    for (sym, &val) in syms.iter().zip(vals.iter()) {
                        map.insert(sym.to_string(), val);
                    }
                    black_box(map)
                })
            },
        );

        // ALTERNATIVE A: pre-convert to String once, reuse — no-alloc in the loop
        group.bench_with_input(
            BenchmarkId::new("preconverted_string_hashmap", n),
            &(&symbols, &values),
            |b, (syms, vals)| {
                // Strings already owned (simulates if we stored Vec<String> instead of Vec<Arc<str>>)
                let strings: Vec<String> = syms.iter().map(|s| s.to_string()).collect();
                b.iter(|| {
                    let mut map: HashMap<String, f64> = HashMap::with_capacity(strings.len());
                    for (sym, &val) in strings.iter().zip(vals.iter()) {
                        map.insert(sym.clone(), val);
                    }
                    black_box(map)
                })
            },
        );

        // ALTERNATIVE B: Arc<str> keys — avoids String alloc entirely
        group.bench_with_input(
            BenchmarkId::new("arc_str_hashmap", n),
            &(&symbols, &values),
            |b, (syms, vals)| {
                b.iter(|| {
                    let mut map: HashMap<Arc<str>, f64> = HashMap::with_capacity(syms.len());
                    for (sym, &val) in syms.iter().zip(vals.iter()) {
                        map.insert(Arc::clone(sym), val);
                    }
                    black_box(map)
                })
            },
        );
    }

    group.finish();
}

// ── Symbols join for HTTP params ──────────────────────────────────────────────
//
// symbols.join(",") is called on every batch fetch (quotes.rs:37, etc.)

fn bench_symbols_join(c: &mut Criterion) {
    let mut group = c.benchmark_group("symbols_join");

    for n in [10usize, 50, 100] {
        let symbols: Vec<Arc<str>> = (0..n).map(|i| format!("SYM{i:04}").into()).collect();

        // Current: collect refs then join
        group.bench_with_input(BenchmarkId::new("collect_join", n), &symbols, |b, syms| {
            b.iter(|| {
                let refs: Vec<&str> = syms.iter().map(|s| &**s).collect();
                black_box(refs.join(","))
            })
        });

        // Alternative: itertools-style fold (avoids intermediate Vec allocation)
        group.bench_with_input(BenchmarkId::new("fold_join", n), &symbols, |b, syms| {
            b.iter(|| {
                let joined = syms.iter().enumerate().fold(
                    String::with_capacity(n * 6),
                    |mut acc, (i, s)| {
                        if i > 0 {
                            acc.push(',');
                        }
                        acc.push_str(s);
                        acc
                    },
                );
                black_box(joined)
            })
        });
    }

    group.finish();
}

// ── Concurrent cache read under contention ───────────────────────────────────

fn bench_concurrent_cache(c: &mut Criterion) {
    use std::sync::Arc;
    use tokio::sync::RwLock;

    let rt = tokio::runtime::Runtime::new().unwrap();

    // Simulate QuoteCache: Arc<RwLock<HashMap<Arc<str>, f64>>>
    let cache: Arc<RwLock<HashMap<Arc<str>, f64>>> = Arc::new(RwLock::new(HashMap::new()));

    // Pre-populate
    rt.block_on(async {
        let mut map = cache.write().await;
        for i in 0..100usize {
            let key: Arc<str> = format!("SYM{i:04}").into();
            map.insert(key, 100.0 + i as f64);
        }
    });

    let mut group = c.benchmark_group("concurrent_cache_read");

    // all_cached check — done on fast path before any network I/O
    let symbols_10: Vec<Arc<str>> = (0..10usize).map(|i| format!("SYM{i:04}").into()).collect();
    let symbols_100: Vec<Arc<str>> = (0..100usize).map(|i| format!("SYM{i:04}").into()).collect();

    group.bench_function("all_cached_check_10", |b| {
        b.iter(|| {
            rt.block_on(async {
                let map = cache.read().await;
                let all_hit = symbols_10.iter().all(|s| map.contains_key(s));
                black_box(all_hit)
            })
        })
    });

    group.bench_function("all_cached_check_100", |b| {
        b.iter(|| {
            rt.block_on(async {
                let map = cache.read().await;
                let all_hit = symbols_100.iter().all(|s| map.contains_key(s));
                black_box(all_hit)
            })
        })
    });

    // Build response: iterate over cache hits (current code with to_string)
    group.bench_function("build_response_100_to_string", |b| {
        b.iter(|| {
            rt.block_on(async {
                let map = cache.read().await;
                let mut response: HashMap<String, f64> = HashMap::with_capacity(100);
                for sym in &symbols_100 {
                    if let Some(&v) = map.get(sym) {
                        response.insert(sym.to_string(), v);
                    }
                }
                black_box(response)
            })
        })
    });

    // Build response: Arc<str> keys (no String alloc)
    group.bench_function("build_response_100_arc_keys", |b| {
        b.iter(|| {
            rt.block_on(async {
                let map = cache.read().await;
                let mut response: HashMap<Arc<str>, f64> = HashMap::with_capacity(100);
                for sym in &symbols_100 {
                    if let Some(&v) = map.get(sym) {
                        response.insert(Arc::clone(sym), v);
                    }
                }
                black_box(response)
            })
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_response_build,
    bench_symbols_join,
    bench_concurrent_cache,
);
criterion_main!(benches);
