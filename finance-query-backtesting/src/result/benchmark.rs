use serde::{Deserialize, Serialize};

/// Comparison of strategy performance against a benchmark.
///
/// Populated when a benchmark symbol is supplied to `backtest_with_benchmark`.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkMetrics {
    /// Benchmark symbol (e.g. `"SPY"`)
    pub symbol: String,

    /// Buy-and-hold return of the benchmark over the same period (percentage)
    pub benchmark_return_pct: f64,

    /// Buy-and-hold return of the backtested symbol over the same period (percentage)
    pub buy_and_hold_return_pct: f64,

    /// Jensen's Alpha: annualised strategy excess return over the benchmark (CAPM).
    ///
    /// Computed as `strategy_ann - rf - β × (benchmark_ann - rf)` on the
    /// timestamp-aligned subset of strategy and benchmark returns.
    ///
    /// # Accuracy Caveat
    ///
    /// Annualisation uses `aligned_bars / bars_per_year` to estimate elapsed
    /// years.  If the strategy and benchmark candles have **different sampling
    /// frequencies** (e.g., daily strategy vs. weekly benchmark), the aligned
    /// subset contains far fewer bars than the full backtest period and the
    /// per-year estimate will be wrong — both `strategy_ann` and `benchmark_ann`
    /// are inflated by the same factor, but the risk-free rate is always the
    /// true annual rate, making alpha unreliable.
    ///
    /// For accurate alpha, supply benchmark candles with the **same interval**
    /// as the strategy candles.
    pub alpha: f64,

    /// Beta: sensitivity of strategy returns to benchmark movements
    pub beta: f64,

    /// Information ratio: excess return per unit of tracking error (annualised)
    pub information_ratio: f64,

    /// Tracking error: annualised standard deviation of (strategy − benchmark)
    /// periodic returns — the denominator of `information_ratio`.
    pub tracking_error: f64,
}
