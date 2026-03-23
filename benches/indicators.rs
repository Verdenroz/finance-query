use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use finance_query::Candle;
use finance_query::indicators::{
    accumulation_distribution, adx, alma, aroon, atr, awesome_oscillator, balance_of_power,
    bollinger_bands, bull_bear_power, cci, chaikin_oscillator, choppiness_index, cmf, cmo,
    coppock_curve, dema, donchian_channels, elder_ray, ema, hma, ichimoku, keltner_channels, macd,
    mcginley_dynamic, mfi, momentum, obv, parabolic_sar, patterns, roc, rsi, sma, stochastic,
    stochastic_rsi, supertrend, tema, true_range, vwap, vwma, williams_r, wma,
};
use std::hint::black_box;

/// Generate deterministic synthetic OHLCV candles.
///
/// Candle is `#[non_exhaustive]` outside the crate — constructed via serde_json.
/// Oscillating price data ensures non-trivial indicator output.
fn synthetic_candles(n: usize) -> Vec<Candle> {
    let mut price = 100.0_f64;
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let phase = (i as f64) * 0.1;
        let swing = 5.0 * phase.sin();
        let open = price;
        let close = price + swing + (i as f64) * 0.01;
        let high = open.max(close) + 1.0 + (phase * 0.5).abs();
        let low = open.min(close) - 1.0 - (phase * 0.5).abs();
        let volume = (1_000_000.0 + 100_000.0 * (phase * 2.0).sin()) as i64;
        let candle: Candle = serde_json::from_value(serde_json::json!({
            "timestamp": 1_700_000_000_i64 + i as i64 * 86400,
            "open": open, "high": high, "low": low, "close": close,
            "volume": volume, "adjClose": close
        }))
        .unwrap();
        out.push(candle);
        price = close;
    }
    out
}

// ── Moving Averages ──────────────────────────────────────────────────────────

fn bench_moving_averages(c: &mut Criterion) {
    let candles = synthetic_candles(1000);
    let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
    let volumes: Vec<f64> = candles.iter().map(|c| c.volume as f64).collect();

    let mut group = c.benchmark_group("moving_averages");

    group.bench_function("sma_20", |b| b.iter(|| sma(black_box(&closes), 20)));
    group.bench_function("sma_200", |b| b.iter(|| sma(black_box(&closes), 200)));
    group.bench_function("ema_20", |b| b.iter(|| ema(black_box(&closes), 20)));
    group.bench_function("ema_200", |b| b.iter(|| ema(black_box(&closes), 200)));
    group.bench_function("wma_20", |b| b.iter(|| wma(black_box(&closes), 20)));
    group.bench_function("hma_20", |b| b.iter(|| hma(black_box(&closes), 20)));
    group.bench_function("dema_20", |b| b.iter(|| dema(black_box(&closes), 20)));
    group.bench_function("tema_20", |b| b.iter(|| tema(black_box(&closes), 20)));
    group.bench_function("alma_9", |b| b.iter(|| alma(black_box(&closes), 9, 0.85, 6.0)));
    group.bench_function("mcginley_20", |b| b.iter(|| mcginley_dynamic(black_box(&closes), 20)));
    group.bench_function("vwma_20", |b| {
        b.iter(|| vwma(black_box(&closes), black_box(&volumes), 20))
    });

    group.finish();
}

// ── Momentum Oscillators ─────────────────────────────────────────────────────

fn bench_momentum(c: &mut Criterion) {
    let candles = synthetic_candles(1000);
    let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
    let highs: Vec<f64> = candles.iter().map(|c| c.high).collect();
    let lows: Vec<f64> = candles.iter().map(|c| c.low).collect();

    let mut group = c.benchmark_group("momentum");

    group.bench_function("rsi_14", |b| b.iter(|| rsi(black_box(&closes), 14)));
    group.bench_function("stochastic", |b| {
        b.iter(|| stochastic(black_box(&highs), black_box(&lows), black_box(&closes), 14, 1, 3))
    });
    group.bench_function("stochastic_rsi", |b| {
        b.iter(|| stochastic_rsi(black_box(&closes), 14, 14, 3, 3))
    });
    group.bench_function("cci_20", |b| {
        b.iter(|| cci(black_box(&highs), black_box(&lows), black_box(&closes), 20))
    });
    group.bench_function("macd", |b| b.iter(|| macd(black_box(&closes), 12, 26, 9)));
    group.bench_function("williams_r", |b| {
        b.iter(|| williams_r(black_box(&highs), black_box(&lows), black_box(&closes), 14))
    });
    group.bench_function("roc_12", |b| b.iter(|| roc(black_box(&closes), 12)));
    group.bench_function("momentum_10", |b| b.iter(|| momentum(black_box(&closes), 10)));
    group.bench_function("cmo_14", |b| b.iter(|| cmo(black_box(&closes), 14)));
    group.bench_function("awesome_oscillator", |b| {
        b.iter(|| awesome_oscillator(black_box(&highs), black_box(&lows), 5, 34))
    });
    group.bench_function("coppock_curve", |b| {
        b.iter(|| coppock_curve(black_box(&closes), 14, 11, 10))
    });

    group.finish();
}

// ── Trend Indicators ─────────────────────────────────────────────────────────

fn bench_trend(c: &mut Criterion) {
    let candles = synthetic_candles(1000);
    let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
    let highs: Vec<f64> = candles.iter().map(|c| c.high).collect();
    let lows: Vec<f64> = candles.iter().map(|c| c.low).collect();

    let mut group = c.benchmark_group("trend");

    group.bench_function("adx_14", |b| {
        b.iter(|| adx(black_box(&highs), black_box(&lows), black_box(&closes), 14))
    });
    group.bench_function("aroon_25", |b| {
        b.iter(|| aroon(black_box(&highs), black_box(&lows), 25))
    });
    group.bench_function("supertrend", |b| {
        b.iter(|| supertrend(black_box(&highs), black_box(&lows), black_box(&closes), 10, 3.0))
    });
    group.bench_function("ichimoku", |b| {
        b.iter(|| ichimoku(black_box(&highs), black_box(&lows), black_box(&closes), 9, 26, 26, 26))
    });
    group.bench_function("parabolic_sar", |b| {
        b.iter(|| parabolic_sar(black_box(&highs), black_box(&lows), black_box(&closes), 0.02, 0.2))
    });
    group.bench_function("bull_bear_power", |b| {
        b.iter(|| bull_bear_power(black_box(&highs), black_box(&lows), black_box(&closes), 13))
    });
    group.bench_function("elder_ray", |b| {
        b.iter(|| elder_ray(black_box(&highs), black_box(&lows), black_box(&closes), 13))
    });

    group.finish();
}

// ── Volatility Indicators ────────────────────────────────────────────────────

fn bench_volatility(c: &mut Criterion) {
    let candles = synthetic_candles(1000);
    let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
    let highs: Vec<f64> = candles.iter().map(|c| c.high).collect();
    let lows: Vec<f64> = candles.iter().map(|c| c.low).collect();

    let mut group = c.benchmark_group("volatility");

    group.bench_function("bollinger_20", |b| {
        b.iter(|| bollinger_bands(black_box(&closes), 20, 2.0))
    });
    group.bench_function("keltner_20", |b| {
        b.iter(|| keltner_channels(black_box(&highs), black_box(&lows), black_box(&closes), 20, 10, 2.0))
    });
    group.bench_function("donchian_20", |b| {
        b.iter(|| donchian_channels(black_box(&highs), black_box(&lows), 20))
    });
    group.bench_function("atr_14", |b| {
        b.iter(|| atr(black_box(&highs), black_box(&lows), black_box(&closes), 14))
    });
    group.bench_function("true_range", |b| {
        b.iter(|| true_range(black_box(&highs), black_box(&lows), black_box(&closes)))
    });
    group.bench_function("choppiness_14", |b| {
        b.iter(|| choppiness_index(black_box(&highs), black_box(&lows), black_box(&closes), 14))
    });

    group.finish();
}

// ── Volume Indicators ────────────────────────────────────────────────────────

fn bench_volume(c: &mut Criterion) {
    let candles = synthetic_candles(1000);
    let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
    let highs: Vec<f64> = candles.iter().map(|c| c.high).collect();
    let lows: Vec<f64> = candles.iter().map(|c| c.low).collect();
    let opens: Vec<f64> = candles.iter().map(|c| c.open).collect();
    let volumes: Vec<f64> = candles.iter().map(|c| c.volume as f64).collect();

    let mut group = c.benchmark_group("volume");

    group.bench_function("obv", |b| {
        b.iter(|| obv(black_box(&closes), black_box(&volumes)))
    });
    group.bench_function("mfi_14", |b| {
        b.iter(|| mfi(black_box(&highs), black_box(&lows), black_box(&closes), black_box(&volumes), 14))
    });
    group.bench_function("cmf_20", |b| {
        b.iter(|| cmf(black_box(&highs), black_box(&lows), black_box(&closes), black_box(&volumes), 20))
    });
    group.bench_function("chaikin_oscillator", |b| {
        b.iter(|| chaikin_oscillator(black_box(&highs), black_box(&lows), black_box(&closes), black_box(&volumes)))
    });
    group.bench_function("accumulation_distribution", |b| {
        b.iter(|| accumulation_distribution(black_box(&highs), black_box(&lows), black_box(&closes), black_box(&volumes)))
    });
    group.bench_function("vwap", |b| {
        b.iter(|| vwap(black_box(&highs), black_box(&lows), black_box(&closes), black_box(&volumes)))
    });
    group.bench_function("balance_of_power", |b| {
        b.iter(|| balance_of_power(black_box(&opens), black_box(&highs), black_box(&lows), black_box(&closes), None))
    });

    group.finish();
}

// ── Candlestick Patterns ─────────────────────────────────────────────────────

fn bench_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("patterns");

    for n in [100usize, 500, 1000] {
        let candles = synthetic_candles(n);
        group.bench_with_input(BenchmarkId::new("patterns", n), &candles, |b, candles| {
            b.iter(|| patterns(black_box(candles)))
        });
    }

    group.finish();
}

// ── Full Indicator Suite (equivalent to IndicatorsSummary workload) ───────────

fn bench_full_suite(c: &mut Criterion) {
    let mut group = c.benchmark_group("indicators_summary_equivalent");

    for n in [100usize, 500, 1000] {
        let candles = synthetic_candles(n);
        group.bench_with_input(BenchmarkId::new("all_indicators", n), &candles, |b, candles| {
            b.iter(|| {
                let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
                let highs: Vec<f64> = candles.iter().map(|c| c.high).collect();
                let lows: Vec<f64> = candles.iter().map(|c| c.low).collect();
                let opens: Vec<f64> = candles.iter().map(|c| c.open).collect();
                let volumes: Vec<f64> = candles.iter().map(|c| c.volume as f64).collect();

                black_box(sma(&closes, 10));
                black_box(sma(&closes, 20));
                black_box(sma(&closes, 50));
                black_box(sma(&closes, 100));
                black_box(sma(&closes, 200));
                black_box(ema(&closes, 10));
                black_box(ema(&closes, 20));
                black_box(ema(&closes, 50));
                black_box(ema(&closes, 100));
                black_box(ema(&closes, 200));
                let _ = black_box(wma(&closes, 10));
                let _ = black_box(wma(&closes, 20));
                let _ = black_box(wma(&closes, 50));
                let _ = black_box(dema(&closes, 20));
                let _ = black_box(tema(&closes, 20));
                let _ = black_box(hma(&closes, 20));
                let _ = black_box(vwma(&closes, &volumes, 20));
                let _ = black_box(alma(&closes, 9, 0.85, 6.0));
                let _ = black_box(mcginley_dynamic(&closes, 20));
                let _ = black_box(rsi(&closes, 14));
                let _ = black_box(stochastic(&highs, &lows, &closes, 14, 1, 3));
                let _ = black_box(stochastic_rsi(&closes, 14, 14, 3, 3));
                let _ = black_box(cci(&highs, &lows, &closes, 20));
                let _ = black_box(williams_r(&highs, &lows, &closes, 14));
                let _ = black_box(roc(&closes, 12));
                let _ = black_box(momentum(&closes, 10));
                let _ = black_box(cmo(&closes, 14));
                let _ = black_box(awesome_oscillator(&highs, &lows, 5, 34));
                let _ = black_box(coppock_curve(&closes, 14, 11, 10));
                let _ = black_box(macd(&closes, 12, 26, 9));
                let _ = black_box(adx(&highs, &lows, &closes, 14));
                let _ = black_box(aroon(&highs, &lows, 25));
                let _ = black_box(supertrend(&highs, &lows, &closes, 10, 3.0));
                let _ = black_box(ichimoku(&highs, &lows, &closes, 9, 26, 26, 26));
                let _ = black_box(parabolic_sar(&highs, &lows, &closes, 0.02, 0.2));
                let _ = black_box(bull_bear_power(&highs, &lows, &closes, 13));
                let _ = black_box(elder_ray(&highs, &lows, &closes, 13));
                let _ = black_box(bollinger_bands(&closes, 20, 2.0));
                let _ = black_box(keltner_channels(&highs, &lows, &closes, 20, 10, 2.0));
                let _ = black_box(donchian_channels(&highs, &lows, 20));
                let _ = black_box(atr(&highs, &lows, &closes, 14));
                let _ = black_box(true_range(&highs, &lows, &closes));
                let _ = black_box(choppiness_index(&highs, &lows, &closes, 14));
                let _ = black_box(obv(&closes, &volumes));
                let _ = black_box(mfi(&highs, &lows, &closes, &volumes, 14));
                let _ = black_box(cmf(&highs, &lows, &closes, &volumes, 20));
                let _ = black_box(chaikin_oscillator(&highs, &lows, &closes, &volumes));
                let _ = black_box(accumulation_distribution(&highs, &lows, &closes, &volumes));
                let _ = black_box(vwap(&highs, &lows, &closes, &volumes));
                let _ = black_box(balance_of_power(&opens, &highs, &lows, &closes, None));
            })
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_moving_averages,
    bench_momentum,
    bench_trend,
    bench_volatility,
    bench_volume,
    bench_patterns,
    bench_full_suite,
);
criterion_main!(benches);
