//! Measured regression gate (`cargo soothfast`).
//!
//! Replaces the iai-callgrind/valgrind gate: `cargo soothfast measure` counts
//! per-iteration CPU instructions natively via perf_event (plus allocations
//! and walltime), and `cargo soothfast gate` fails CI when any benchmark
//! regresses against the PR's merge-base, measured live in a temporary
//! worktree — no committed baseline, no valgrind, no container.
//!
//! The gate spans every compute/parse/dispatch facet of the library:
//! indicators, the backtesting engine, risk analytics, streaming
//! (de)serialize, provider capability dispatch, model (de)serialization, and
//! `Ticker`/`Tickers` cache behavior (via a canned, network-free
//! `ProviderAdapter` — see `CannedProvider` below). The `benches/ticker.rs` /
//! `benches/tickers.rs` criterion benches stay separate: they compare raw
//! `std` container micro-ops (`Arc<str>` clone, `RwLock` read) that model the
//! cache's *internal allocation strategy*, not the library's own logic.
//!
//! ## Running
//!
//! ```text
//! cargo soothfast measure --features bench-gate --save-baseline base
//! cargo soothfast gate    --features bench-gate --against-ref origin/master
//! cargo bench --bench soothfast --features bench-gate    # raw runner output
//! ```
//!
//! Assertions on the annotations (`alloc = N`, `complexity = "..."`) are
//! checked on every run; a run with failing assertions is never saved as a
//! baseline.

use finance_query::Capability;
use finance_query::backtesting::condition::{
    Condition, always_false, always_true, has_position, held_for_bars, in_loss, in_profit, is_long,
    is_short, no_position, stop_loss, take_profit, trailing_stop, trailing_take_profit,
};
use finance_query::backtesting::refs::{
    IndicatorRefExt, accumulation_distribution as ref_accumulation_distribution, adx as ref_adx,
    alma as ref_alma, aroon as ref_aroon, atr as ref_atr,
    awesome_oscillator as ref_awesome_oscillator, balance_of_power as ref_balance_of_power,
    bear_power as ref_bear_power, bollinger as ref_bollinger, bull_power as ref_bull_power,
    candle_body, candle_range, cci as ref_cci, chaikin_oscillator as ref_chaikin_oscillator,
    choppiness_index as ref_choppiness_index, close, cmf as ref_cmf, cmo as ref_cmo,
    coppock_curve as ref_coppock_curve, dema as ref_dema, donchian as ref_donchian,
    elder_bear_power as ref_elder_bear_power, elder_bull_power as ref_elder_bull_power,
    ema as ref_ema, gap_pct, high, hma as ref_hma, htf, htf_region, ichimoku as ref_ichimoku,
    ichimoku_custom as ref_ichimoku_custom, is_bearish, is_bullish, keltner as ref_keltner, low,
    macd as ref_macd, mcginley as ref_mcginley, median_price, mfi as ref_mfi,
    momentum as ref_momentum, obv as ref_obv, open, parabolic_sar as ref_parabolic_sar, price,
    price_change_pct, relative_volume, roc as ref_roc, rsi as ref_rsi, sma as ref_sma,
    stochastic as ref_stochastic, stochastic_rsi as ref_stochastic_rsi,
    supertrend as ref_supertrend, tema as ref_tema, true_range as ref_true_range, typical_price,
    volume, vwap as ref_vwap, vwma as ref_vwma, williams_r as ref_williams_r, wma as ref_wma,
};
use finance_query::backtesting::resample::{base_to_htf_index, resample};
use finance_query::backtesting::{
    BacktestConfig, BacktestEngine, BayesianSearch, GridSearch, MonteCarloConfig, OptimizeMetric,
    ParamRange, PositionSizing, SmaCrossover, StrategyBuilder,
};
use finance_query::crypto::CoinQuote;
use finance_query::fred::{MacroSeries, TreasuryYield};
use finance_query::indicators::{
    accumulation_distribution, adx, alma, aroon, atr, awesome_oscillator, balance_of_power,
    bollinger_bands, bull_bear_power, cci, chaikin_oscillator, choppiness_index, cmf, cmo,
    coppock_curve, dema, donchian_channels, elder_ray, ema, fibonacci_pivot_points,
    fibonacci_retracement, heikin_ashi, hma, ichimoku, keltner_channels, last_value, macd,
    mcginley_dynamic, mfi, momentum, obv, parabolic_sar, patterns, pivot_points, roc, rsi, sma,
    stochastic, stochastic_rsi, supertrend, tema, true_range, vwap, vwma, williams_r, wma, zigzag,
};
use finance_query::risk::{
    beta, calmar_ratio, historical_cvar, historical_var, information_ratio, kelly_criterion,
    max_drawdown, omega_ratio, parametric_cvar, parametric_var, sharpe_ratio, sortino_ratio,
    tracking_error, ulcer_index, win_loss_stats,
};
use finance_query::streaming::{MarketHoursType, OptionType, PriceUpdate, QuoteType};
use finance_query::translation::{
    Lang, TranslationBackend, set_backend, translate, translate_texts, translate_with,
};
use finance_query::{
    Candle, Chart, CompanyFacts, Currency, EdgarSubmissions, FearAndGreed, Fetch,
    FinancialStatement, Interval, MarketHours, MarketSummaryQuote, News, Options, Provider,
    ProviderAdapter, ProviderSet, Quote, QuoteSummaryResponse, Region, Routes, ScreenerResults,
    SearchResults, Ticker, Tickers, TimeRange, Transcript, TrendingQuote, analyze_sentiment,
};
use soothfast::{bench, fixture, keep};
use std::sync::Arc;

soothfast::bench_main!();

// ── Deterministic synthetic inputs (no network, no randomness) ───────────────

/// A reproducible pseudo-random walk of close prices.
#[fixture]
fn synthetic_closes(n: usize) -> Vec<f64> {
    let mut price = 100.0_f64;
    let mut state = 0x2545_F491_4F6C_DD1D_u64;
    (0..n)
        .map(|_| {
            // xorshift — deterministic, branch-light
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let step = ((state >> 40) as f64 / (1u64 << 24) as f64 - 0.5) * 2.0;
            price = (price + step).max(1.0);
            price
        })
        .collect()
}

/// Close-to-close simple returns derived from a synthetic price series.
#[fixture]
fn synthetic_returns(n: usize) -> Vec<f64> {
    let closes = synthetic_closes(n + 1);
    closes.windows(2).map(|w| (w[1] - w[0]) / w[0]).collect()
}

/// Deterministic synthetic OHLCV candles. `Candle` is `#[non_exhaustive]`
/// outside the crate, so construct via `serde_json` (matches benches/backtesting.rs).
fn synthetic_candles(n: usize) -> Vec<Candle> {
    let closes = synthetic_closes(n);
    closes
        .iter()
        .enumerate()
        .map(|(i, &close)| {
            serde_json::from_value(serde_json::json!({
                "timestamp": 1_700_000_000_i64 + i as i64 * 86_400,
                "open": close,
                "high": close + 1.0,
                "low": close - 1.0,
                "close": close,
                "volume": 1_000_000_i64,
                "adjClose": close,
            }))
            .unwrap()
        })
        .collect()
}

#[fixture]
fn returns_1000() -> Vec<f64> {
    synthetic_returns(1000)
}

/// Feeds `risk_beta` — both series scale with `n`.
fn synthetic_returns_pair(n: usize) -> (Vec<f64>, Vec<f64>) {
    (synthetic_returns(n), synthetic_returns(n))
}

// ── Indicator hot paths ──────────────────────────────────────────────────────
//
// The gate spans the whole indicator suite (all 42), grouped by family so a
// regression in any single indicator pushes its family's instruction count
// past the gate threshold. Families mirror `src/indicators/` and the criterion
// bench `benches/indicators.rs`. The per-family setup extracts O/H/L/C/V
// arrays once, outside the measured region.

/// `(opens, highs, lows, closes, volumes)` — extracted from synthetic candles.
type Series = (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>);

#[fixture]
fn series_1000() -> Series {
    let candles = synthetic_candles(1000);
    let opens = candles.iter().map(|c| c.open).collect();
    let highs = candles.iter().map(|c| c.high).collect();
    let lows = candles.iter().map(|c| c.low).collect();
    let closes = candles.iter().map(|c| c.close).collect();
    let volumes = candles.iter().map(|c| c.volume as f64).collect();
    (opens, highs, lows, closes, volumes)
}

/// Single representative indicator with verified claims: SMA is a
/// rolling-sum O(n) pass allocating only the result vector.
#[bench(
    group = "indicators",
    setup_sized = synthetic_closes,
    sizes(1000, 10000, 100000),
    complexity = "n",
    alloc = 1,
    p99 = "1ms",
    covers = "finance_query::indicators::sma"
)]
fn ind_sma(closes: &[f64]) {
    keep(sma(keep(closes), 20));
}

#[bench(group = "indicators", setup = series_1000, covers = "finance_query::indicators::ema")]
fn ind_moving_averages(s: &Series) {
    let (_, _, _, closes, volumes) = s;
    keep(sma(keep(closes), 20));
    keep(sma(keep(closes), 200));
    keep(ema(keep(closes), 20));
    keep(ema(keep(closes), 200));
    keep(wma(keep(closes), 20)).unwrap();
    keep(hma(keep(closes), 20)).unwrap();
    keep(dema(keep(closes), 20)).unwrap();
    keep(tema(keep(closes), 20)).unwrap();
    keep(alma(keep(closes), 9, 0.85, 6.0)).unwrap();
    keep(mcginley_dynamic(keep(closes), 20)).unwrap();
    keep(vwma(keep(closes), keep(volumes), 20)).unwrap();
}

#[bench(group = "indicators", setup = series_1000, covers = "finance_query::indicators::rsi")]
fn ind_momentum(s: &Series) {
    let (_, highs, lows, closes, _) = s;
    keep(rsi(keep(closes), 14)).unwrap();
    keep(stochastic(keep(highs), keep(lows), keep(closes), 14, 1, 3)).unwrap();
    keep(stochastic_rsi(keep(closes), 14, 14, 3, 3)).unwrap();
    keep(cci(keep(highs), keep(lows), keep(closes), 20)).unwrap();
    keep(macd(keep(closes), 12, 26, 9)).unwrap();
    keep(williams_r(keep(highs), keep(lows), keep(closes), 14)).unwrap();
    keep(roc(keep(closes), 12)).unwrap();
    keep(momentum(keep(closes), 10)).unwrap();
    keep(cmo(keep(closes), 14)).unwrap();
    keep(awesome_oscillator(keep(highs), keep(lows), 5, 34)).unwrap();
    keep(coppock_curve(keep(closes), 14, 11, 10)).unwrap();
}

#[bench(group = "indicators", setup = series_1000, covers = "finance_query::indicators::adx")]
fn ind_trend(s: &Series) {
    let (_, highs, lows, closes, _) = s;
    keep(adx(keep(highs), keep(lows), keep(closes), 14)).unwrap();
    keep(aroon(keep(highs), keep(lows), 25)).unwrap();
    keep(supertrend(keep(highs), keep(lows), keep(closes), 10, 3.0)).unwrap();
    keep(ichimoku(
        keep(highs),
        keep(lows),
        keep(closes),
        9,
        26,
        26,
        26,
    ))
    .unwrap();
    keep(parabolic_sar(
        keep(highs),
        keep(lows),
        keep(closes),
        0.02,
        0.2,
    ))
    .unwrap();
    keep(bull_bear_power(keep(highs), keep(lows), keep(closes), 13)).unwrap();
    keep(elder_ray(keep(highs), keep(lows), keep(closes), 13)).unwrap();
}

#[bench(
    group = "indicators",
    setup = series_1000,
    covers = "finance_query::indicators::bollinger_bands"
)]
fn ind_volatility(s: &Series) {
    let (_, highs, lows, closes, _) = s;
    keep(bollinger_bands(keep(closes), 20, 2.0)).unwrap();
    keep(keltner_channels(
        keep(highs),
        keep(lows),
        keep(closes),
        20,
        10,
        2.0,
    ))
    .unwrap();
    keep(donchian_channels(keep(highs), keep(lows), 20)).unwrap();
    keep(atr(keep(highs), keep(lows), keep(closes), 14)).unwrap();
    keep(true_range(keep(highs), keep(lows), keep(closes))).unwrap();
    keep(choppiness_index(keep(highs), keep(lows), keep(closes), 14)).unwrap();
}

#[bench(group = "indicators", setup = series_1000, covers = "finance_query::indicators::obv")]
fn ind_volume(s: &Series) {
    let (opens, highs, lows, closes, volumes) = s;
    keep(obv(keep(closes), keep(volumes))).unwrap();
    keep(mfi(
        keep(highs),
        keep(lows),
        keep(closes),
        keep(volumes),
        14,
    ))
    .unwrap();
    keep(cmf(
        keep(highs),
        keep(lows),
        keep(closes),
        keep(volumes),
        20,
    ))
    .unwrap();
    keep(chaikin_oscillator(
        keep(highs),
        keep(lows),
        keep(closes),
        keep(volumes),
    ))
    .unwrap();
    keep(accumulation_distribution(
        keep(highs),
        keep(lows),
        keep(closes),
        keep(volumes),
    ))
    .unwrap();
    keep(vwap(keep(highs), keep(lows), keep(closes), keep(volumes))).unwrap();
    keep(balance_of_power(
        keep(opens),
        keep(highs),
        keep(lows),
        keep(closes),
        None,
    ))
    .unwrap();
}

#[bench(
    group = "indicators",
    setup_sized = synthetic_candles,
    complexity = "n",
    covers = "finance_query::indicators::patterns"
)]
fn ind_patterns(candles: &[Candle]) {
    keep(patterns(keep(candles)));
}

// ── Individual indicator claims ──────────────────────────────────────────────
//
// Every function above already runs inside its family's group bench, so
// regressions in any of them already fail the gate — these give each one its
// own measured item, so a specific indicator can carry its own claim/bind
// chip in docs/library/indicators.md instead of only being an anonymous
// contributor to its family's aggregate cost.

// Moving averages
#[bench(group = "indicators", setup = series_1000, covers = "finance_query::indicators::wma")]
fn ind_wma(s: &Series) {
    let (_, _, _, closes, _) = s;
    keep(wma(keep(closes), 20)).unwrap();
}

#[bench(group = "indicators", setup = series_1000, covers = "finance_query::indicators::hma")]
fn ind_hma(s: &Series) {
    let (_, _, _, closes, _) = s;
    keep(hma(keep(closes), 20)).unwrap();
}

#[bench(group = "indicators", setup = series_1000, covers = "finance_query::indicators::dema")]
fn ind_dema(s: &Series) {
    let (_, _, _, closes, _) = s;
    keep(dema(keep(closes), 20)).unwrap();
}

#[bench(group = "indicators", setup = series_1000, covers = "finance_query::indicators::tema")]
fn ind_tema(s: &Series) {
    let (_, _, _, closes, _) = s;
    keep(tema(keep(closes), 20)).unwrap();
}

#[bench(group = "indicators", setup = series_1000, covers = "finance_query::indicators::alma")]
fn ind_alma(s: &Series) {
    let (_, _, _, closes, _) = s;
    keep(alma(keep(closes), 9, 0.85, 6.0)).unwrap();
}

#[bench(
    group = "indicators",
    setup = series_1000,
    covers = "finance_query::indicators::mcginley_dynamic"
)]
fn ind_mcginley_dynamic(s: &Series) {
    let (_, _, _, closes, _) = s;
    keep(mcginley_dynamic(keep(closes), 20)).unwrap();
}

#[bench(group = "indicators", setup = series_1000, covers = "finance_query::indicators::vwma")]
fn ind_vwma(s: &Series) {
    let (_, _, _, closes, volumes) = s;
    keep(vwma(keep(closes), keep(volumes), 20)).unwrap();
}

// Momentum
#[bench(
    group = "indicators",
    setup = series_1000,
    covers = "finance_query::indicators::stochastic"
)]
fn ind_stochastic(s: &Series) {
    let (_, highs, lows, closes, _) = s;
    keep(stochastic(keep(highs), keep(lows), keep(closes), 14, 1, 3)).unwrap();
}

#[bench(
    group = "indicators",
    setup = series_1000,
    covers = "finance_query::indicators::stochastic_rsi"
)]
fn ind_stochastic_rsi(s: &Series) {
    let (_, _, _, closes, _) = s;
    keep(stochastic_rsi(keep(closes), 14, 14, 3, 3)).unwrap();
}

#[bench(group = "indicators", setup = series_1000, covers = "finance_query::indicators::cci")]
fn ind_cci(s: &Series) {
    let (_, highs, lows, closes, _) = s;
    keep(cci(keep(highs), keep(lows), keep(closes), 20)).unwrap();
}

#[bench(group = "indicators", setup = series_1000, covers = "finance_query::indicators::macd")]
fn ind_macd(s: &Series) {
    let (_, _, _, closes, _) = s;
    keep(macd(keep(closes), 12, 26, 9)).unwrap();
}

#[bench(
    group = "indicators",
    setup = series_1000,
    covers = "finance_query::indicators::williams_r"
)]
fn ind_williams_r(s: &Series) {
    let (_, highs, lows, closes, _) = s;
    keep(williams_r(keep(highs), keep(lows), keep(closes), 14)).unwrap();
}

#[bench(group = "indicators", setup = series_1000, covers = "finance_query::indicators::roc")]
fn ind_roc(s: &Series) {
    let (_, _, _, closes, _) = s;
    keep(roc(keep(closes), 12)).unwrap();
}

#[bench(
    group = "indicators",
    setup = series_1000,
    covers = "finance_query::indicators::momentum"
)]
fn ind_momentum_single(s: &Series) {
    let (_, _, _, closes, _) = s;
    keep(momentum(keep(closes), 10)).unwrap();
}

#[bench(group = "indicators", setup = series_1000, covers = "finance_query::indicators::cmo")]
fn ind_cmo(s: &Series) {
    let (_, _, _, closes, _) = s;
    keep(cmo(keep(closes), 14)).unwrap();
}

#[bench(
    group = "indicators",
    setup = series_1000,
    covers = "finance_query::indicators::awesome_oscillator"
)]
fn ind_awesome_oscillator(s: &Series) {
    let (_, highs, lows, _, _) = s;
    keep(awesome_oscillator(keep(highs), keep(lows), 5, 34)).unwrap();
}

#[bench(
    group = "indicators",
    setup = series_1000,
    covers = "finance_query::indicators::coppock_curve"
)]
fn ind_coppock_curve(s: &Series) {
    let (_, _, _, closes, _) = s;
    keep(coppock_curve(keep(closes), 14, 11, 10)).unwrap();
}

// Trend
#[bench(group = "indicators", setup = series_1000, covers = "finance_query::indicators::aroon")]
fn ind_aroon(s: &Series) {
    let (_, highs, lows, _, _) = s;
    keep(aroon(keep(highs), keep(lows), 25)).unwrap();
}

#[bench(
    group = "indicators",
    setup = series_1000,
    covers = "finance_query::indicators::supertrend"
)]
fn ind_supertrend(s: &Series) {
    let (_, highs, lows, closes, _) = s;
    keep(supertrend(keep(highs), keep(lows), keep(closes), 10, 3.0)).unwrap();
}

#[bench(
    group = "indicators",
    setup = series_1000,
    covers = "finance_query::indicators::ichimoku"
)]
fn ind_ichimoku(s: &Series) {
    let (_, highs, lows, closes, _) = s;
    keep(ichimoku(
        keep(highs),
        keep(lows),
        keep(closes),
        9,
        26,
        26,
        26,
    ))
    .unwrap();
}

#[bench(
    group = "indicators",
    setup = series_1000,
    covers = "finance_query::indicators::parabolic_sar"
)]
fn ind_parabolic_sar(s: &Series) {
    let (_, highs, lows, closes, _) = s;
    keep(parabolic_sar(
        keep(highs),
        keep(lows),
        keep(closes),
        0.02,
        0.2,
    ))
    .unwrap();
}

#[bench(
    group = "indicators",
    setup = series_1000,
    covers = "finance_query::indicators::bull_bear_power"
)]
fn ind_bull_bear_power(s: &Series) {
    let (_, highs, lows, closes, _) = s;
    keep(bull_bear_power(keep(highs), keep(lows), keep(closes), 13)).unwrap();
}

#[bench(
    group = "indicators",
    setup = series_1000,
    covers = "finance_query::indicators::elder_ray"
)]
fn ind_elder_ray(s: &Series) {
    let (_, highs, lows, closes, _) = s;
    keep(elder_ray(keep(highs), keep(lows), keep(closes), 13)).unwrap();
}

// Volatility
#[bench(
    group = "indicators",
    setup = series_1000,
    covers = "finance_query::indicators::keltner_channels"
)]
fn ind_keltner_channels(s: &Series) {
    let (_, highs, lows, closes, _) = s;
    keep(keltner_channels(
        keep(highs),
        keep(lows),
        keep(closes),
        20,
        10,
        2.0,
    ))
    .unwrap();
}

#[bench(
    group = "indicators",
    setup = series_1000,
    covers = "finance_query::indicators::donchian_channels"
)]
fn ind_donchian_channels(s: &Series) {
    let (_, highs, lows, _, _) = s;
    keep(donchian_channels(keep(highs), keep(lows), 20)).unwrap();
}

#[bench(group = "indicators", setup = series_1000, covers = "finance_query::indicators::atr")]
fn ind_atr(s: &Series) {
    let (_, highs, lows, closes, _) = s;
    keep(atr(keep(highs), keep(lows), keep(closes), 14)).unwrap();
}

#[bench(
    group = "indicators",
    setup = series_1000,
    covers = "finance_query::indicators::true_range"
)]
fn ind_true_range(s: &Series) {
    let (_, highs, lows, closes, _) = s;
    keep(true_range(keep(highs), keep(lows), keep(closes))).unwrap();
}

#[bench(
    group = "indicators",
    setup = series_1000,
    covers = "finance_query::indicators::choppiness_index"
)]
fn ind_choppiness_index(s: &Series) {
    let (_, highs, lows, closes, _) = s;
    keep(choppiness_index(keep(highs), keep(lows), keep(closes), 14)).unwrap();
}

// Volume
#[bench(group = "indicators", setup = series_1000, covers = "finance_query::indicators::mfi")]
fn ind_mfi(s: &Series) {
    let (_, highs, lows, closes, volumes) = s;
    keep(mfi(
        keep(highs),
        keep(lows),
        keep(closes),
        keep(volumes),
        14,
    ))
    .unwrap();
}

#[bench(group = "indicators", setup = series_1000, covers = "finance_query::indicators::cmf")]
fn ind_cmf(s: &Series) {
    let (_, highs, lows, closes, volumes) = s;
    keep(cmf(
        keep(highs),
        keep(lows),
        keep(closes),
        keep(volumes),
        20,
    ))
    .unwrap();
}

#[bench(
    group = "indicators",
    setup = series_1000,
    covers = "finance_query::indicators::chaikin_oscillator"
)]
fn ind_chaikin_oscillator(s: &Series) {
    let (_, highs, lows, closes, volumes) = s;
    keep(chaikin_oscillator(
        keep(highs),
        keep(lows),
        keep(closes),
        keep(volumes),
    ))
    .unwrap();
}

#[bench(
    group = "indicators",
    setup = series_1000,
    covers = "finance_query::indicators::accumulation_distribution"
)]
fn ind_accumulation_distribution(s: &Series) {
    let (_, highs, lows, closes, volumes) = s;
    keep(accumulation_distribution(
        keep(highs),
        keep(lows),
        keep(closes),
        keep(volumes),
    ))
    .unwrap();
}

#[bench(group = "indicators", setup = series_1000, covers = "finance_query::indicators::vwap")]
fn ind_vwap(s: &Series) {
    let (_, highs, lows, closes, volumes) = s;
    keep(vwap(keep(highs), keep(lows), keep(closes), keep(volumes))).unwrap();
}

#[bench(
    group = "indicators",
    setup = series_1000,
    covers = "finance_query::indicators::balance_of_power"
)]
fn ind_balance_of_power(s: &Series) {
    let (opens, highs, lows, closes, _) = s;
    keep(balance_of_power(
        keep(opens),
        keep(highs),
        keep(lows),
        keep(closes),
        None,
    ))
    .unwrap();
}

// Pivot points, Heikin-Ashi, ZigZag, Fibonacci retracement
#[fixture]
fn candles_1000() -> Vec<Candle> {
    synthetic_candles(1000)
}

#[bench(
    group = "indicators",
    setup = series_1000,
    covers = "finance_query::indicators::pivot_points"
)]
fn ind_pivot_points(s: &Series) {
    let (_, highs, lows, closes, _) = s;
    keep(pivot_points(keep(highs), keep(lows), keep(closes))).unwrap();
}

#[bench(
    group = "indicators",
    setup = series_1000,
    covers = "finance_query::indicators::fibonacci_pivot_points"
)]
fn ind_fibonacci_pivot_points(s: &Series) {
    let (_, highs, lows, closes, _) = s;
    keep(fibonacci_pivot_points(
        keep(highs),
        keep(lows),
        keep(closes),
    ))
    .unwrap();
}

#[bench(
    group = "indicators",
    setup = candles_1000,
    covers = "finance_query::indicators::heikin_ashi"
)]
fn ind_heikin_ashi(candles: &[Candle]) {
    keep(heikin_ashi(keep(candles))).unwrap();
}

#[bench(group = "indicators", setup = series_1000, covers = "finance_query::indicators::zigzag")]
fn ind_zigzag(s: &Series) {
    let (_, highs, lows, _, _) = s;
    keep(zigzag(keep(highs), keep(lows), 5.0)).unwrap();
}

#[bench(
    group = "indicators",
    setup = series_1000,
    covers = "finance_query::indicators::fibonacci_retracement"
)]
fn ind_fibonacci_retracement(s: &Series) {
    let (_, highs, lows, _, _) = s;
    keep(fibonacci_retracement(keep(highs), keep(lows), 50)).unwrap();
}

// ── Backtesting engine ───────────────────────────────────────────────────────

#[fixture]
fn backtest_inputs() -> (BacktestConfig, Vec<Candle>) {
    let config = BacktestConfig::builder()
        .initial_capital(10_000.0)
        .commission_pct(0.001)
        .build()
        .unwrap();
    (config, synthetic_candles(1000))
}

/// Sized variant of `backtest_inputs` — candle count scales with `n`, config fixed.
fn backtest_inputs_sized(n: usize) -> (BacktestConfig, Vec<Candle>) {
    let config = BacktestConfig::builder()
        .initial_capital(10_000.0)
        .commission_pct(0.001)
        .build()
        .unwrap();
    (config, synthetic_candles(n))
}

#[bench(
    group = "backtesting",
    setup_sized = backtest_inputs_sized,
    complexity = "n",
    covers = "finance_query::backtesting::BacktestEngine::run"
)]
fn bt_sma_crossover(input: &(BacktestConfig, Vec<Candle>)) {
    let (config, candles) = input;
    // Engine construction consumes the config; the clone is trivial next to a
    // 1000-candle strategy run.
    let engine = BacktestEngine::new(config.clone());
    let strategy = SmaCrossover::new(10, 20);
    keep(engine.run(keep("BENCH"), keep(candles), strategy)).unwrap();
}

/// Multi-indicator strategy via the `refs` DSL — 5 distinct indicators
/// triggers the engine's parallel indicator-computation path (threshold 4).
#[bench(
    group = "backtesting",
    setup = backtest_inputs,
    covers = "finance_query::backtesting::StrategyBuilder"
)]
fn bt_strategy_builder(input: &(BacktestConfig, Vec<Candle>)) {
    let (config, candles) = input;
    let m = ref_macd(12, 26, 9);
    let strategy = StrategyBuilder::new("multi_indicator")
        .entry(
            ref_rsi(14)
                .below(50.0)
                .and(ref_sma(50).above_ref(ref_sma(200)))
                .and(ref_ema(20).above_ref(ref_sma(50)))
                .and(ref_atr(14).above(1.0))
                .and(m.line().above(0.0)),
        )
        .exit(ref_rsi(14).above(70.0))
        .build();
    let engine = BacktestEngine::new(config.clone());
    keep(engine.run(keep("BENCH"), keep(candles), strategy)).unwrap();
}

#[bench(
    group = "backtesting",
    setup = backtest_inputs,
    covers = "finance_query::backtesting::GridSearch"
)]
fn bt_grid_search(input: &(BacktestConfig, Vec<Candle>)) {
    let (config, candles) = input;
    // 5x5 = 25 evaluations — the criterion bench's larger sweep.
    let search = GridSearch::new()
        .param("fast", ParamRange::int_range(5, 25, 5))
        .param("slow", ParamRange::int_range(20, 60, 10))
        .optimize_for(OptimizeMetric::SharpeRatio);
    keep(
        search
            .run(keep("BENCH"), keep(candles), config, |params| {
                SmaCrossover::new(
                    params["fast"].as_int() as usize,
                    params["slow"].as_int() as usize,
                )
            })
            .unwrap(),
    );
}

#[bench(
    group = "backtesting",
    setup = backtest_inputs,
    covers = "finance_query::backtesting::BayesianSearch"
)]
fn bt_bayesian_search(input: &(BacktestConfig, Vec<Candle>)) {
    let (config, candles) = input;
    // 2-param search, 20 evaluations (10 LHS + 10 sequential), seeded for
    // determinism.
    let search = BayesianSearch::new()
        .param("fast", ParamRange::int_bounds(5, 25))
        .param("slow", ParamRange::int_bounds(20, 60))
        .optimize_for(OptimizeMetric::SharpeRatio)
        .max_evaluations(20)
        .seed(42);
    keep(
        search
            .run(keep("BENCH"), keep(candles), config, |params| {
                SmaCrossover::new(
                    params["fast"].as_int() as usize,
                    params["slow"].as_int() as usize,
                )
            })
            .unwrap(),
    );
}

#[fixture]
fn leveraged_backtest_inputs() -> (BacktestConfig, Vec<Candle>) {
    let config = BacktestConfig::builder()
        .initial_capital(10_000.0)
        .commission_pct(0.001)
        .max_leverage(2.0)
        .maintenance_margin_pct(0.25)
        .margin_interest_rate(0.05)
        .build()
        .unwrap();
    (config, synthetic_candles(1000))
}

#[bench(
    group = "backtesting",
    setup = leveraged_backtest_inputs,
    covers = "finance_query::backtesting::BacktestConfigBuilder::max_leverage"
)]
fn bt_leveraged_margin(input: &(BacktestConfig, Vec<Candle>)) {
    let (config, candles) = input;
    let engine = BacktestEngine::new(config.clone());
    let strategy = SmaCrossover::new(10, 20);
    keep(engine.run(keep("BENCH"), keep(candles), strategy)).unwrap();
}

#[fixture]
fn atr_sizing_backtest_inputs() -> (BacktestConfig, Vec<Candle>) {
    let config = BacktestConfig::builder()
        .initial_capital(10_000.0)
        .commission_pct(0.001)
        .position_sizing(PositionSizing::Atr {
            risk_pct: 0.02,
            atr_period: 14,
            atr_multiple: 2.0,
        })
        .build()
        .unwrap();
    (config, synthetic_candles(1000))
}

#[bench(
    group = "backtesting",
    setup = atr_sizing_backtest_inputs,
    covers = "finance_query::backtesting::BacktestConfig::calculate_position_size_with_context"
)]
fn bt_atr_sizing(input: &(BacktestConfig, Vec<Candle>)) {
    let (config, candles) = input;
    let engine = BacktestEngine::new(config.clone());
    let strategy = SmaCrossover::new(10, 20);
    keep(engine.run(keep("BENCH"), keep(candles), strategy)).unwrap();
}

#[fixture]
fn kelly_sizing_backtest_inputs() -> (BacktestConfig, Vec<Candle>) {
    let config = BacktestConfig::builder()
        .initial_capital(10_000.0)
        .commission_pct(0.001)
        .position_sizing(PositionSizing::FractionalKelly {
            kelly_fraction: 0.5,
            lookback_trades: 20,
        })
        .build()
        .unwrap();
    (config, synthetic_candles(1000))
}

#[bench(
    group = "backtesting",
    setup = kelly_sizing_backtest_inputs,
    covers = "finance_query::backtesting::BacktestConfigBuilder::position_sizing"
)]
fn bt_fractional_kelly_sizing(input: &(BacktestConfig, Vec<Candle>)) {
    let (config, candles) = input;
    let engine = BacktestEngine::new(config.clone());
    let strategy = SmaCrossover::new(10, 20);
    keep(engine.run(keep("BENCH"), keep(candles), strategy)).unwrap();
}

#[bench(
    group = "backtesting",
    setup = backtest_inputs,
    covers = "finance_query::backtesting::GridSearch::run_pareto"
)]
fn bt_grid_pareto(input: &(BacktestConfig, Vec<Candle>)) {
    let (config, candles) = input;
    let search = GridSearch::new()
        .param("fast", ParamRange::int_range(5, 25, 5))
        .param("slow", ParamRange::int_range(20, 60, 10));
    keep(
        search
            .run_pareto(
                keep("BENCH"),
                keep(candles),
                config,
                &[OptimizeMetric::SharpeRatio, OptimizeMetric::MinDrawdown],
                |params| {
                    SmaCrossover::new(
                        params["fast"].as_int() as usize,
                        params["slow"].as_int() as usize,
                    )
                },
            )
            .unwrap(),
    );
}

#[fixture]
fn backtest_result() -> finance_query::backtesting::BacktestResult {
    let (config, candles) = backtest_inputs();
    let engine = BacktestEngine::new(config);
    engine
        .run("BENCH", &candles, SmaCrossover::new(10, 20))
        .unwrap()
}

#[bench(
    group = "backtesting",
    setup = backtest_result,
    covers = "finance_query::backtesting::MonteCarloConfig"
)]
fn bt_monte_carlo(result: &finance_query::backtesting::BacktestResult) {
    let mc_config = MonteCarloConfig::new().num_simulations(500).seed(42);
    keep(mc_config.run(keep(result)));
}

// ── Strategy DSL: refs and conditions ────────────────────────────────────────
//
// `refs::*` and `condition::*` builders can't be exercised directly from an
// external bench crate — `StrategyContext` (what `IndicatorRef::value` and
// `Condition::evaluate` both take) is `#[non_exhaustive]` with no public
// constructor, so the only way to reach them from outside the crate is
// through the same public pipeline real strategies use:
// `StrategyBuilder` + `BacktestEngine::run`. Each bench below runs a minimal
// one-condition strategy so that specific builder gets its own claim.

fn run_ref_strategy<E: Condition, X: Condition>(
    input: &(BacktestConfig, Vec<Candle>),
    entry: E,
    exit: X,
) {
    let (config, candles) = input;
    let strategy = StrategyBuilder::new("bench")
        .entry(entry)
        .exit(exit)
        .build();
    let engine = BacktestEngine::new(config.clone());
    keep(engine.run(keep("BENCH"), keep(candles), strategy)).unwrap();
}

macro_rules! ref_bench {
    ($fn_name:ident, $covers:literal, $entry:expr) => {
        #[bench(group = "backtesting_refs", setup = backtest_inputs, covers = $covers)]
        fn $fn_name(input: &(BacktestConfig, Vec<Candle>)) {
            run_ref_strategy(input, $entry, always_true());
        }
    };
    ($fn_name:ident, $covers:literal, $entry:expr, tolerance = $tolerance:literal) => {
        #[bench(group = "backtesting_refs", setup = backtest_inputs, covers = $covers, tolerance = $tolerance)]
        fn $fn_name(input: &(BacktestConfig, Vec<Candle>)) {
            run_ref_strategy(input, $entry, always_true());
        }
    };
}

ref_bench!(
    ref_bench_accumulation_distribution,
    "finance_query::backtesting::refs::accumulation_distribution",
    ref_accumulation_distribution().above(0.0)
);
ref_bench!(
    ref_bench_adx,
    "finance_query::backtesting::refs::adx",
    ref_adx(14).above(0.0)
);
ref_bench!(
    ref_bench_alma,
    "finance_query::backtesting::refs::alma",
    ref_alma(9, 0.85, 6.0).above(0.0)
);
// Small enough that the engine's per-bar margin/sizing cost straddles
// the default threshold run-to-run.
#[bench(
    group = "backtesting_refs",
    setup = backtest_inputs,
    covers = "finance_query::backtesting::refs::aroon",
    tolerance = "8%"
)]
fn ref_bench_aroon(input: &(BacktestConfig, Vec<Candle>)) {
    run_ref_strategy(input, ref_aroon(25).up().above(0.0), always_true());
}
ref_bench!(
    ref_bench_atr,
    "finance_query::backtesting::refs::atr",
    ref_atr(14).above(0.0)
);
ref_bench!(
    ref_bench_awesome_oscillator,
    "finance_query::backtesting::refs::awesome_oscillator",
    ref_awesome_oscillator(5, 34).above(0.0)
);
ref_bench!(
    ref_bench_balance_of_power,
    "finance_query::backtesting::refs::balance_of_power",
    ref_balance_of_power(None).above(0.0)
);
ref_bench!(
    ref_bench_bear_power,
    "finance_query::backtesting::refs::bear_power",
    ref_bear_power(13).above(0.0)
);
ref_bench!(
    ref_bench_bollinger,
    "finance_query::backtesting::refs::bollinger",
    ref_bollinger(20, 2.0).middle().above(0.0)
);
ref_bench!(
    ref_bench_bull_power,
    "finance_query::backtesting::refs::bull_power",
    ref_bull_power(13).above(0.0)
);
ref_bench!(
    ref_bench_cci,
    "finance_query::backtesting::refs::cci",
    ref_cci(20).above(0.0)
);
ref_bench!(
    ref_bench_chaikin_oscillator,
    "finance_query::backtesting::refs::chaikin_oscillator",
    ref_chaikin_oscillator().above(0.0)
);
ref_bench!(
    ref_bench_choppiness_index,
    "finance_query::backtesting::refs::choppiness_index",
    ref_choppiness_index(14).above(0.0)
);
ref_bench!(
    ref_bench_cmf,
    "finance_query::backtesting::refs::cmf",
    ref_cmf(20).above(0.0)
);
ref_bench!(
    ref_bench_cmo,
    "finance_query::backtesting::refs::cmo",
    ref_cmo(14).above(0.0)
);
ref_bench!(
    ref_bench_coppock_curve,
    "finance_query::backtesting::refs::coppock_curve",
    ref_coppock_curve(14, 11, 10).above(0.0)
);
ref_bench!(
    ref_bench_dema,
    "finance_query::backtesting::refs::dema",
    ref_dema(20).above(0.0)
);
ref_bench!(
    ref_bench_donchian,
    "finance_query::backtesting::refs::donchian",
    ref_donchian(20).middle().above(0.0)
);
ref_bench!(
    ref_bench_elder_bear_power,
    "finance_query::backtesting::refs::elder_bear_power",
    ref_elder_bear_power(13).above(0.0)
);
ref_bench!(
    ref_bench_elder_bull_power,
    "finance_query::backtesting::refs::elder_bull_power",
    ref_elder_bull_power(13).above(0.0)
);
ref_bench!(
    ref_bench_ema,
    "finance_query::backtesting::refs::ema",
    ref_ema(20).above(0.0)
);
ref_bench!(
    ref_bench_hma,
    "finance_query::backtesting::refs::hma",
    ref_hma(20).above(0.0)
);
ref_bench!(
    ref_bench_ichimoku,
    "finance_query::backtesting::refs::ichimoku",
    ref_ichimoku().conversion_line().above(0.0)
);
ref_bench!(
    ref_bench_ichimoku_custom,
    "finance_query::backtesting::refs::ichimoku_custom",
    ref_ichimoku_custom(9, 26, 52, 26)
        .conversion_line()
        .above(0.0)
);
// The engine's accepted per-bar cost leaves this bench and the other
// expanded ones below straddling their default limits run-to-run.
#[bench(
    group = "backtesting_refs",
    setup = backtest_inputs,
    covers = "finance_query::backtesting::refs::keltner",
    tolerance = "8%"
)]
fn ref_bench_keltner(input: &(BacktestConfig, Vec<Candle>)) {
    run_ref_strategy(
        input,
        ref_keltner(20, 2.0, 10).middle().above(0.0),
        always_true(),
    );
}
#[bench(
    group = "backtesting_refs",
    setup = backtest_inputs,
    covers = "finance_query::backtesting::refs::macd",
    tolerance = "8%"
)]
fn ref_bench_macd(input: &(BacktestConfig, Vec<Candle>)) {
    run_ref_strategy(input, ref_macd(12, 26, 9).line().above(0.0), always_true());
}
ref_bench!(
    ref_bench_mcginley,
    "finance_query::backtesting::refs::mcginley",
    ref_mcginley(20).above(0.0)
);
ref_bench!(
    ref_bench_mfi,
    "finance_query::backtesting::refs::mfi",
    ref_mfi(14).above(0.0)
);
ref_bench!(
    ref_bench_momentum,
    "finance_query::backtesting::refs::momentum",
    ref_momentum(10).above(0.0)
);
ref_bench!(
    ref_bench_obv,
    "finance_query::backtesting::refs::obv",
    ref_obv().above(0.0)
);
ref_bench!(
    ref_bench_parabolic_sar,
    "finance_query::backtesting::refs::parabolic_sar",
    ref_parabolic_sar(0.02, 0.2).above(0.0)
);
ref_bench!(
    ref_bench_roc,
    "finance_query::backtesting::refs::roc",
    ref_roc(12).above(0.0)
);
ref_bench!(
    ref_bench_rsi,
    "finance_query::backtesting::refs::rsi",
    ref_rsi(14).above(0.0)
);
ref_bench!(
    ref_bench_sma,
    "finance_query::backtesting::refs::sma",
    ref_sma(20).above(0.0)
);
ref_bench!(
    ref_bench_stochastic,
    "finance_query::backtesting::refs::stochastic",
    ref_stochastic(14, 1, 3).k().above(0.0)
);
#[bench(
    group = "backtesting_refs",
    setup = backtest_inputs,
    covers = "finance_query::backtesting::refs::stochastic_rsi",
    tolerance = "8%"
)]
fn ref_bench_stochastic_rsi(input: &(BacktestConfig, Vec<Candle>)) {
    run_ref_strategy(
        input,
        ref_stochastic_rsi(14, 14, 3, 3).k().above(0.0),
        always_true(),
    );
}
// Small enough that a codegen change alone crosses +5% with source unchanged.
ref_bench!(
    ref_bench_supertrend,
    "finance_query::backtesting::refs::supertrend",
    ref_supertrend(10, 3.0).value().above(0.0),
    tolerance = "15%"
);
ref_bench!(
    ref_bench_tema,
    "finance_query::backtesting::refs::tema",
    ref_tema(20).above(0.0)
);
ref_bench!(
    ref_bench_true_range,
    "finance_query::backtesting::refs::true_range",
    ref_true_range().above(0.0)
);
ref_bench!(
    ref_bench_vwap,
    "finance_query::backtesting::refs::vwap",
    ref_vwap().above(0.0)
);
ref_bench!(
    ref_bench_vwma,
    "finance_query::backtesting::refs::vwma",
    ref_vwma(20).above(0.0)
);
#[bench(
    group = "backtesting_refs",
    setup = backtest_inputs,
    covers = "finance_query::backtesting::refs::williams_r",
    tolerance = "16%"
)]
fn ref_bench_williams_r(input: &(BacktestConfig, Vec<Candle>)) {
    run_ref_strategy(input, ref_williams_r(14).above(0.0), always_true());
}
ref_bench!(
    ref_bench_wma,
    "finance_query::backtesting::refs::wma",
    ref_wma(20).above(0.0)
);

// Price/candle-field refs — all zero-arg except `relative_volume`.
// The two tiniest ref benches: the engine's accepted per-bar cost sits
// near +14% of their bodies and pulls walltime past its limit with it.
#[bench(
    group = "backtesting_refs",
    setup = backtest_inputs,
    covers = "finance_query::backtesting::refs::candle_body",
    tolerance = "16%"
)]
fn ref_bench_candle_body(input: &(BacktestConfig, Vec<Candle>)) {
    run_ref_strategy(input, candle_body().above(0.0), always_true());
}
ref_bench!(
    ref_bench_candle_range,
    "finance_query::backtesting::refs::candle_range",
    candle_range().above(0.0)
);
ref_bench!(
    ref_bench_close,
    "finance_query::backtesting::refs::close",
    close().above(0.0)
);
ref_bench!(
    ref_bench_gap_pct,
    "finance_query::backtesting::refs::gap_pct",
    gap_pct().above(0.0)
);
ref_bench!(
    ref_bench_high,
    "finance_query::backtesting::refs::high",
    high().above(0.0)
);
ref_bench!(
    ref_bench_is_bearish,
    "finance_query::backtesting::refs::is_bearish",
    is_bearish().above(0.0)
);
#[bench(
    group = "backtesting_refs",
    setup = backtest_inputs,
    covers = "finance_query::backtesting::refs::is_bullish",
    tolerance = "16%"
)]
fn ref_bench_is_bullish(input: &(BacktestConfig, Vec<Candle>)) {
    run_ref_strategy(input, is_bullish().above(0.0), always_true());
}
ref_bench!(
    ref_bench_low,
    "finance_query::backtesting::refs::low",
    low().above(0.0)
);
ref_bench!(
    ref_bench_median_price,
    "finance_query::backtesting::refs::median_price",
    median_price().above(0.0)
);
ref_bench!(
    ref_bench_open,
    "finance_query::backtesting::refs::open",
    open().above(0.0)
);
ref_bench!(
    ref_bench_price,
    "finance_query::backtesting::refs::price",
    price().above(0.0)
);
ref_bench!(
    ref_bench_price_change_pct,
    "finance_query::backtesting::refs::price_change_pct",
    price_change_pct().above(0.0)
);
ref_bench!(
    ref_bench_relative_volume,
    "finance_query::backtesting::refs::relative_volume",
    relative_volume(20).above(0.0)
);
ref_bench!(
    ref_bench_typical_price,
    "finance_query::backtesting::refs::typical_price",
    typical_price().above(0.0)
);
ref_bench!(
    ref_bench_volume,
    "finance_query::backtesting::refs::volume",
    volume().above(0.0)
);

// Higher-timeframe wrappers — compose any Condition, so wrap a simple one.
ref_bench!(
    ref_bench_htf,
    "finance_query::backtesting::refs::htf",
    htf(Interval::OneWeek, price().above(0.0))
);
ref_bench!(
    ref_bench_htf_region,
    "finance_query::backtesting::refs::htf_region",
    htf_region(Interval::OneWeek, Region::UnitedStates, price().above(0.0))
);

// Condition builders (position-state predicates + thresholds) — paired with
// a trivial always-on entry so each gets a live position to evaluate against.
macro_rules! condition_bench {
    ($fn_name:ident, $covers:literal, $exit:expr) => {
        #[bench(group = "backtesting_conditions", setup = backtest_inputs, covers = $covers)]
        fn $fn_name(input: &(BacktestConfig, Vec<Candle>)) {
            run_ref_strategy(input, always_true(), $exit);
        }
    };
}

condition_bench!(
    cond_bench_has_position,
    "finance_query::backtesting::condition::has_position",
    has_position()
);
condition_bench!(
    cond_bench_held_for_bars,
    "finance_query::backtesting::condition::held_for_bars",
    held_for_bars(3)
);
condition_bench!(
    cond_bench_in_loss,
    "finance_query::backtesting::condition::in_loss",
    in_loss()
);
condition_bench!(
    cond_bench_in_profit,
    "finance_query::backtesting::condition::in_profit",
    in_profit()
);
condition_bench!(
    cond_bench_is_long,
    "finance_query::backtesting::condition::is_long",
    is_long()
);
condition_bench!(
    cond_bench_is_short,
    "finance_query::backtesting::condition::is_short",
    is_short()
);
condition_bench!(
    cond_bench_no_position,
    "finance_query::backtesting::condition::no_position",
    no_position()
);
condition_bench!(
    cond_bench_stop_loss,
    "finance_query::backtesting::condition::stop_loss",
    stop_loss(0.05)
);
condition_bench!(
    cond_bench_take_profit,
    "finance_query::backtesting::condition::take_profit",
    take_profit(0.10)
);
condition_bench!(
    cond_bench_trailing_stop,
    "finance_query::backtesting::condition::trailing_stop",
    trailing_stop(0.05)
);
condition_bench!(
    cond_bench_trailing_take_profit,
    "finance_query::backtesting::condition::trailing_take_profit",
    trailing_take_profit(0.05)
);

// As tiny as the smallest ref benches: the engine's accepted per-bar cost
// sits near +17% of its body and pulls walltime past its limit with it.
#[bench(
    group = "backtesting_conditions",
    setup = backtest_inputs,
    covers = "finance_query::backtesting::condition::always_false",
    tolerance = "20%"
)]
fn cond_bench_always_false(input: &(BacktestConfig, Vec<Candle>)) {
    run_ref_strategy(input, always_false(), always_true());
}

#[bench(
    group = "backtesting_conditions",
    setup = backtest_inputs,
    covers = "finance_query::backtesting::condition::always_true"
)]
fn cond_bench_always_true(input: &(BacktestConfig, Vec<Candle>)) {
    run_ref_strategy(input, always_true(), always_true());
}

// ── Resampling utilities (pure functions on candle data — no context) ────────

#[bench(
    group = "backtesting_resample",
    setup_sized = synthetic_candles,
    complexity = "n",
    covers = "finance_query::backtesting::resample::resample"
)]
fn bt_resample(candles: &[Candle]) {
    keep(resample(keep(candles), Interval::OneWeek, 0));
}

/// Feeds `bt_base_to_htf_index` — base candle count scales with `n`; the
/// HTF series is derived from it, so it scales proportionally too.
fn resampled_pair_sized(n: usize) -> (Vec<Candle>, Vec<Candle>) {
    let base = synthetic_candles(n);
    let htf = resample(&base, Interval::OneWeek, 0);
    (base, htf)
}

// ~20 instructions per element: one more already exceeds +5%, at any size.
#[bench(
    group = "backtesting_resample",
    setup_sized = resampled_pair_sized,
    complexity = "n",
    tolerance = "8%",
    covers = "finance_query::backtesting::resample::base_to_htf_index"
)]
fn bt_base_to_htf_index(pair: &(Vec<Candle>, Vec<Candle>)) {
    keep(base_to_htf_index(keep(&pair.0), keep(&pair.1)));
}

// ── Indicators utility ───────────────────────────────────────────────────────

#[fixture]
fn optional_series_1000() -> Vec<Option<f64>> {
    (0..1000)
        .map(|i| if i % 7 == 0 { None } else { Some(i as f64) })
        .collect()
}

#[bench(
    group = "indicators",
    setup = optional_series_1000,
    covers = "finance_query::indicators::last_value"
)]
fn ind_last_value(values: &[Option<f64>]) {
    keep(last_value(keep(values)));
}

// ── Risk metric hot paths ────────────────────────────────────────────────────

/// Clone + stable sort of the return series — the claims pin the class and
/// the two allocations (the clone and the merge-sort scratch buffer).
#[bench(
    group = "risk",
    setup_sized = synthetic_returns,
    sizes(1000, 10000, 100000),
    complexity = "n log n",
    alloc = 2,
    covers = "finance_query::risk::historical_var"
)]
fn risk_historical_var(returns: &[f64]) {
    keep(historical_var(keep(returns), 0.95));
}

#[bench(
    group = "risk",
    setup_sized = synthetic_returns,
    complexity = "n",
    alloc = 0,
    covers = "finance_query::risk::parametric_var"
)]
fn risk_parametric_var(returns: &[f64]) {
    keep(parametric_var(keep(returns), 0.95));
}

#[bench(
    group = "risk",
    setup_sized = synthetic_returns,
    complexity = "n",
    alloc = 0,
    covers = "finance_query::risk::sharpe_ratio"
)]
fn risk_sharpe(returns: &[f64]) {
    keep(sharpe_ratio(keep(returns), 0.0, 252.0));
}

#[bench(
    group = "risk",
    setup_sized = synthetic_returns,
    complexity = "n",
    alloc = 0,
    covers = "finance_query::risk::sortino_ratio"
)]
fn risk_sortino(returns: &[f64]) {
    keep(sortino_ratio(keep(returns), 0.0, 252.0));
}

#[bench(
    group = "risk",
    setup = returns_1000,
    alloc = 1,
    covers = "finance_query::risk::max_drawdown"
)]
fn risk_max_drawdown(returns: &[f64]) {
    // `DrawdownResult` is not publicly nameable; keep its fields so the full
    // computation is still measured and not optimised away.
    let dd = max_drawdown(keep(returns));
    keep((dd.max_drawdown, dd.recovery_periods));
}

#[bench(
    group = "risk",
    setup_sized = synthetic_returns_pair,
    complexity = "n",
    alloc = 0,
    covers = "finance_query::risk::beta"
)]
fn risk_beta(series: &(Vec<f64>, Vec<f64>)) {
    keep(beta(keep(&series.0), keep(&series.1)));
}

/// Pure scalar arithmetic (CAGR / max drawdown ratio) — no allocation.
#[soothfast::measured(
    group = "risk",
    alloc = 0,
    covers = "finance_query::risk::calmar_ratio"
)]
fn risk_calmar() {
    keep(calmar_ratio(keep(0.18), keep(5.0), keep(0.30)));
}

#[bench(
    group = "risk",
    setup_sized = synthetic_returns,
    covers = "finance_query::risk::historical_cvar"
)]
fn risk_historical_cvar(returns: &[f64]) {
    keep(historical_cvar(keep(returns), 0.95));
}

#[bench(
    group = "risk",
    setup_sized = synthetic_returns,
    covers = "finance_query::risk::parametric_cvar"
)]
fn risk_parametric_cvar(returns: &[f64]) {
    keep(parametric_cvar(keep(returns), 0.95));
}

#[bench(
    group = "risk",
    setup_sized = synthetic_returns,
    covers = "finance_query::risk::omega_ratio"
)]
fn risk_omega_ratio(returns: &[f64]) {
    keep(omega_ratio(keep(returns)));
}

#[bench(
    group = "risk",
    setup_sized = synthetic_returns,
    covers = "finance_query::risk::ulcer_index"
)]
fn risk_ulcer_index(returns: &[f64]) {
    keep(ulcer_index(keep(returns)));
}

#[bench(
    group = "risk",
    setup_sized = synthetic_returns,
    covers = "finance_query::risk::win_loss_stats"
)]
fn risk_win_loss_stats(returns: &[f64]) {
    keep(win_loss_stats(keep(returns)));
}

/// Derives the win/loss inputs from the series, then the Kelly fraction —
/// the same composition the iai gate measured.
#[bench(
    group = "risk",
    setup = returns_1000,
    covers = "finance_query::risk::kelly_criterion"
)]
fn risk_kelly_criterion(returns: &[f64]) {
    let (win_rate, avg_win_pct, avg_loss_pct) = win_loss_stats(keep(returns));
    keep(kelly_criterion(win_rate, avg_win_pct, avg_loss_pct));
}

#[bench(
    group = "risk",
    setup_sized = synthetic_returns_pair,
    covers = "finance_query::risk::information_ratio"
)]
fn risk_information_ratio(series: &(Vec<f64>, Vec<f64>)) {
    keep(information_ratio(keep(&series.0), keep(&series.1), 252.0));
}

#[bench(
    group = "risk",
    setup_sized = synthetic_returns_pair,
    covers = "finance_query::risk::tracking_error"
)]
fn risk_tracking_error(series: &(Vec<f64>, Vec<f64>)) {
    keep(tracking_error(keep(&series.0), keep(&series.1), 252.0));
}

// ── Streaming (PriceUpdate serde — the per-tick hot path) ─────────────────────

#[fixture]
fn make_price_update() -> PriceUpdate {
    PriceUpdate {
        id: "AAPL".to_string(),
        price: 175.5,
        time: 1_774_040_720,
        currency: "USD".to_string(),
        exchange: "NMS".to_string(),
        quote_type: QuoteType::Equity,
        market_hours: MarketHoursType::RegularMarket,
        change_percent: 1.23,
        day_volume: 52_345_678,
        day_high: 176.2,
        day_low: 174.1,
        change: 2.15,
        short_name: "Apple Inc.".to_string(),
        expire_date: 0,
        open_price: 173.8,
        previous_close: 173.35,
        strike_price: 0.0,
        underlying_symbol: String::new(),
        open_interest: 0,
        options_type: OptionType::Call,
        mini_option: 0,
        last_size: 100,
        bid: 175.48,
        bid_size: 200,
        ask: 175.52,
        ask_size: 300,
        price_hint: 2,
        vol_24hr: 0,
        vol_all_currencies: 0,
        from_currency: String::new(),
        last_market: String::new(),
        circulating_supply: 0.0,
        market_cap: 2_700_000_000_000.0,
    }
}

#[fixture]
fn price_update_json() -> String {
    serde_json::to_string(&make_price_update()).unwrap()
}

#[bench(
    group = "streaming",
    setup = make_price_update,
    covers = "finance_query::streaming::PriceUpdate"
)]
fn stream_serialize(update: &PriceUpdate) {
    keep(serde_json::to_string(keep(update)).unwrap());
}

#[bench(
    group = "streaming",
    setup = price_update_json,
    covers = "finance_query::streaming::PriceUpdate"
)]
fn stream_deserialize(json: &str) {
    keep(serde_json::from_str::<PriceUpdate>(keep(json)).unwrap());
}

// ── Provider capability dispatch (the per-fetch selection) ───────────────────

#[fixture]
fn provider_registry() -> Vec<Capability> {
    use Capability as C;
    vec![
        C::QUOTE | C::CHART | C::FUNDAMENTALS | C::CORPORATE | C::OPTIONS, // yahoo
        C::QUOTE | C::CHART | C::FUNDAMENTALS | C::FOREX | C::CRYPTO | C::COMMODITIES | C::INDICES, // fmp
        C::QUOTE | C::CHART | C::FUNDAMENTALS | C::OPTIONS | C::FOREX | C::CRYPTO | C::ECONOMIC, // alphavantage
        C::QUOTE
            | C::CHART
            | C::FUNDAMENTALS
            | C::CORPORATE
            | C::OPTIONS
            | C::CRYPTO
            | C::FOREX
            | C::FUTURES
            | C::INDICES
            | C::FILINGS
            | C::ECONOMIC, // polygon
        C::CRYPTO,   // coingecko
        C::ECONOMIC, // fred
        C::FILINGS,  // edgar
    ]
}

/// Pure bitflag filtering over a fixed registry — never allocates.
#[bench(
    group = "providers",
    setup = provider_registry,
    alloc = 0,
    covers = "finance_query::Capability"
)]
fn dispatch_select(registry: &[Capability]) {
    let wanted = keep(Capability::QUOTE);
    keep(registry.iter().filter(|c| c.contains(wanted)).count());
}

/// `&'static str` lookup — never allocates.
#[soothfast::measured(
    group = "providers",
    alloc = 0,
    covers = "finance_query::Capability::name"
)]
fn capability_name() {
    keep(keep(Capability::FUNDAMENTALS).name());
}

// ── Ticker / Tickers cache behavior (canned provider, no network) ───────────
//
// `Ticker`/`Tickers` normally fetch over HTTP, and `StrategyContext`-style
// direct construction isn't available here (their caches are genuinely
// private state, not a public-visibility issue) — but the *provider* they
// fetch through is an injectable seam. `CannedProvider` answers every quote
// and chart request immediately from an in-memory fixture, so these gate the
// real cache/dispatch logic (first access fetches, subsequent accesses are
// free) with zero network and full determinism.

struct CannedProvider;

impl finance_query::ProviderCore for CannedProvider {
    fn id(&self) -> Provider {
        Provider::Yahoo
    }
}

#[async_trait::async_trait]
impl finance_query::QuoteProvider for CannedProvider {
    async fn fetch_quote(&self, symbol: &str) -> finance_query::Result<QuoteSummaryResponse> {
        Ok(QuoteSummaryResponse {
            symbol: symbol.to_string(),
            price: serde_json::from_value(serde_json::json!({})).ok(),
            ..Default::default()
        })
    }

    async fn fetch_quotes_batch(
        &self,
        symbols: &[&str],
    ) -> finance_query::Result<Vec<(String, QuoteSummaryResponse)>> {
        Ok(symbols
            .iter()
            .map(|s| {
                (
                    s.to_string(),
                    QuoteSummaryResponse {
                        symbol: s.to_string(),
                        price: serde_json::from_value(serde_json::json!({})).ok(),
                        ..Default::default()
                    },
                )
            })
            .collect())
    }
}

#[async_trait::async_trait]
impl finance_query::ChartProvider for CannedProvider {
    async fn fetch_chart(
        &self,
        _symbol: &str,
        _interval: Interval,
        _range: TimeRange,
    ) -> finance_query::Result<Chart> {
        Ok(serde_json::from_str(CHART_JSON).unwrap())
    }
}

impl ProviderAdapter for CannedProvider {
    fn as_quote(&self) -> Option<&dyn finance_query::QuoteProvider> {
        Some(self)
    }

    fn as_chart(&self) -> Option<&dyn finance_query::ChartProvider> {
        Some(self)
    }
}

fn canned_provider_set() -> Arc<ProviderSet> {
    Arc::new(ProviderSet::new(
        vec![Arc::new(CannedProvider) as Arc<dyn ProviderAdapter>],
        None,
        Routes::new(Fetch::Sequential),
    ))
}

#[fixture]
fn canned_ticker() -> (tokio::runtime::Runtime, Ticker) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let ticker = rt
        .block_on(
            Ticker::builder("AAPL")
                .with_provider_set(canned_provider_set())
                .build(),
        )
        .unwrap();
    (rt, ticker)
}

/// First `.quote()` dispatches through the canned provider; the second hits
/// the module cache — both run every measured iteration, so this gates the
/// combined dispatch-then-cache-hit cost of the real `Ticker` cache logic.
#[bench(
    group = "ticker",
    setup = canned_ticker,
    covers = "finance_query::Ticker::quote"
)]
fn ticker_quote_then_cached(input: &(tokio::runtime::Runtime, Ticker)) {
    let (rt, ticker) = input;
    let first = rt
        .block_on(ticker.quote::<finance_query::format::Raw>())
        .unwrap();
    let second = rt
        .block_on(ticker.quote::<finance_query::format::Raw>())
        .unwrap();
    keep((first, second));
}

#[bench(
    group = "ticker",
    setup = canned_ticker,
    covers = "finance_query::Ticker::chart"
)]
fn ticker_chart_then_cached(input: &(tokio::runtime::Runtime, Ticker)) {
    let (rt, ticker) = input;
    let first = rt
        .block_on(ticker.chart(Interval::OneDay, TimeRange::OneMonth))
        .unwrap();
    let second = rt
        .block_on(ticker.chart(Interval::OneDay, TimeRange::OneMonth))
        .unwrap();
    keep((first, second));
}

#[fixture]
fn canned_tickers() -> (tokio::runtime::Runtime, Tickers) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let tickers = rt
        .block_on(
            Tickers::builder(["AAPL", "MSFT", "GOOGL"])
                .with_provider_set(canned_provider_set())
                .build(),
        )
        .unwrap();
    (rt, tickers)
}

/// Batch dispatch (one provider call for all symbols) then a cache hit.
#[bench(
    group = "tickers",
    setup = canned_tickers,
    covers = "finance_query::Tickers::quotes"
)]
fn tickers_batch_quote_then_cached(input: &(tokio::runtime::Runtime, Tickers)) {
    let (rt, tickers) = input;
    let first = rt.block_on(tickers.quotes()).unwrap();
    let second = rt.block_on(tickers.quotes()).unwrap();
    keep((first, second));
}

// ── Model (de)serialization (real Yahoo-shaped fixtures) ─────────────────────

static SEARCH_JSON: &str = include_str!("fixtures/search.json");
static NEWS_JSON: &str = include_str!("fixtures/news.json");
static CURRENCIES_JSON: &str = include_str!("fixtures/currencies.json");
static MARKET_SUMMARY_JSON: &str = include_str!("fixtures/market_summary.json");
static TRENDING_JSON: &str = include_str!("fixtures/trending.json");
static FEAR_AND_GREED_JSON: &str = include_str!("fixtures/fear_and_greed.json");
static FEAR_AND_GREED_CRYPTO_HISTORY_JSON: &str =
    include_str!("fixtures/fear_and_greed_crypto_history.json");
static HOURS_JSON: &str = include_str!("fixtures/hours.json");

#[soothfast::measured(group = "model_serde", covers = "finance_query::SearchResults")]
fn de_search() {
    keep(serde_json::from_str::<SearchResults>(keep(SEARCH_JSON)).unwrap());
}

#[soothfast::measured(group = "model_serde", covers = "finance_query::News")]
fn de_news() {
    keep(serde_json::from_str::<Vec<News>>(keep(NEWS_JSON)).unwrap());
}

#[soothfast::measured(group = "model_serde", covers = "finance_query::Currency")]
fn de_currencies() {
    keep(serde_json::from_str::<Vec<Currency>>(keep(CURRENCIES_JSON)).unwrap());
}

#[soothfast::measured(group = "model_serde", covers = "finance_query::MarketSummaryQuote")]
fn de_market_summary() {
    keep(serde_json::from_str::<Vec<MarketSummaryQuote>>(keep(MARKET_SUMMARY_JSON)).unwrap());
}

#[soothfast::measured(group = "model_serde", covers = "finance_query::TrendingQuote")]
fn de_trending() {
    keep(serde_json::from_str::<Vec<TrendingQuote>>(keep(TRENDING_JSON)).unwrap());
}

#[soothfast::measured(group = "model_serde", covers = "finance_query::FearAndGreed")]
fn de_fear_and_greed() {
    keep(serde_json::from_str::<FearAndGreed>(keep(FEAR_AND_GREED_JSON)).unwrap());
}

#[soothfast::measured(group = "model_serde", covers = "finance_query::FearAndGreed")]
fn de_fear_and_greed_crypto_history() {
    keep(
        serde_json::from_str::<Vec<FearAndGreed>>(keep(FEAR_AND_GREED_CRYPTO_HISTORY_JSON))
            .unwrap(),
    );
}

#[soothfast::measured(group = "model_serde", covers = "finance_query::MarketHours")]
fn de_hours() {
    keep(serde_json::from_str::<MarketHours>(keep(HOURS_JSON)).unwrap());
}

#[fixture]
fn parsed_currencies() -> Vec<Currency> {
    serde_json::from_str(CURRENCIES_JSON).unwrap()
}

#[bench(group = "model_serde", setup = parsed_currencies, covers = "finance_query::Currency")]
fn ser_currencies(currencies: &[Currency]) {
    keep(serde_json::to_string(keep(currencies)).unwrap());
}

// ── Endpoint response (de)serialization (real captured server payloads) ──────
//
// These are the heaviest per-request serde paths in production. Fixtures are
// real `GET /v2/...` responses (see `.claude/rules/benches.md`), so the gate
// tracks the exact shapes users parse. `dataframe` conversions are
// intentionally excluded (Polars is too heavy for the gate — criterion-only).

static QUOTE_JSON: &str = include_str!("fixtures/quote.json");
static CHART_JSON: &str = include_str!("fixtures/chart.json");
static OPTIONS_JSON: &str = include_str!("fixtures/options.json");
static FINANCIALS_JSON: &str = include_str!("fixtures/financials.json");
static SCREENER_JSON: &str = include_str!("fixtures/screener.json");
static EDGAR_SUBMISSIONS_JSON: &str = include_str!("fixtures/edgar_submissions.json");
static EDGAR_FACTS_JSON: &str = include_str!("fixtures/edgar_facts.json");

#[soothfast::measured(group = "endpoint_serde", p99 = "3ms", covers = "finance_query::Quote")]
fn de_quote() {
    keep(serde_json::from_str::<Quote>(keep(QUOTE_JSON)).unwrap());
}

#[soothfast::measured(group = "endpoint_serde", covers = "finance_query::Chart")]
fn de_chart() {
    keep(serde_json::from_str::<Chart>(keep(CHART_JSON)).unwrap());
}

#[soothfast::measured(group = "endpoint_serde", covers = "finance_query::Options")]
fn de_options() {
    keep(serde_json::from_str::<Options>(keep(OPTIONS_JSON)).unwrap());
}

#[soothfast::measured(group = "endpoint_serde", covers = "finance_query::FinancialStatement")]
fn de_financials() {
    keep(serde_json::from_str::<FinancialStatement>(keep(FINANCIALS_JSON)).unwrap());
}

#[soothfast::measured(group = "endpoint_serde", covers = "finance_query::ScreenerResults")]
fn de_screener() {
    keep(serde_json::from_str::<ScreenerResults>(keep(SCREENER_JSON)).unwrap());
}

#[soothfast::measured(group = "endpoint_serde", covers = "finance_query::EdgarSubmissions")]
fn de_edgar_submissions() {
    keep(serde_json::from_str::<EdgarSubmissions>(keep(EDGAR_SUBMISSIONS_JSON)).unwrap());
}

#[soothfast::measured(group = "endpoint_serde", covers = "finance_query::CompanyFacts")]
fn de_edgar_facts() {
    keep(serde_json::from_str::<CompanyFacts>(keep(EDGAR_FACTS_JSON)).unwrap());
}

// ── FRED economic series + Treasury yield curve (feature: fred) ───────────────

static FRED_SERIES_JSON: &str = include_str!("fixtures/fred_series.json");
static TREASURY_YIELDS_JSON: &str = include_str!("fixtures/treasury_yields.json");

#[soothfast::measured(group = "economic_serde", covers = "finance_query::fred::MacroSeries")]
fn de_fred_series() {
    keep(serde_json::from_str::<MacroSeries>(keep(FRED_SERIES_JSON)).unwrap());
}

#[soothfast::measured(
    group = "economic_serde",
    covers = "finance_query::fred::TreasuryYield"
)]
fn de_treasury_yields() {
    keep(serde_json::from_str::<Vec<TreasuryYield>>(keep(TREASURY_YIELDS_JSON)).unwrap());
}

// ── CoinGecko crypto market data (feature: crypto) ───────────────────────────

static CRYPTO_JSON: &str = include_str!("fixtures/crypto.json");

#[soothfast::measured(group = "crypto_serde", covers = "finance_query::crypto::CoinQuote")]
fn de_crypto_coins() {
    keep(serde_json::from_str::<Vec<CoinQuote>>(keep(CRYPTO_JSON)).unwrap());
}

// ── RSS/Atom feed parsing ────────────────────────────────────────────────────

static FEED_RSS: &[u8] = include_bytes!("fixtures/feed_rss.xml");

#[fixture]
fn feed_bytes() -> Vec<u8> {
    FEED_RSS.to_vec()
}

#[bench(group = "feeds", setup = feed_bytes, covers = "finance_query::feeds::parse_bytes")]
fn rss_parse(bytes: &[u8]) {
    keep(finance_query::feeds::parse_bytes(keep(bytes), "bench").unwrap());
}

// ── Dictionary-tier translation (feature: translation, pure Rust) ─────────────
//
// `translate_texts` is async, but the dictionary tier is sync compute; we
// drive it on a single-threaded runtime built in setup (outside the measured
// region). With `translation` (and NOT `translation-offline`) only the
// dictionary backend runs — deterministic, no model load, no network.

#[fixture]
fn translation_inputs() -> (tokio::runtime::Runtime, Vec<String>, Lang) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let texts: Vec<String> = [
        "Strong Buy",
        "Buy",
        "Hold",
        "Sell",
        "Underperform",
        "Market Cap",
        "Earnings per share grew year over year.",
        "The company raised its full-year revenue guidance.",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    let lang = Lang::parse("es").unwrap();
    (rt, texts, lang)
}

// Small enough that a codegen change alone crosses +5% with source unchanged.
#[bench(
    group = "translation",
    setup = translation_inputs,
    tolerance = "8%",
    covers = "finance_query::translation::translate_texts"
)]
fn translate_dictionary(input: &(tokio::runtime::Runtime, Vec<String>, Lang)) {
    let (rt, texts, lang) = input;
    keep(
        rt.block_on(translate_texts(keep(texts), keep(lang)))
            .unwrap(),
    );
}

/// Echoes input back unchanged — deterministic, no network, no model load.
struct EchoBackend;

#[async_trait::async_trait]
impl TranslationBackend for EchoBackend {
    fn id(&self) -> &'static str {
        "bench-echo"
    }

    async fn translate_batch(
        &self,
        texts: &[String],
        _target: &Lang,
    ) -> finance_query::Result<Vec<String>> {
        Ok(texts.to_vec())
    }
}

#[soothfast::measured(
    group = "translation",
    covers = "finance_query::translation::set_backend"
)]
fn translation_set_backend() {
    set_backend(keep(std::sync::Arc::new(EchoBackend)));
}

#[fixture]
fn translate_news_inputs() -> (tokio::runtime::Runtime, Vec<News>, Lang) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let news: Vec<News> = serde_json::from_str(NEWS_JSON).unwrap();
    let lang = Lang::parse("ja").unwrap();
    (rt, news, lang)
}

#[bench(
    group = "translation",
    setup = translate_news_inputs,
    covers = "finance_query::translation::translate"
)]
fn translation_translate(input: &(tokio::runtime::Runtime, Vec<News>, Lang)) {
    let (rt, news, lang) = input;
    let mut news = news.clone();
    rt.block_on(translate(keep(&mut news), &lang.code()))
        .unwrap();
    keep(news);
}

#[bench(
    group = "translation",
    setup = translate_news_inputs,
    covers = "finance_query::translation::translate_with"
)]
fn translation_translate_with(input: &(tokio::runtime::Runtime, Vec<News>, Lang)) {
    let (rt, news, lang) = input;
    let mut news = news.clone();
    rt.block_on(translate_with(keep(&mut news), lang)).unwrap();
    keep(news);
}

// ── Offline VADER sentiment scoring (feature: sentiment, pure Rust) ───────────
//
// Gates the lexicon-scoring path behind `Ticker::news()` /
// `Transcript::overall_sentiment()` over real captured payloads.

static NEWS_SYMBOL_JSON: &str = include_str!("fixtures/news_symbol.json");
static SENTIMENT_TRANSCRIPT_JSON: &str = include_str!("fixtures/transcripts.json");

#[fixture]
fn news_titles() -> Vec<String> {
    let news: Vec<News> = serde_json::from_str(NEWS_SYMBOL_JSON).unwrap();
    news.into_iter().map(|n| n.title).collect()
}

// Small enough that its instruction count tracks codegen-unit composition:
// reshuffling unrelated modules moves it ~5% without touching sentiment code.
#[bench(
    group = "sentiment",
    setup = news_titles,
    covers = "finance_query::analyze_sentiment",
    tolerance = "8%"
)]
fn score_news(titles: &[String]) {
    keep(
        titles
            .iter()
            .filter(|t| keep(analyze_sentiment(t)).score != 0.0)
            .count(),
    );
}

#[soothfast::measured(
    group = "sentiment",
    covers = "finance_query::Transcript::overall_sentiment"
)]
fn score_transcript() {
    let t: Transcript = serde_json::from_str(SENTIMENT_TRANSCRIPT_JSON).unwrap();
    keep(keep(&t).overall_sentiment().score);
}

#[fixture]
fn canned_ticker_only() -> Ticker {
    tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(
            Ticker::builder("AAPL")
                .with_provider_set(canned_provider_set())
                .build(),
        )
        .unwrap()
}

/// The same dispatch-then-cache-hit path as `ticker_quote_then_cached`, but
/// awaited directly so it runs on the counting executor. Gates how many
/// times the quote future is polled, which instruction counts cannot see.
#[bench(
    group = "ticker",
    setup = canned_ticker_only,
    covers = "finance_query::Ticker::quote"
)]
async fn ticker_quote_polls(ticker: &Ticker) {
    let first = ticker.quote::<finance_query::format::Raw>().await.unwrap();
    let second = ticker.quote::<finance_query::format::Raw>().await.unwrap();
    keep((first, second));
}
