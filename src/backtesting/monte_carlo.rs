//! Monte Carlo simulation for backtesting results.
//!
//! Re-samples trade return sequences to estimate the distribution of outcomes.
//! Uses an embedded xorshift64 PRNG to avoid adding an external dependency.
//!
//! # Example
//!
//! ```ignore
//! use finance_query::backtesting::monte_carlo::{MonteCarloConfig, MonteCarloMethod, MonteCarloResult};
//!
//! // `result` is a completed BacktestResult
//! let mc = MonteCarloConfig::default()
//!     .method(MonteCarloMethod::BlockBootstrap { block_size: 10 })
//!     .run(&result);
//! println!("Median return: {:.2}%", mc.total_return.p50);
//! println!("5th pct drawdown: {:.2}%", mc.max_drawdown.p5 * 100.0);
//! ```

use serde::{Deserialize, Serialize};

use super::result::BacktestResult;

// ── Resampling method ─────────────────────────────────────────────────────────

/// Resampling method used for Monte Carlo simulation.
///
/// Each method makes different assumptions about trade return structure.
/// Choose based on your strategy's autocorrelation characteristics.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum MonteCarloMethod {
    /// Random IID shuffle (Fisher-Yates). Default.
    ///
    /// Treats every trade return as an independent, identically distributed draw
    /// and randomises the sequence. Fast and appropriate for mean-reversion
    /// strategies whose trades are mostly independent. Destroys autocorrelation,
    /// which may underestimate the probability of sustained drawdowns for
    /// trend-following strategies.
    #[default]
    IidShuffle,

    /// Fixed-size block bootstrap.
    ///
    /// Samples consecutive blocks of `block_size` trades (with circular
    /// wrap-around) and reassembles them in random order. Preserves short-range
    /// autocorrelation and regime structure better than IID shuffle. A good
    /// default block size is `sqrt(n_trades)`. More conservative than IID for
    /// trending strategies.
    BlockBootstrap {
        /// Number of consecutive trades per block.
        block_size: usize,
    },

    /// Stationary bootstrap with geometrically-distributed block lengths.
    ///
    /// Like `BlockBootstrap` but block length is drawn from
    /// Geometric(1 / mean_block_size) at each step. Less sensitive to the choice
    /// of block size than the fixed variant — a good default when you are
    /// uncertain about the true autocorrelation length.
    StationaryBootstrap {
        /// Expected (average) number of trades per block.
        mean_block_size: usize,
    },

    /// Parametric simulation assuming normally-distributed trade returns.
    ///
    /// Fits N(μ, σ) to the observed trade returns and generates synthetic
    /// sequences by sampling from that distribution (Box-Muller transform).
    /// Useful when the observed trade count is very small and non-parametric
    /// resampling would produce near-identical sequences. Assumes normality,
    /// which may not hold in fat-tailed markets.
    Parametric,
}

// ── Configuration ─────────────────────────────────────────────────────────────

/// Configuration for Monte Carlo simulation.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonteCarloConfig {
    /// Number of reshuffled equity-curve simulations to run.
    ///
    /// Higher values give more stable percentile estimates but take longer.
    /// Default: `1000`.
    pub num_simulations: usize,

    /// Optional seed for the PRNG, enabling reproducible results.
    ///
    /// `None` (default) uses a fixed internal seed (`12345`). Provide an
    /// explicit seed when you need deterministic output across runs.
    pub seed: Option<u64>,

    /// Resampling method. Default: [`MonteCarloMethod::IidShuffle`].
    pub method: MonteCarloMethod,
}

impl Default for MonteCarloConfig {
    fn default() -> Self {
        Self {
            num_simulations: 1_000,
            seed: None,
            method: MonteCarloMethod::IidShuffle,
        }
    }
}

impl MonteCarloConfig {
    /// Create a new Monte Carlo configuration with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the number of simulations.
    pub fn num_simulations(mut self, n: usize) -> Self {
        self.num_simulations = n;
        self
    }

    /// Set a fixed PRNG seed for reproducible results.
    pub fn seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Set the resampling method.
    pub fn method(mut self, method: MonteCarloMethod) -> Self {
        self.method = method;
        self
    }

    /// Run the Monte Carlo simulation against a completed backtest result.
    ///
    /// Extracts trade returns, generates `num_simulations` synthetic sequences
    /// using the configured [`MonteCarloMethod`], rebuilds a synthetic equity
    /// curve for each, and reports percentile statistics over all outcomes.
    ///
    /// If the result has fewer than 2 trades, every percentile is derived from
    /// the single observed result.
    ///
    /// Use the percentile outputs as a *relative* stress-test tool rather than
    /// a precise probability statement about future performance.
    pub fn run(&self, result: &BacktestResult) -> MonteCarloResult {
        let initial_capital = result.initial_capital;
        let trade_returns: Vec<f64> = result.trades.iter().map(|t| t.return_pct / 100.0).collect();

        if trade_returns.len() < 2 {
            // Not enough trades to reshuffle — return degenerate result
            let obs_return = result.metrics.total_return_pct;
            let obs_dd = result.metrics.max_drawdown_pct;
            let obs_sharpe = result.metrics.sharpe_ratio;
            let obs_pf = result.metrics.profit_factor;
            let trivial = |v: f64| PercentileStats {
                p5: v,
                p25: v,
                p50: v,
                p75: v,
                p95: v,
                mean: v,
            };
            return MonteCarloResult {
                num_simulations: self.num_simulations,
                method: self.method.clone(),
                total_return: trivial(obs_return),
                max_drawdown: trivial(obs_dd),
                sharpe_ratio: trivial(obs_sharpe),
                profit_factor: trivial(obs_pf),
            };
        }

        let seed = self.seed.unwrap_or(12345);
        let mut rng = Xorshift64::new(seed);

        let position_size = result.config.position_size_pct;
        let num_bars = result.equity_curve.len().saturating_sub(1) as f64;
        let years = if result.config.bars_per_year > 0.0 {
            num_bars / result.config.bars_per_year
        } else {
            0.0
        };
        let periods_per_year = if years > 0.0 {
            trade_returns.len() as f64 / years
        } else {
            trade_returns.len().max(1) as f64
        };

        let mut sim_returns: Vec<f64> = Vec::with_capacity(self.num_simulations);
        let mut sim_drawdowns: Vec<f64> = Vec::with_capacity(self.num_simulations);
        let mut sim_sharpes: Vec<f64> = Vec::with_capacity(self.num_simulations);
        let mut sim_pfs: Vec<f64> = Vec::with_capacity(self.num_simulations);

        // Single allocation reused across all simulations.
        let mut sim_buf: Vec<f64> = vec![0.0; trade_returns.len()];

        for _ in 0..self.num_simulations {
            match &self.method {
                MonteCarloMethod::IidShuffle => {
                    sim_buf.copy_from_slice(&trade_returns);
                    fisher_yates_shuffle(&mut sim_buf, &mut rng);
                }
                MonteCarloMethod::BlockBootstrap { block_size } => {
                    block_bootstrap_into(&trade_returns, *block_size, &mut rng, &mut sim_buf);
                }
                MonteCarloMethod::StationaryBootstrap { mean_block_size } => {
                    stationary_bootstrap_into(
                        &trade_returns,
                        *mean_block_size,
                        &mut rng,
                        &mut sim_buf,
                    );
                }
                MonteCarloMethod::Parametric => {
                    parametric_sample_into(&trade_returns, &mut rng, &mut sim_buf);
                }
            }

            // Build synthetic equity curve from the sampled trade returns.
            let (equity_curve, final_equity) =
                build_equity_curve(&sim_buf, initial_capital, position_size);

            let total_return = ((final_equity / initial_capital) - 1.0) * 100.0;
            let max_dd = compute_max_drawdown(&equity_curve);
            let sharpe = compute_sharpe(&equity_curve, periods_per_year);
            let pf = compute_profit_factor(&sim_buf);

            sim_returns.push(total_return);
            sim_drawdowns.push(max_dd);
            sim_sharpes.push(sharpe);
            sim_pfs.push(pf);
        }

        MonteCarloResult {
            num_simulations: self.num_simulations,
            method: self.method.clone(),
            total_return: PercentileStats::from_sorted(&mut sim_returns),
            max_drawdown: PercentileStats::from_sorted(&mut sim_drawdowns),
            sharpe_ratio: PercentileStats::from_sorted(&mut sim_sharpes),
            profit_factor: PercentileStats::from_sorted(&mut sim_pfs),
        }
    }
}

// ── Output types ──────────────────────────────────────────────────────────────

/// Percentile summary over the Monte Carlo simulations for a single metric.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PercentileStats {
    /// 5th percentile (worst-case tail)
    pub p5: f64,
    /// 25th percentile (lower quartile)
    pub p25: f64,
    /// 50th percentile (median)
    pub p50: f64,
    /// 75th percentile (upper quartile)
    pub p75: f64,
    /// 95th percentile (best-case tail)
    pub p95: f64,
    /// Mean across all simulations
    pub mean: f64,
}

impl PercentileStats {
    /// Compute percentile stats from a slice (sorts in place for efficiency).
    fn from_sorted(values: &mut [f64]) -> Self {
        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let n = values.len();

        let percentile = |p: f64| {
            let idx = ((p / 100.0) * (n - 1) as f64).round() as usize;
            values[idx.min(n - 1)]
        };

        let mean = values.iter().sum::<f64>() / n as f64;

        Self {
            p5: percentile(5.0),
            p25: percentile(25.0),
            p50: percentile(50.0),
            p75: percentile(75.0),
            p95: percentile(95.0),
            mean,
        }
    }
}

/// Results of the Monte Carlo simulation.
///
/// Each field gives the distribution of that metric across all simulations.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonteCarloResult {
    /// Number of simulations that were run
    pub num_simulations: usize,

    /// Resampling method used to generate the simulations
    pub method: MonteCarloMethod,

    /// Distribution of total return (%) across simulations
    pub total_return: PercentileStats,

    /// Distribution of maximum drawdown (0.0–1.0) across simulations
    pub max_drawdown: PercentileStats,

    /// Distribution of Sharpe ratio across simulations.
    ///
    /// **Interpretation note:** this Sharpe is computed from inter-trade returns
    /// (one data point per trade), not from bar-by-bar returns as in
    /// [`PerformanceMetrics::sharpe_ratio`]. Annualisation uses
    /// `sqrt(bars_per_year)`, which is only correct if trades occur every bar —
    /// an assumption that is rarely satisfied. Use this field to *rank
    /// simulations against each other*, not to compare against the
    /// `PerformanceMetrics` value.
    ///
    /// [`PerformanceMetrics::sharpe_ratio`]: super::result::PerformanceMetrics::sharpe_ratio
    pub sharpe_ratio: PercentileStats,

    /// Distribution of profit factor across simulations
    pub profit_factor: PercentileStats,
}

// ── PRNG: xorshift64 ──────────────────────────────────────────────────────────

/// Minimal xorshift64 PRNG — avoids adding a `rand` dependency.
///
/// `pub(crate)` so that other backtesting submodules (e.g. `bayesian_search`)
/// can reuse the same PRNG without duplicating the implementation.
pub(crate) struct Xorshift64 {
    state: u64,
}

impl Xorshift64 {
    pub(crate) fn new(seed: u64) -> Self {
        // Ensure state is never zero (xorshift requirement)
        Self {
            state: if seed == 0 { 1 } else { seed },
        }
    }

    /// Generate the next pseudo-random u64.
    pub(crate) fn next(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    /// Generate a random `usize` in `[0, n)`.
    ///
    /// Uses rejection sampling to eliminate modulo bias — otherwise the lower
    /// values of `[0, n)` would be slightly more probable when `n` is not a
    /// power of two (since `u64::MAX` is not evenly divisible by arbitrary `n`).
    pub(crate) fn next_usize(&mut self, n: usize) -> usize {
        let n64 = n as u64;
        // The largest multiple of n64 that fits in u64; values in [threshold, u64::MAX]
        // are rejected to ensure uniform distribution.
        let threshold = u64::MAX - (u64::MAX % n64);
        loop {
            let x = self.next();
            if x < threshold {
                return (x % n64) as usize;
            }
        }
    }

    /// Generate a uniform `f64` in `(0, 1]` using 53-bit precision.
    ///
    /// Returns a value strictly greater than zero, making it safe for use as
    /// the argument to `f64::ln()` in Box-Muller transforms.
    pub(crate) fn next_f64_positive(&mut self) -> f64 {
        // Take the top 53 bits (full double mantissa precision), add 1 to shift
        // from [0, 2^53) to [1, 2^53], then scale to (0, 1].
        ((self.next() >> 11) + 1) as f64 * (1.0 / (1u64 << 53) as f64)
    }
}

// ── Sampling helpers ──────────────────────────────────────────────────────────

/// Fisher-Yates in-place shuffle using the provided RNG.
fn fisher_yates_shuffle(slice: &mut [f64], rng: &mut Xorshift64) {
    for i in (1..slice.len()).rev() {
        let j = rng.next_usize(i + 1);
        slice.swap(i, j);
    }
}

/// Block bootstrap sampler — fixed block size, circular wrap-around.
///
/// Draws random starting positions and copies `block_size` consecutive
/// elements (wrapping around the end), filling `out` to exactly its length.
fn block_bootstrap_into(trades: &[f64], block_size: usize, rng: &mut Xorshift64, out: &mut [f64]) {
    let n = trades.len();
    let block_size = block_size.max(1);
    let mut filled = 0;
    while filled < n {
        let start = rng.next_usize(n);
        let take = block_size.min(n - filled);
        for i in 0..take {
            out[filled + i] = trades[(start + i) % n];
        }
        filled += take;
    }
}

/// Stationary bootstrap sampler — geometrically-distributed block lengths.
///
/// At each position, continues the current block with probability
/// `(mean_block_size - 1) / mean_block_size`, or jumps to a new random start
/// with probability `1 / mean_block_size`. Implemented without floating-point
/// division by testing `rng.next_usize(mean_block_size) == 0`.
fn stationary_bootstrap_into(
    trades: &[f64],
    mean_block_size: usize,
    rng: &mut Xorshift64,
    out: &mut [f64],
) {
    let n = trades.len();
    let mean_block_size = mean_block_size.max(1);
    let mut pos = rng.next_usize(n);
    for slot in out.iter_mut() {
        *slot = trades[pos % n];
        if rng.next_usize(mean_block_size) == 0 {
            // Start a new block at a random position.
            pos = rng.next_usize(n);
        } else {
            pos += 1;
        }
    }
}

/// Parametric sampler — draws from N(μ, σ) fitted to `trades`.
///
/// Uses the Box-Muller transform to convert pairs of uniform draws into
/// standard-normal samples, then shifts and scales by the empirical mean and
/// standard deviation. When fewer than 2 trades exist (σ undefined), all
/// samples are set to the empirical mean.
fn parametric_sample_into(trades: &[f64], rng: &mut Xorshift64, out: &mut [f64]) {
    let n = trades.len();
    let mean = trades.iter().sum::<f64>() / n as f64;
    let variance = if n > 1 {
        trades.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (n as f64 - 1.0)
    } else {
        0.0
    };
    let std_dev = variance.sqrt();

    // Nothing to sample if variance is zero.
    if std_dev == 0.0 {
        out.iter_mut().for_each(|v| *v = mean);
        return;
    }

    let mut i = 0;
    while i < n {
        // Box-Muller: two uniform (0,1] draws → two independent standard-normal samples.
        let u1 = rng.next_f64_positive();
        let u2 = rng.next_f64_positive();
        let mag = (-2.0 * u1.ln()).sqrt();
        let angle = std::f64::consts::TAU * u2;
        let z0 = mag * angle.cos();
        let z1 = mag * angle.sin();

        out[i] = mean + std_dev * z0;
        if i + 1 < n {
            out[i + 1] = mean + std_dev * z1;
        }
        i += 2;
    }
}

// ── Equity-curve helpers ──────────────────────────────────────────────────────

/// Build a per-trade equity curve from a sequence of trade returns.
///
/// Returns `(equity_points, final_equity)`. Each point represents the
/// portfolio value after applying one trade's return to the previous equity.
fn build_equity_curve(
    trade_returns: &[f64],
    initial_capital: f64,
    position_size_pct: f64,
) -> (Vec<f64>, f64) {
    let mut curve = Vec::with_capacity(trade_returns.len() + 1);
    curve.push(initial_capital);
    let mut equity = initial_capital;
    let exposure = position_size_pct.max(0.0);
    for &ret in trade_returns {
        equity *= 1.0 + ret * exposure;
        curve.push(equity);
    }
    (curve, equity)
}

/// Compute maximum drawdown (fraction, 0.0–1.0) from an equity curve.
fn compute_max_drawdown(equity_curve: &[f64]) -> f64 {
    let mut peak = f64::NEG_INFINITY;
    let mut max_dd = 0.0_f64;
    for &equity in equity_curve {
        if equity > peak {
            peak = equity;
        }
        if peak > 0.0 {
            let dd = (peak - equity) / peak;
            max_dd = max_dd.max(dd);
        }
    }
    max_dd
}

/// Compute a simplified Sharpe ratio from the Monte Carlo equity curve.
///
/// Uses bar-to-bar returns with no risk-free rate adjustment. Uses sample
/// standard deviation (n-1) and annualises by `sqrt(bars_per_year)`, consistent
/// with [`PerformanceMetrics`].
///
/// **Important**: the equity curve passed here is built from shuffled *trade*
/// returns (one equity point per trade), **not** from bar-by-bar values.  For
/// a backtest with N trades over M bars (N << M), the resulting Sharpe is
/// computed from N−1 inter-trade returns rather than the M−1 daily returns used
/// by `PerformanceMetrics`.  This produces a different annualisation baseline
/// and should be interpreted as a *relative* metric for comparing simulations
/// against each other rather than compared directly to the bar-by-bar Sharpe in
/// the original `BacktestResult`.
///
/// [`PerformanceMetrics`]: super::result::PerformanceMetrics
fn compute_sharpe(equity_curve: &[f64], periods_per_year: f64) -> f64 {
    if equity_curve.len() < 2 {
        return 0.0;
    }
    let returns: Vec<f64> = equity_curve
        .windows(2)
        .map(|w| {
            if w[0] > 0.0 {
                (w[1] - w[0]) / w[0]
            } else {
                0.0
            }
        })
        .collect();

    if returns.len() < 2 {
        return 0.0;
    }
    let n = returns.len() as f64;
    let mean = returns.iter().sum::<f64>() / n;
    // Sample variance (n-1)
    let variance = returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (n - 1.0);
    let std_dev = variance.sqrt();
    if std_dev == 0.0 {
        return 0.0;
    }
    (mean / std_dev) * periods_per_year.sqrt()
}

/// Compute profit factor from a sequence of trade return fractions.
fn compute_profit_factor(trade_returns: &[f64]) -> f64 {
    let gross_profit: f64 = trade_returns.iter().filter(|&&r| r > 0.0).sum();
    let gross_loss: f64 = trade_returns
        .iter()
        .filter(|&&r| r < 0.0)
        .map(|r| r.abs())
        .sum();
    if gross_loss > 0.0 {
        gross_profit / gross_loss
    } else if gross_profit > 0.0 {
        f64::MAX
    } else {
        0.0
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backtesting::result::{BacktestResult, EquityPoint, PerformanceMetrics};
    use crate::backtesting::signal::Signal;
    use crate::backtesting::{BacktestConfig, PositionSide, Trade};

    fn make_signal() -> Signal {
        Signal::long(0, 100.0)
    }

    fn make_trade(entry: f64, exit: f64, qty: f64) -> Trade {
        Trade {
            side: PositionSide::Long,
            entry_timestamp: 0,
            exit_timestamp: 86400,
            entry_price: entry,
            exit_price: exit,
            quantity: qty,
            entry_quantity: qty,
            commission: 0.0,
            transaction_tax: 0.0,
            pnl: (exit - entry) * qty,
            return_pct: ((exit / entry) - 1.0) * 100.0,
            dividend_income: 0.0,
            unreinvested_dividends: 0.0,
            tags: Vec::new(),
            is_partial: false,
            scale_sequence: 0,
            entry_signal: make_signal(),
            exit_signal: Signal::exit(86400, exit),
        }
    }

    fn make_equity_point(ts: i64, equity: f64) -> EquityPoint {
        EquityPoint {
            timestamp: ts,
            equity,
            drawdown_pct: 0.0,
        }
    }

    fn minimal_result(trades: Vec<Trade>) -> BacktestResult {
        let n_candles = 252;
        let equity_curve: Vec<EquityPoint> = (0..n_candles)
            .map(|i| make_equity_point(i as i64, 10_000.0))
            .collect();

        BacktestResult {
            symbol: "TEST".into(),
            strategy_name: "test".into(),
            config: BacktestConfig::default(),
            start_timestamp: 0,
            end_timestamp: n_candles as i64,
            initial_capital: 10_000.0,
            final_equity: 10_000.0,
            metrics: PerformanceMetrics::calculate(
                &trades,
                &equity_curve,
                10_000.0,
                0,
                0,
                0.0,
                252.0,
            ),
            trades,
            equity_curve,
            signals: vec![],
            open_position: None,
            benchmark: None,
            diagnostics: vec![],
        }
    }

    fn mixed_trades() -> Vec<Trade> {
        vec![
            make_trade(100.0, 110.0, 10.0),
            make_trade(100.0, 90.0, 10.0),
            make_trade(100.0, 115.0, 10.0),
            make_trade(100.0, 95.0, 10.0),
        ]
    }

    // ── IidShuffle ──────────────────────────────────────────────────────────

    #[test]
    fn test_reproducible_with_seed() {
        let result = minimal_result(mixed_trades());
        let config = MonteCarloConfig::default().seed(42);
        let mc1 = config.run(&result);
        let mc2 = config.run(&result);

        assert!((mc1.total_return.p50 - mc2.total_return.p50).abs() < f64::EPSILON);
        assert!((mc1.max_drawdown.p50 - mc2.max_drawdown.p50).abs() < f64::EPSILON);
    }

    #[test]
    fn test_percentile_ordering() {
        let trades = vec![
            make_trade(100.0, 120.0, 10.0),
            make_trade(100.0, 80.0, 10.0),
            make_trade(100.0, 130.0, 10.0),
            make_trade(100.0, 75.0, 10.0),
            make_trade(100.0, 110.0, 10.0),
        ];
        let result = minimal_result(trades);
        let mc = MonteCarloConfig::default()
            .num_simulations(500)
            .seed(1)
            .run(&result);

        assert!(mc.total_return.p5 <= mc.total_return.p25);
        assert!(mc.total_return.p25 <= mc.total_return.p50);
        assert!(mc.total_return.p50 <= mc.total_return.p75);
        assert!(mc.total_return.p75 <= mc.total_return.p95);
        assert!(mc.max_drawdown.p5 <= mc.max_drawdown.p95);
    }

    #[test]
    fn test_degenerate_single_trade() {
        let result = minimal_result(vec![make_trade(100.0, 110.0, 10.0)]);
        let mc = MonteCarloConfig::default().run(&result);

        // With only 1 trade there's nothing to resample — all percentiles equal observed
        assert_eq!(mc.total_return.p5, mc.total_return.p50);
        assert_eq!(mc.total_return.p50, mc.total_return.p95);
    }

    #[test]
    fn test_all_winning_trades_tight_distribution() {
        let trades: Vec<Trade> = (0..20).map(|_| make_trade(100.0, 110.0, 10.0)).collect();
        let result = minimal_result(trades);
        let mc = MonteCarloConfig::default().seed(99).run(&result);

        let spread = mc.total_return.p95 - mc.total_return.p5;
        assert!(
            spread < 1e-6,
            "expected tight spread for identical trades, got {spread}"
        );
    }

    // ── BlockBootstrap ──────────────────────────────────────────────────────

    #[test]
    fn test_block_bootstrap_percentile_ordering() {
        let trades = vec![
            make_trade(100.0, 120.0, 10.0),
            make_trade(100.0, 80.0, 10.0),
            make_trade(100.0, 130.0, 10.0),
            make_trade(100.0, 75.0, 10.0),
            make_trade(100.0, 110.0, 10.0),
            make_trade(100.0, 95.0, 10.0),
        ];
        let result = minimal_result(trades);
        let mc = MonteCarloConfig::default()
            .method(MonteCarloMethod::BlockBootstrap { block_size: 2 })
            .num_simulations(500)
            .seed(7)
            .run(&result);

        assert!(mc.total_return.p5 <= mc.total_return.p50);
        assert!(mc.total_return.p50 <= mc.total_return.p95);
    }

    #[test]
    fn test_block_bootstrap_reproducible() {
        let result = minimal_result(mixed_trades());
        let config = MonteCarloConfig::default()
            .method(MonteCarloMethod::BlockBootstrap { block_size: 2 })
            .seed(13);
        let mc1 = config.run(&result);
        let mc2 = config.run(&result);

        assert!((mc1.total_return.p50 - mc2.total_return.p50).abs() < f64::EPSILON);
    }

    #[test]
    fn test_block_bootstrap_block_size_one_matches_iid_distribution() {
        // block_size=1 is equivalent to IID shuffle in terms of return distribution
        // (same set of individual values, different orderings). Both should give
        // the same set of possible total returns.
        let trades: Vec<Trade> = (0..10).map(|_| make_trade(100.0, 110.0, 10.0)).collect();
        let result = minimal_result(trades);

        let iid = MonteCarloConfig::default().seed(1).run(&result);
        let bb = MonteCarloConfig::default()
            .method(MonteCarloMethod::BlockBootstrap { block_size: 1 })
            .seed(1)
            .run(&result);

        // All identical trades → both should give the same tight distribution
        let iid_spread = iid.total_return.p95 - iid.total_return.p5;
        let bb_spread = bb.total_return.p95 - bb.total_return.p5;
        assert!(iid_spread < 1e-6, "iid spread {iid_spread}");
        assert!(bb_spread < 1e-6, "bb spread {bb_spread}");
    }

    // ── StationaryBootstrap ─────────────────────────────────────────────────

    #[test]
    fn test_stationary_bootstrap_percentile_ordering() {
        let trades = vec![
            make_trade(100.0, 120.0, 10.0),
            make_trade(100.0, 80.0, 10.0),
            make_trade(100.0, 130.0, 10.0),
            make_trade(100.0, 75.0, 10.0),
            make_trade(100.0, 110.0, 10.0),
            make_trade(100.0, 95.0, 10.0),
        ];
        let result = minimal_result(trades);
        let mc = MonteCarloConfig::default()
            .method(MonteCarloMethod::StationaryBootstrap { mean_block_size: 2 })
            .num_simulations(500)
            .seed(5)
            .run(&result);

        assert!(mc.total_return.p5 <= mc.total_return.p50);
        assert!(mc.total_return.p50 <= mc.total_return.p95);
    }

    #[test]
    fn test_stationary_bootstrap_reproducible() {
        let result = minimal_result(mixed_trades());
        let config = MonteCarloConfig::default()
            .method(MonteCarloMethod::StationaryBootstrap { mean_block_size: 2 })
            .seed(77);
        let mc1 = config.run(&result);
        let mc2 = config.run(&result);

        assert!((mc1.total_return.p50 - mc2.total_return.p50).abs() < f64::EPSILON);
    }

    // ── Parametric ──────────────────────────────────────────────────────────

    #[test]
    fn test_parametric_percentile_ordering() {
        let trades = vec![
            make_trade(100.0, 120.0, 10.0),
            make_trade(100.0, 80.0, 10.0),
            make_trade(100.0, 130.0, 10.0),
            make_trade(100.0, 75.0, 10.0),
            make_trade(100.0, 110.0, 10.0),
        ];
        let result = minimal_result(trades);
        let mc = MonteCarloConfig::default()
            .method(MonteCarloMethod::Parametric)
            .num_simulations(1000)
            .seed(3)
            .run(&result);

        assert!(mc.total_return.p5 <= mc.total_return.p25);
        assert!(mc.total_return.p25 <= mc.total_return.p50);
        assert!(mc.total_return.p50 <= mc.total_return.p75);
        assert!(mc.total_return.p75 <= mc.total_return.p95);
    }

    #[test]
    fn test_parametric_reproducible() {
        let result = minimal_result(mixed_trades());
        let config = MonteCarloConfig::default()
            .method(MonteCarloMethod::Parametric)
            .seed(99);
        let mc1 = config.run(&result);
        let mc2 = config.run(&result);

        assert!((mc1.total_return.p50 - mc2.total_return.p50).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parametric_identical_trades_tight_distribution() {
        // σ = 0 → all samples are the mean → tight distribution
        let trades: Vec<Trade> = (0..10).map(|_| make_trade(100.0, 110.0, 10.0)).collect();
        let result = minimal_result(trades);
        let mc = MonteCarloConfig::default()
            .method(MonteCarloMethod::Parametric)
            .seed(1)
            .run(&result);

        let spread = mc.total_return.p95 - mc.total_return.p5;
        assert!(
            spread < 1e-6,
            "expected tight spread for zero-variance trades, got {spread}"
        );
    }

    // ── PRNG ────────────────────────────────────────────────────────────────

    #[test]
    fn test_xorshift_never_zero() {
        let mut rng = Xorshift64::new(0); // seed 0 → should become 1 internally
        for _ in 0..1000 {
            assert_ne!(rng.next(), 0);
        }
    }

    #[test]
    fn test_next_f64_positive_in_range() {
        let mut rng = Xorshift64::new(42);
        for _ in 0..10_000 {
            let v = rng.next_f64_positive();
            assert!(v > 0.0 && v <= 1.0, "out of range: {v}");
        }
    }

    // ── Helpers ─────────────────────────────────────────────────────────────

    #[test]
    fn test_profit_factor_all_wins_is_f64_max() {
        let pf = compute_profit_factor(&[0.01, 0.02, 0.03]);
        assert_eq!(pf, f64::MAX);
    }

    #[test]
    fn test_result_carries_method() {
        let result = minimal_result(mixed_trades());
        let mc = MonteCarloConfig::default()
            .method(MonteCarloMethod::BlockBootstrap { block_size: 3 })
            .run(&result);
        assert!(matches!(
            mc.method,
            MonteCarloMethod::BlockBootstrap { block_size: 3 }
        ));
    }
}
