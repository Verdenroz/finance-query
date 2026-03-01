//! Monte Carlo simulation for backtesting results.
//!
//! Re-samples trade return sequences to estimate the distribution of outcomes.
//! Uses an embedded xorshift64 PRNG to avoid adding an external dependency.
//!
//! # Example
//!
//! ```ignore
//! use finance_query::backtesting::monte_carlo::{MonteCarloConfig, MonteCarloResult};
//!
//! // `result` is a completed BacktestResult
//! let mc = MonteCarloConfig::default().run(&result);
//! println!("Median return: {:.2}%", mc.total_return.p50);
//! println!("5th pct drawdown: {:.2}%", mc.max_drawdown.p5 * 100.0);
//! ```

use serde::{Deserialize, Serialize};

use super::result::BacktestResult;

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
}

impl Default for MonteCarloConfig {
    fn default() -> Self {
        Self {
            num_simulations: 1_000,
            seed: None,
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

    /// Run the Monte Carlo simulation against a completed backtest result.
    ///
    /// Extracts the trade returns, reshuffles them `num_simulations` times using
    /// Fisher-Yates, rebuilds a synthetic equity curve for each shuffle, and
    /// reports percentile statistics over the simulated outcomes.
    ///
    /// If the result has fewer than 2 trades, every percentile is derived from
    /// the single observed result.
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
                total_return: trivial(obs_return),
                max_drawdown: trivial(obs_dd),
                sharpe_ratio: trivial(obs_sharpe),
                profit_factor: trivial(obs_pf),
            };
        }

        let seed = self.seed.unwrap_or(12345);
        let mut rng = Xorshift64::new(seed);

        let mut sim_returns: Vec<f64> = Vec::with_capacity(self.num_simulations);
        let mut sim_drawdowns: Vec<f64> = Vec::with_capacity(self.num_simulations);
        let mut sim_sharpes: Vec<f64> = Vec::with_capacity(self.num_simulations);
        let mut sim_pfs: Vec<f64> = Vec::with_capacity(self.num_simulations);

        for _ in 0..self.num_simulations {
            let mut shuffled = trade_returns.clone();
            fisher_yates_shuffle(&mut shuffled, &mut rng);

            // Build synthetic equity curve from shuffled trade returns
            let (equity_curve, final_equity) = build_equity_curve(&shuffled, initial_capital);

            let total_return = ((final_equity / initial_capital) - 1.0) * 100.0;
            let max_dd = compute_max_drawdown(&equity_curve);
            let sharpe = compute_sharpe(&equity_curve, result.config.bars_per_year);
            let pf = compute_profit_factor(&shuffled);

            sim_returns.push(total_return);
            sim_drawdowns.push(max_dd);
            sim_sharpes.push(sharpe);
            sim_pfs.push(pf);
        }

        MonteCarloResult {
            num_simulations: self.num_simulations,
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
struct Xorshift64 {
    state: u64,
}

impl Xorshift64 {
    fn new(seed: u64) -> Self {
        // Ensure state is never zero (xorshift requirement)
        Self {
            state: if seed == 0 { 1 } else { seed },
        }
    }

    /// Generate the next pseudo-random u64.
    fn next(&mut self) -> u64 {
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
    fn next_usize(&mut self, n: usize) -> usize {
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
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Fisher-Yates in-place shuffle using the provided RNG.
fn fisher_yates_shuffle(slice: &mut [f64], rng: &mut Xorshift64) {
    for i in (1..slice.len()).rev() {
        let j = rng.next_usize(i + 1);
        slice.swap(i, j);
    }
}

/// Build a per-trade equity curve from a sequence of trade returns.
///
/// Returns `(equity_points, final_equity)`. Each point represents the
/// portfolio value after applying one trade's return to the previous equity.
fn build_equity_curve(trade_returns: &[f64], initial_capital: f64) -> (Vec<f64>, f64) {
    let mut curve = Vec::with_capacity(trade_returns.len() + 1);
    curve.push(initial_capital);
    let mut equity = initial_capital;
    for &ret in trade_returns {
        equity *= 1.0 + ret;
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
fn compute_sharpe(equity_curve: &[f64], bars_per_year: f64) -> f64 {
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
    (mean / std_dev) * bars_per_year.sqrt()
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
        f64::MAX // avoid INFINITY which cannot be serialized to JSON
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
            commission: 0.0,
            pnl: (exit - entry) * qty,
            return_pct: ((exit / entry) - 1.0) * 100.0,
            dividend_income: 0.0,
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

    #[test]
    fn test_reproducible_with_seed() {
        let trades = vec![
            make_trade(100.0, 110.0, 10.0),
            make_trade(100.0, 90.0, 10.0),
            make_trade(100.0, 115.0, 10.0),
            make_trade(100.0, 95.0, 10.0),
        ];
        let result = minimal_result(trades);

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

        // Percentiles must be ordered
        assert!(mc.total_return.p5 <= mc.total_return.p25);
        assert!(mc.total_return.p25 <= mc.total_return.p50);
        assert!(mc.total_return.p50 <= mc.total_return.p75);
        assert!(mc.total_return.p75 <= mc.total_return.p95);

        assert!(mc.max_drawdown.p5 <= mc.max_drawdown.p95);
    }

    #[test]
    fn test_degenerate_single_trade() {
        let trades = vec![make_trade(100.0, 110.0, 10.0)];
        let result = minimal_result(trades);

        let mc = MonteCarloConfig::default().run(&result);

        // With only 1 trade there's nothing to shuffle — all percentiles equal observed value
        assert_eq!(mc.total_return.p5, mc.total_return.p50);
        assert_eq!(mc.total_return.p50, mc.total_return.p95);
    }

    #[test]
    fn test_all_winning_trades_tight_distribution() {
        let trades: Vec<Trade> = (0..20).map(|_| make_trade(100.0, 110.0, 10.0)).collect();
        let result = minimal_result(trades);

        let mc = MonteCarloConfig::default().seed(99).run(&result);

        // All trades identical → reshuffling makes no difference → tight distribution
        let spread = mc.total_return.p95 - mc.total_return.p5;
        assert!(
            spread < 1e-6,
            "expected tight spread for identical trades, got {spread}"
        );
    }

    #[test]
    fn test_xorshift_never_zero() {
        let mut rng = Xorshift64::new(0); // seed 0 → should become 1 internally
        for _ in 0..1000 {
            assert_ne!(rng.next(), 0);
        }
    }
}
