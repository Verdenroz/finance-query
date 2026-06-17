//! Deterministic instruction-count regression gate (iai-callgrind).
//!
//! Unlike the criterion benches in this directory — which measure wall-clock
//! time and are too noisy to gate CI on — these benchmarks count CPU
//! instructions via valgrind/callgrind. Instruction counts are deterministic
//! across runs and machines, so a change in the count is a real change in the
//! work done, not measurement noise. That makes them safe to fail CI on.
//!
//! The gate spans every compute/parse/dispatch facet of the library:
//! indicators, the backtesting engine, risk analytics, streaming (de)serialize,
//! provider capability dispatch, and model (de)serialization. (The `ticker` /
//! `tickers` criterion benches are intentionally excluded — they compare
//! internal allocation strategies, `std` container micro-ops rather than
//! library logic.)
//!
//! Each benchmark carries soft limits of +5% on instructions (`Ir`, raw work)
//! and +10% on `EstimatedCycles` (a cache-weighted time proxy — looser because
//! it's noisier than pure instruction counts). CI saves a baseline from the
//! target branch, then re-runs on the PR; any benchmark exceeding either limit
//! over its baseline fails the gate.
//!
//! ## Running
//!
//! Requires `valgrind`. This host's environment matters: a glibc compiled for
//! `x86-64-v4` (e.g. CachyOS/Arch with AVX-512) emits AVX-512 in its startup
//! code, which valgrind 3.25 cannot decode (SIGILL). Run on vanilla glibc
//! (CI's Ubuntu, or a Debian container — see `.claude/rules/benches.md`):
//!
//! ```text
//! make baseline                # Debian container (recommended; vanilla glibc)
//! cargo bench --bench regression --features bench-gate   # vanilla-glibc hosts/CI
//! ```
//!
//! The `GLIBC_TUNABLES` entry below additionally stops glibc from dispatching
//! to AVX-512 string routines (`memcpy`/`memmove`) at runtime, hardening the
//! gate on AVX-512 hosts that *do* have a valgrind-decodable loader.

use finance_query::backtesting::{BacktestConfig, BacktestEngine, SmaCrossover};
use finance_query::crypto::CoinQuote;
use finance_query::fred::{MacroSeries, TreasuryYield};
use finance_query::indicators::{
    accumulation_distribution, adx, alma, aroon, atr, awesome_oscillator, balance_of_power,
    bollinger_bands, bull_bear_power, cci, chaikin_oscillator, choppiness_index, cmf, cmo,
    coppock_curve, dema, donchian_channels, elder_ray, ema, hma, ichimoku, keltner_channels, macd,
    mcginley_dynamic, mfi, momentum, obv, parabolic_sar, patterns, roc, rsi, sma, stochastic,
    stochastic_rsi, supertrend, tema, true_range, vwap, vwma, williams_r, wma,
};
use finance_query::risk::{
    beta, historical_var, max_drawdown, parametric_var, sharpe_ratio, sortino_ratio,
};
use finance_query::streaming::{MarketHoursType, OptionType, PriceUpdate, QuoteType};
use finance_query::translation::{Lang, translate_texts};
use finance_query::{
    Candle, Chart, CompanyFacts, Currency, EdgarSubmissions, FinancialStatement, News, Options,
    Quote, ScreenerResults, SearchResults, Transcript, analyze_sentiment,
};
use iai_callgrind::{
    Callgrind, EventKind, LibraryBenchmarkConfig, library_benchmark, library_benchmark_group, main,
};
use std::hint::black_box;

// ── Deterministic synthetic inputs (no network, no randomness) ───────────────

/// A reproducible pseudo-random walk of close prices.
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

// Setup functions run *outside* the measured region — iai-callgrind requires a
// function path (not a closure) for the `setup` argument.
fn returns_1000() -> Vec<f64> {
    synthetic_returns(1000)
}
fn returns_pair_1000() -> (Vec<f64>, Vec<f64>) {
    (synthetic_returns(1000), synthetic_returns(1000))
}

// ── Indicator hot paths ──────────────────────────────────────────────────────
//
// The gate spans the whole indicator suite (all 42), grouped by family so a
// regression in any single indicator pushes its family's instruction count past
// the +5% soft limit. Families mirror `src/indicators/` and the criterion bench
// `benches/indicators.rs`. The per-family setup extracts O/H/L/C/V arrays once,
// outside the measured region.

/// `(opens, highs, lows, closes, volumes)` — extracted from synthetic candles.
type Series = (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>);

fn series_1000() -> Series {
    let candles = synthetic_candles(1000);
    let opens = candles.iter().map(|c| c.open).collect();
    let highs = candles.iter().map(|c| c.high).collect();
    let lows = candles.iter().map(|c| c.low).collect();
    let closes = candles.iter().map(|c| c.close).collect();
    let volumes = candles.iter().map(|c| c.volume as f64).collect();
    (opens, highs, lows, closes, volumes)
}

fn candles_1000() -> Vec<Candle> {
    synthetic_candles(1000)
}

#[library_benchmark]
#[bench::n1000(setup = series_1000)]
fn ind_moving_averages(s: Series) {
    let (_, _, _, closes, volumes) = &s;
    let _ = black_box(sma(black_box(closes), 20));
    let _ = black_box(sma(black_box(closes), 200));
    let _ = black_box(ema(black_box(closes), 20));
    let _ = black_box(ema(black_box(closes), 200));
    let _ = black_box(wma(black_box(closes), 20));
    let _ = black_box(hma(black_box(closes), 20));
    let _ = black_box(dema(black_box(closes), 20));
    let _ = black_box(tema(black_box(closes), 20));
    let _ = black_box(alma(black_box(closes), 9, 0.85, 6.0));
    let _ = black_box(mcginley_dynamic(black_box(closes), 20));
    let _ = black_box(vwma(black_box(closes), black_box(volumes), 20));
}

#[library_benchmark]
#[bench::n1000(setup = series_1000)]
fn ind_momentum(s: Series) {
    let (_, highs, lows, closes, _) = &s;
    let _ = black_box(rsi(black_box(closes), 14));
    let _ = black_box(stochastic(
        black_box(highs),
        black_box(lows),
        black_box(closes),
        14,
        1,
        3,
    ));
    let _ = black_box(stochastic_rsi(black_box(closes), 14, 14, 3, 3));
    let _ = black_box(cci(
        black_box(highs),
        black_box(lows),
        black_box(closes),
        20,
    ));
    let _ = black_box(macd(black_box(closes), 12, 26, 9));
    let _ = black_box(williams_r(
        black_box(highs),
        black_box(lows),
        black_box(closes),
        14,
    ));
    let _ = black_box(roc(black_box(closes), 12));
    let _ = black_box(momentum(black_box(closes), 10));
    let _ = black_box(cmo(black_box(closes), 14));
    let _ = black_box(awesome_oscillator(black_box(highs), black_box(lows), 5, 34));
    let _ = black_box(coppock_curve(black_box(closes), 14, 11, 10));
}

#[library_benchmark]
#[bench::n1000(setup = series_1000)]
fn ind_trend(s: Series) {
    let (_, highs, lows, closes, _) = &s;
    let _ = black_box(adx(
        black_box(highs),
        black_box(lows),
        black_box(closes),
        14,
    ));
    let _ = black_box(aroon(black_box(highs), black_box(lows), 25));
    let _ = black_box(supertrend(
        black_box(highs),
        black_box(lows),
        black_box(closes),
        10,
        3.0,
    ));
    let _ = black_box(ichimoku(
        black_box(highs),
        black_box(lows),
        black_box(closes),
        9,
        26,
        26,
        26,
    ));
    let _ = black_box(parabolic_sar(
        black_box(highs),
        black_box(lows),
        black_box(closes),
        0.02,
        0.2,
    ));
    let _ = black_box(bull_bear_power(
        black_box(highs),
        black_box(lows),
        black_box(closes),
        13,
    ));
    let _ = black_box(elder_ray(
        black_box(highs),
        black_box(lows),
        black_box(closes),
        13,
    ));
}

#[library_benchmark]
#[bench::n1000(setup = series_1000)]
fn ind_volatility(s: Series) {
    let (_, highs, lows, closes, _) = &s;
    let _ = black_box(bollinger_bands(black_box(closes), 20, 2.0));
    let _ = black_box(keltner_channels(
        black_box(highs),
        black_box(lows),
        black_box(closes),
        20,
        10,
        2.0,
    ));
    let _ = black_box(donchian_channels(black_box(highs), black_box(lows), 20));
    let _ = black_box(atr(
        black_box(highs),
        black_box(lows),
        black_box(closes),
        14,
    ));
    let _ = black_box(true_range(
        black_box(highs),
        black_box(lows),
        black_box(closes),
    ));
    let _ = black_box(choppiness_index(
        black_box(highs),
        black_box(lows),
        black_box(closes),
        14,
    ));
}

#[library_benchmark]
#[bench::n1000(setup = series_1000)]
fn ind_volume(s: Series) {
    let (opens, highs, lows, closes, volumes) = &s;
    let _ = black_box(obv(black_box(closes), black_box(volumes)));
    let _ = black_box(mfi(
        black_box(highs),
        black_box(lows),
        black_box(closes),
        black_box(volumes),
        14,
    ));
    let _ = black_box(cmf(
        black_box(highs),
        black_box(lows),
        black_box(closes),
        black_box(volumes),
        20,
    ));
    let _ = black_box(chaikin_oscillator(
        black_box(highs),
        black_box(lows),
        black_box(closes),
        black_box(volumes),
    ));
    let _ = black_box(accumulation_distribution(
        black_box(highs),
        black_box(lows),
        black_box(closes),
        black_box(volumes),
    ));
    let _ = black_box(vwap(
        black_box(highs),
        black_box(lows),
        black_box(closes),
        black_box(volumes),
    ));
    let _ = black_box(balance_of_power(
        black_box(opens),
        black_box(highs),
        black_box(lows),
        black_box(closes),
        None,
    ));
}

#[library_benchmark]
#[bench::n1000(setup = candles_1000)]
fn ind_patterns(candles: Vec<Candle>) {
    let _ = black_box(patterns(black_box(&candles)));
}

library_benchmark_group!(
    name = indicators;
    benchmarks = ind_moving_averages, ind_momentum, ind_trend, ind_volatility, ind_volume,
        ind_patterns
);

// ── Backtesting engine ───────────────────────────────────────────────────────

fn backtest_inputs() -> (BacktestConfig, Vec<Candle>) {
    let config = BacktestConfig::builder()
        .initial_capital(10_000.0)
        .commission_pct(0.001)
        .build()
        .unwrap();
    (config, synthetic_candles(1000))
}

#[library_benchmark]
#[bench::n1000(setup = backtest_inputs)]
fn bt_sma_crossover(input: (BacktestConfig, Vec<Candle>)) {
    let (config, candles) = input;
    let engine = BacktestEngine::new(config);
    let strategy = SmaCrossover::new(10, 20);
    let _ = black_box(engine.run(black_box("BENCH"), black_box(&candles), strategy));
}

library_benchmark_group!(
    name = backtesting;
    benchmarks = bt_sma_crossover
);

// ── Risk metric hot paths ────────────────────────────────────────────────────

#[library_benchmark]
#[bench::n1000(setup = returns_1000)]
fn risk_historical_var(returns: Vec<f64>) -> Option<f64> {
    black_box(historical_var(black_box(&returns), 0.95))
}

#[library_benchmark]
#[bench::n1000(setup = returns_1000)]
fn risk_parametric_var(returns: Vec<f64>) -> Option<f64> {
    black_box(parametric_var(black_box(&returns), 0.95))
}

#[library_benchmark]
#[bench::n1000(setup = returns_1000)]
fn risk_sharpe(returns: Vec<f64>) -> Option<f64> {
    black_box(sharpe_ratio(black_box(&returns), 0.0, 252.0))
}

#[library_benchmark]
#[bench::n1000(setup = returns_1000)]
fn risk_sortino(returns: Vec<f64>) -> Option<f64> {
    black_box(sortino_ratio(black_box(&returns), 0.0, 252.0))
}

#[library_benchmark]
#[bench::n1000(setup = returns_1000)]
fn risk_max_drawdown(returns: Vec<f64>) -> (f64, Option<u64>) {
    // `DrawdownResult` is not publicly nameable; return its fields so the full
    // computation is still measured and not optimised away.
    let dd = max_drawdown(black_box(&returns));
    black_box((dd.max_drawdown, dd.recovery_periods))
}

#[library_benchmark]
#[bench::n1000(setup = returns_pair_1000)]
fn risk_beta(series: (Vec<f64>, Vec<f64>)) -> Option<f64> {
    black_box(beta(black_box(&series.0), black_box(&series.1)))
}

library_benchmark_group!(
    name = risk;
    benchmarks = risk_historical_var, risk_parametric_var, risk_sharpe, risk_sortino,
        risk_max_drawdown, risk_beta
);

// ── Streaming (PriceUpdate serde — the per-tick hot path) ─────────────────────

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

fn price_update_json() -> String {
    serde_json::to_string(&make_price_update()).unwrap()
}

#[library_benchmark]
#[bench::one(setup = make_price_update)]
fn stream_serialize(update: PriceUpdate) -> String {
    black_box(serde_json::to_string(black_box(&update)).unwrap())
}

#[library_benchmark]
#[bench::one(setup = price_update_json)]
fn stream_deserialize(json: String) -> PriceUpdate {
    black_box(serde_json::from_str(black_box(&json)).unwrap())
}

library_benchmark_group!(
    name = streaming;
    benchmarks = stream_serialize, stream_deserialize
);

// ── Provider capability dispatch (the per-fetch selection) ───────────────────

use finance_query::Capability;

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

#[library_benchmark]
#[bench::quote(setup = provider_registry)]
fn dispatch_select(registry: Vec<Capability>) -> usize {
    let wanted = black_box(Capability::QUOTE);
    black_box(registry.iter().filter(|c| c.contains(wanted)).count())
}

library_benchmark_group!(
    name = providers;
    benchmarks = dispatch_select
);

// ── Model (de)serialization (real Yahoo-shaped fixtures) ─────────────────────

static SEARCH_JSON: &str = include_str!("fixtures/search.json");
static NEWS_JSON: &str = include_str!("fixtures/news.json");
static CURRENCIES_JSON: &str = include_str!("fixtures/currencies.json");

#[library_benchmark]
fn de_search() -> SearchResults {
    black_box(serde_json::from_str(black_box(SEARCH_JSON)).unwrap())
}

#[library_benchmark]
fn de_news() -> Vec<News> {
    black_box(serde_json::from_str(black_box(NEWS_JSON)).unwrap())
}

#[library_benchmark]
fn de_currencies() -> Vec<Currency> {
    black_box(serde_json::from_str(black_box(CURRENCIES_JSON)).unwrap())
}

fn parsed_currencies() -> Vec<Currency> {
    serde_json::from_str(CURRENCIES_JSON).unwrap()
}

#[library_benchmark]
#[bench::c168(setup = parsed_currencies)]
fn ser_currencies(currencies: Vec<Currency>) -> String {
    black_box(serde_json::to_string(black_box(&currencies)).unwrap())
}

library_benchmark_group!(
    name = model_serde;
    benchmarks = de_search, de_news, de_currencies, ser_currencies
);

// ── Endpoint response (de)serialization (real captured server payloads) ──────
//
// These are the heaviest per-request serde paths in production. Fixtures are
// real `GET /v2/...` responses (see `.claude/rules/benches.md`), so the gate
// tracks the exact shapes users parse. `dataframe` conversions are intentionally
// excluded (Polars is too heavy for the valgrind gate — criterion-only).

static QUOTE_JSON: &str = include_str!("fixtures/quote.json");
static CHART_JSON: &str = include_str!("fixtures/chart.json");
static OPTIONS_JSON: &str = include_str!("fixtures/options.json");
static FINANCIALS_JSON: &str = include_str!("fixtures/financials.json");
static SCREENER_JSON: &str = include_str!("fixtures/screener.json");
static EDGAR_SUBMISSIONS_JSON: &str = include_str!("fixtures/edgar_submissions.json");
static EDGAR_FACTS_JSON: &str = include_str!("fixtures/edgar_facts.json");

#[library_benchmark]
fn de_quote() -> Quote {
    black_box(serde_json::from_str(black_box(QUOTE_JSON)).unwrap())
}

#[library_benchmark]
fn de_chart() -> Chart {
    black_box(serde_json::from_str(black_box(CHART_JSON)).unwrap())
}

#[library_benchmark]
fn de_options() -> Options {
    black_box(serde_json::from_str(black_box(OPTIONS_JSON)).unwrap())
}

#[library_benchmark]
fn de_financials() -> FinancialStatement {
    black_box(serde_json::from_str(black_box(FINANCIALS_JSON)).unwrap())
}

#[library_benchmark]
fn de_screener() -> ScreenerResults {
    black_box(serde_json::from_str(black_box(SCREENER_JSON)).unwrap())
}

#[library_benchmark]
fn de_edgar_submissions() -> EdgarSubmissions {
    black_box(serde_json::from_str(black_box(EDGAR_SUBMISSIONS_JSON)).unwrap())
}

#[library_benchmark]
fn de_edgar_facts() -> CompanyFacts {
    black_box(serde_json::from_str(black_box(EDGAR_FACTS_JSON)).unwrap())
}

library_benchmark_group!(
    name = endpoint_serde;
    benchmarks = de_quote, de_chart, de_options, de_financials, de_screener,
        de_edgar_submissions, de_edgar_facts
);

// ── FRED economic series + Treasury yield curve (feature: fred) ───────────────
//
// Deserializing the series / yield-curve models is the fred adapter's only
// pure-Rust cost. Fixtures are real `GET /v2/fred/...` payloads.

static FRED_SERIES_JSON: &str = include_str!("fixtures/fred_series.json");
static TREASURY_YIELDS_JSON: &str = include_str!("fixtures/treasury_yields.json");

#[library_benchmark]
fn de_fred_series() -> MacroSeries {
    black_box(serde_json::from_str(black_box(FRED_SERIES_JSON)).unwrap())
}

#[library_benchmark]
fn de_treasury_yields() -> Vec<TreasuryYield> {
    black_box(serde_json::from_str(black_box(TREASURY_YIELDS_JSON)).unwrap())
}

library_benchmark_group!(
    name = economic_serde;
    benchmarks = de_fred_series, de_treasury_yields
);

// ── CoinGecko crypto market data (feature: crypto) ───────────────────────────
//
// Deserializing the market array into `CoinQuote` is the crypto adapter's only
// deterministic cost. Fixture is a real `GET /v2/crypto/coins` payload.

static CRYPTO_JSON: &str = include_str!("fixtures/crypto.json");

#[library_benchmark]
fn de_crypto_coins() -> Vec<CoinQuote> {
    black_box(serde_json::from_str(black_box(CRYPTO_JSON)).unwrap())
}

library_benchmark_group!(
    name = crypto_serde;
    benchmarks = de_crypto_coins
);

// ── RSS/Atom feed parsing (feature: rss) ─────────────────────────────────────

static FEED_RSS: &[u8] = include_bytes!("fixtures/feed_rss.xml");

fn feed_bytes() -> &'static [u8] {
    FEED_RSS
}

#[library_benchmark]
#[bench::rss(setup = feed_bytes)]
fn rss_parse(bytes: &[u8]) -> feed_rs::model::Feed {
    black_box(feed_rs::parser::parse(black_box(bytes)).unwrap())
}

library_benchmark_group!(
    name = feeds;
    benchmarks = rss_parse
);

// ── Dictionary-tier translation (feature: translation, pure Rust) ─────────────
//
// `translate` is async, but the dictionary tier is sync compute; we drive it on
// a single-threaded runtime built in setup (outside the measured region). With
// `translation` (and NOT `translation-offline`) only the dictionary backend
// runs — deterministic, no model load, no network.

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

#[library_benchmark]
#[bench::dictionary_es(setup = translation_inputs)]
fn translate_dictionary(input: (tokio::runtime::Runtime, Vec<String>, Lang)) -> Vec<String> {
    let (rt, texts, lang) = input;
    black_box(
        rt.block_on(translate_texts(black_box(&texts), black_box(&lang)))
            .unwrap(),
    )
}

library_benchmark_group!(
    name = translation;
    benchmarks = translate_dictionary
);

// ── Offline VADER sentiment scoring (feature: sentiment, pure Rust) ───────────
//
// Gates the lexicon-scoring path behind `Ticker::news()` /
// `Transcript::overall_sentiment()` over real captured payloads.

static NEWS_SYMBOL_JSON: &str = include_str!("fixtures/news_symbol.json");
static SENTIMENT_TRANSCRIPT_JSON: &str = include_str!("fixtures/transcripts.json");

fn news_titles() -> Vec<String> {
    let news: Vec<News> = serde_json::from_str(NEWS_SYMBOL_JSON).unwrap();
    news.into_iter().map(|n| n.title).collect()
}

#[library_benchmark]
#[bench::headlines(setup = news_titles)]
fn score_news(titles: Vec<String>) -> usize {
    titles
        .iter()
        .filter(|t| black_box(analyze_sentiment(t)).score != 0.0)
        .count()
}

#[library_benchmark]
fn score_transcript() -> f64 {
    let t: Transcript = serde_json::from_str(SENTIMENT_TRANSCRIPT_JSON).unwrap();
    black_box(black_box(&t).overall_sentiment()).score
}

library_benchmark_group!(
    name = sentiment;
    benchmarks = score_news, score_transcript
);

// ── Gate: fail any benchmark whose instruction count regresses > 5% ──────────

main!(
    config = LibraryBenchmarkConfig::default()
        // Ir (raw work, tight) + EstimatedCycles (time proxy folding in cache
        // misses, looser since it's noisier than pure instruction counts).
        .tool(Callgrind::default().soft_limits([
            (EventKind::Ir, 5.0),
            (EventKind::EstimatedCycles, 10.0),
        ]))
        .env(
            "GLIBC_TUNABLES",
            "glibc.cpu.hwcaps=-AVX512F,-AVX512VL,-AVX512BW,-AVX512DQ,-AVX512CD,-AVX512IFMA,-AVX512_VBMI,-AVX512_VBMI2,-AVX512_VNNI,-AVX512_BITALG,-AVX512_VPOPCNTDQ",
        );
    library_benchmark_groups = indicators, backtesting, risk, streaming, providers, model_serde,
        endpoint_serde, economic_serde, crypto_serde, feeds, translation, sentiment
);
