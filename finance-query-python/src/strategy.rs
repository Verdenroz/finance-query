//! PyO3 wrappers for the six prebuilt backtest strategies.
//!
//! Each wrapper is a frozen `#[pyclass]` that holds the corresponding Rust strategy.
//! `StrategyKind` and `extract_strategy` are `pub` for use by Task 15 (backtest methods).

use pyo3::prelude::*;

use finance_query::backtesting::strategy::prebuilt::{
    BollingerMeanReversion as RsBoll, DonchianBreakout as RsDonchian, MacdSignal as RsMacd,
    RsiReversal as RsRsi, SmaCrossover as RsSma, SuperTrendFollow as RsSuper,
};

// ── PyO3 wrapper classes ──────────────────────────────────────────────────────

#[pyclass(frozen, name = "SmaCrossover")]
#[derive(Clone)]
pub struct PySmaCrossover(pub RsSma);

#[pymethods]
impl PySmaCrossover {
    #[new]
    fn new(fast_period: usize, slow_period: usize) -> Self {
        Self(RsSma::new(fast_period, slow_period))
    }
}

#[pyclass(frozen, name = "RsiReversal")]
#[derive(Clone)]
pub struct PyRsiReversal(pub RsRsi);

#[pymethods]
impl PyRsiReversal {
    #[new]
    fn new(period: usize) -> Self {
        Self(RsRsi::new(period))
    }
}

#[pyclass(frozen, name = "MacdSignal")]
#[derive(Clone)]
pub struct PyMacdSignal(pub RsMacd);

#[pymethods]
impl PyMacdSignal {
    #[new]
    fn new(fast: usize, slow: usize, signal: usize) -> Self {
        Self(RsMacd::new(fast, slow, signal))
    }
}

#[pyclass(frozen, name = "BollingerMeanReversion")]
#[derive(Clone)]
pub struct PyBollingerMeanReversion(pub RsBoll);

#[pymethods]
impl PyBollingerMeanReversion {
    #[new]
    fn new(period: usize, std_dev: f64) -> Self {
        Self(RsBoll::new(period, std_dev))
    }
}

#[pyclass(frozen, name = "SuperTrendFollow")]
#[derive(Clone)]
pub struct PySuperTrendFollow(pub RsSuper);

#[pymethods]
impl PySuperTrendFollow {
    #[new]
    fn new(period: usize, multiplier: f64) -> Self {
        Self(RsSuper::new(period, multiplier))
    }
}

#[pyclass(frozen, name = "DonchianBreakout")]
#[derive(Clone)]
pub struct PyDonchianBreakout(pub RsDonchian);

#[pymethods]
impl PyDonchianBreakout {
    #[new]
    fn new(period: usize) -> Self {
        Self(RsDonchian::new(period))
    }
}

// ── StrategyKind dispatch enum (consumed by Task 15) ─────────────────────────

/// Typed dispatch enum — each variant holds the unwrapped Rust strategy.
/// Produced by [`extract_strategy`]; consumed by backtest runner in Task 15.
pub enum StrategyKind {
    Sma(RsSma),
    Rsi(RsRsi),
    Macd(RsMacd),
    Boll(RsBoll),
    Super(RsSuper),
    Donchian(RsDonchian),
}

/// Downcast a Python strategy object (`&Py<PyAny>`) to one of the five known
/// strategy classes and return a [`StrategyKind`].
///
/// Returns `PyResult::Err` with a `TypeError` if the object is not one of the
/// five known strategy classes.
pub fn extract_strategy(py: Python<'_>, obj: &Py<PyAny>) -> PyResult<StrategyKind> {
    let bound = obj.bind(py);

    if let Ok(s) = bound.downcast::<PySmaCrossover>() {
        return Ok(StrategyKind::Sma(s.get().0.clone()));
    }
    if let Ok(s) = bound.downcast::<PyRsiReversal>() {
        return Ok(StrategyKind::Rsi(s.get().0.clone()));
    }
    if let Ok(s) = bound.downcast::<PyMacdSignal>() {
        return Ok(StrategyKind::Macd(s.get().0.clone()));
    }
    if let Ok(s) = bound.downcast::<PyBollingerMeanReversion>() {
        return Ok(StrategyKind::Boll(s.get().0.clone()));
    }
    if let Ok(s) = bound.downcast::<PySuperTrendFollow>() {
        return Ok(StrategyKind::Super(s.get().0.clone()));
    }
    if let Ok(s) = bound.downcast::<PyDonchianBreakout>() {
        return Ok(StrategyKind::Donchian(s.get().0.clone()));
    }

    Err(pyo3::exceptions::PyTypeError::new_err(
        "Expected one of: SmaCrossover, RsiReversal, MacdSignal, BollingerMeanReversion, SuperTrendFollow, DonchianBreakout",
    ))
}

// ── Async dispatch helpers (called by ticker.rs / tickers.rs) ────────────────

/// Run `Ticker::backtest` for the given `StrategyKind`, dispatching to the
/// correct monomorphised call.  Must be called inside `future_into_py`.
pub async fn run_backtest(
    ticker: &finance_query::Ticker,
    kind: StrategyKind,
    interval: finance_query::Interval,
    range: finance_query::TimeRange,
    config: Option<finance_query::backtesting::BacktestConfig>,
) -> finance_query::backtesting::Result<finance_query::backtesting::BacktestResult> {
    match kind {
        StrategyKind::Sma(s) => ticker.backtest(s, interval, range, config).await,
        StrategyKind::Rsi(s) => ticker.backtest(s, interval, range, config).await,
        StrategyKind::Macd(s) => ticker.backtest(s, interval, range, config).await,
        StrategyKind::Boll(s) => ticker.backtest(s, interval, range, config).await,
        StrategyKind::Super(s) => ticker.backtest(s, interval, range, config).await,
        StrategyKind::Donchian(s) => ticker.backtest(s, interval, range, config).await,
    }
}

/// Run `Ticker::backtest_with_benchmark` for the given `StrategyKind`.
pub async fn run_backtest_with_benchmark(
    ticker: &finance_query::Ticker,
    kind: StrategyKind,
    interval: finance_query::Interval,
    range: finance_query::TimeRange,
    config: Option<finance_query::backtesting::BacktestConfig>,
    benchmark: String,
) -> finance_query::backtesting::Result<finance_query::backtesting::BacktestResult> {
    match kind {
        StrategyKind::Sma(s) => {
            ticker
                .backtest_with_benchmark(s, interval, range, config, &benchmark)
                .await
        }
        StrategyKind::Rsi(s) => {
            ticker
                .backtest_with_benchmark(s, interval, range, config, &benchmark)
                .await
        }
        StrategyKind::Macd(s) => {
            ticker
                .backtest_with_benchmark(s, interval, range, config, &benchmark)
                .await
        }
        StrategyKind::Boll(s) => {
            ticker
                .backtest_with_benchmark(s, interval, range, config, &benchmark)
                .await
        }
        StrategyKind::Super(s) => {
            ticker
                .backtest_with_benchmark(s, interval, range, config, &benchmark)
                .await
        }
        StrategyKind::Donchian(s) => {
            ticker
                .backtest_with_benchmark(s, interval, range, config, &benchmark)
                .await
        }
    }
}

/// Run `Tickers::backtest` for the given `StrategyKind`.
///
/// The `Tickers::backtest` API takes a `factory: F: Fn(&str) -> S`, so we
/// clone the strategy from the enum for each symbol.
pub async fn run_backtest_portfolio(
    tickers: &finance_query::Tickers,
    kind: StrategyKind,
    interval: finance_query::Interval,
    range: finance_query::TimeRange,
    config: Option<finance_query::backtesting::portfolio::PortfolioConfig>,
) -> finance_query::backtesting::Result<finance_query::backtesting::portfolio::PortfolioResult> {
    match kind {
        StrategyKind::Sma(s) => {
            tickers
                .backtest(interval, range, config, move |_| s.clone())
                .await
        }
        StrategyKind::Rsi(s) => {
            tickers
                .backtest(interval, range, config, move |_| s.clone())
                .await
        }
        StrategyKind::Macd(s) => {
            tickers
                .backtest(interval, range, config, move |_| s.clone())
                .await
        }
        StrategyKind::Boll(s) => {
            tickers
                .backtest(interval, range, config, move |_| s.clone())
                .await
        }
        StrategyKind::Super(s) => {
            tickers
                .backtest(interval, range, config, move |_| s.clone())
                .await
        }
        StrategyKind::Donchian(s) => {
            tickers
                .backtest(interval, range, config, move |_| s.clone())
                .await
        }
    }
}

// ── Module registration ───────────────────────────────────────────────────────

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PySmaCrossover>()?;
    m.add_class::<PyRsiReversal>()?;
    m.add_class::<PyMacdSignal>()?;
    m.add_class::<PyBollingerMeanReversion>()?;
    m.add_class::<PySuperTrendFollow>()?;
    m.add_class::<PyDonchianBreakout>()?;
    Ok(())
}
