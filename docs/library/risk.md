# Risk Analytics

!!! abstract "Cargo Docs"
    [docs.rs/finance-query — risk](https://docs.rs/finance-query/latest/finance_query/risk/index.html)

!!! info "Feature flag required"
    The `risk` feature implies `indicators`.
    ```toml
    finance-query = { version = "...", features = ["risk"] }
    ```

The `risk` module computes portfolio risk metrics from historical price data: Value at Risk, Sharpe/Sortino/Calmar ratios, beta, and maximum drawdown. All metrics are available through the `Ticker` API or as standalone functions.

This page is a **living document**: the code blocks are compiled (and, where
they need no network, run) as generated tests via `cargo soothfast docs
gen-tests`, the field table below is bound to the source with an
`soothfast:bind` marker (CI fails if `RiskSummary` changes under it), and the
performance statements carry `soothfast:claim` markers checked against real
measurements on every CI run.

## Via Ticker

```rust no_run feature=risk
use finance_query::{Interval, Ticker, TimeRange};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ticker = Ticker::new("AAPL").await?;

    // Risk summary over the past year vs S&P 500 as benchmark
    let summary = ticker.risk(Interval::OneDay, TimeRange::OneYear, Some("^GSPC")).await?;

    // Without a benchmark (beta will be None)
    let summary = ticker.risk(Interval::OneDay, TimeRange::OneYear, None).await?;
    Ok(())
}
```

## `RiskSummary` Fields

<!-- soothfast:bind finance_query::risk::RiskSummary -->

| Field | Type | Description |
|-------|------|-------------|
| `var_95` | `f64` | 1-day historical VaR at 95% confidence (positive loss fraction) |
| `var_99` | `f64` | 1-day historical VaR at 99% confidence |
| `parametric_var_95` | `f64` | 1-day parametric VaR at 95% (assumes normal distribution) |
| `cvar_95` | `f64` | 1-day historical Conditional VaR (Expected Shortfall) at 95% — average loss in the worst 5% of periods |
| `cvar_99` | `f64` | 1-day historical Conditional VaR at 99% |
| `parametric_cvar_95` | `f64` | 1-day parametric Conditional VaR at 95% (assumes normal distribution) |
| `omega` | `f64` | Omega ratio at a 0.0 threshold; `f64::MAX` when there are no losing periods |
| `kelly` | `f64` | Kelly Criterion — optimal fraction of capital to risk; `f64::MAX` on an unbounded edge |
| `sharpe` | `Option<f64>` | Annualised Sharpe ratio (risk-free rate = 0, 252 days/year); `None` if fewer than 2 periods or zero volatility |
| `sortino` | `Option<f64>` | Annualised Sortino ratio (penalises downside only); `None` if insufficient data |
| `calmar` | `Option<f64>` | Calmar ratio (annualised return / max drawdown); `None` if drawdown is zero |
| `beta` | `Option<f64>` | Beta vs benchmark; `None` if no benchmark provided or insufficient data |
| `max_drawdown` | `f64` | Maximum drawdown as a positive fraction (e.g., `0.30` = 30%) |
| `max_drawdown_recovery_periods` | `Option<u64>` | Trading periods to recover from max drawdown; `None` if no recovery in window |
| `ulcer_index` | `f64` | Ulcer Index — RMS drawdown depth as a percentage (0–100), penalising depth and duration |
| `information_ratio` | `Option<f64>` | Information ratio vs benchmark; `None` without a benchmark or with insufficient data |
| `tracking_error` | `Option<f64>` | Annualised tracking error vs benchmark; `None` without a benchmark or with insufficient data |

<!-- /soothfast:bind -->

## Example: Full Risk Report

```rust no_run feature=risk covers=finance_query::risk::RiskSummary
use finance_query::{Interval, Ticker, TimeRange};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ticker = Ticker::new("NVDA").await?;
    let risk = ticker.risk(Interval::OneDay, TimeRange::TwoYears, Some("^GSPC")).await?;

    println!("=== Risk Report: NVDA (2Y daily) ===");
    println!("VaR 95%:       {:.2}%", risk.var_95 * 100.0);
    println!("VaR 99%:       {:.2}%", risk.var_99 * 100.0);
    println!("Param VaR 95%: {:.2}%", risk.parametric_var_95 * 100.0);
    println!("Max Drawdown:  {:.2}%", risk.max_drawdown * 100.0);

    if let Some(periods) = risk.max_drawdown_recovery_periods {
        println!("Recovery:      {} trading days", periods);
    } else {
        println!("Recovery:      no full recovery in window");
    }

    if let Some(sharpe) = risk.sharpe {
        println!("Sharpe:        {:.2}", sharpe);
    }
    if let Some(sortino) = risk.sortino {
        println!("Sortino:       {:.2}", sortino);
    }
    if let Some(calmar) = risk.calmar {
        println!("Calmar:        {:.2}", calmar);
    }
    if let Some(beta) = risk.beta {
        println!("Beta (vs SPX): {:.2}", beta);
    }
    Ok(())
}
```

## Standalone Functions

The individual metric functions are available in `finance_query::risk` for direct use on raw return series. This example runs as a real test — no network needed:

```rust capture-output feature=risk covers=finance_query::risk_sharpe,finance_query::risk_sortino,finance_query::risk_parametric_var,finance_query::risk_beta,finance_query::risk_historical_var
use finance_query::risk::{
    beta, calmar_ratio, historical_var, max_drawdown, parametric_var, sharpe_ratio,
    sortino_ratio,
};

// Close-to-close simple returns; real code would derive these from chart data.
let returns: Vec<f64> = (0..252).map(|i| ((i % 7) as f64 - 3.0) / 100.0).collect();
let benchmark_returns: Vec<f64> = (0..252).map(|i| ((i % 5) as f64 - 2.0) / 100.0).collect();

// Value at Risk (positive loss fractions)
let var_95 = historical_var(&returns, 0.95).unwrap();
let var_99 = historical_var(&returns, 0.99).unwrap();
let pvar = parametric_var(&returns, 0.95).unwrap();
assert!(var_95 > 0.0 && var_99 >= var_95);

// Risk-adjusted returns (annualised, rf=0, 252 trading days). `None` only
// for a degenerate all-zero-variance series, so it's safe to unwrap here.
let sharpe = sharpe_ratio(&returns, 0.0, 252.0).unwrap();
let sortino = sortino_ratio(&returns, 0.0, 252.0).unwrap();

// Max drawdown & Calmar
let dd = max_drawdown(&returns);
// dd.max_drawdown: f64 (positive fraction)
// dd.recovery_periods: Option<u64>
let annualised_return = 0.25; // your computed value
let years = returns.len() as f64 / 252.0;
let calmar = calmar_ratio(annualised_return, years, dd.max_drawdown).unwrap();

// Beta
let b = beta(&returns, &benchmark_returns).unwrap();

println!("VaR 95%/99%: {var_95:.4} / {var_99:.4}   parametric VaR 95%: {pvar:.4}");
println!("Sharpe: {sharpe:.4}   Sortino: {sortino:.4}   Calmar: {calmar:.4}");
println!("Max drawdown: {:.4}   Beta: {b:.4}", dd.max_drawdown);
```

```text soothfast-output
VaR 95%/99%: 0.0300 / 0.0300   parametric VaR 95%: 0.0330
Sharpe: -0.0000   Sortino: -0.0000   Calmar: 2.4056
Max drawdown: 0.1039   Beta: 0.0059
```

<!-- soothfast:claim finance_query::risk_sharpe.alloc.allocs <= 0 -->
<!-- soothfast:claim finance_query::risk_sortino.alloc.allocs <= 0 -->
<!-- soothfast:claim finance_query::risk_parametric_var.alloc.allocs <= 0 -->
<!-- soothfast:claim finance_query::risk_beta.alloc.allocs <= 0 -->
<!-- soothfast:claim finance_query::risk_beta.walltime.median_ns < 500000 -->
`sharpe_ratio`, `sortino_ratio`, `parametric_var`, and `beta` fold over the
input slices without copying — zero allocations, and `beta` completes in
well under half a millisecond on commodity hardware.

<!-- soothfast:claim finance_query::risk_historical_var.alloc.allocs <= 2 -->
`historical_var` makes exactly two allocations regardless of input size (the
sorted working copy and the stable sort's scratch buffer), and its runtime
grows as O(n log n) — verified by a measured size sweep from 1,000 to
100,000 returns.

## Metric Definitions

**Value at Risk (VaR)** — the maximum expected loss over one trading day at the given confidence level.
- *Historical VaR*: computed from the empirical return distribution (no distributional assumption).
- *Parametric VaR*: assumes normally distributed returns; uses mean and standard deviation.

<!-- soothfast:bind finance_query::risk::ratios::sharpe_ratio -->
**Sharpe Ratio** — `(mean_return - rf) / std_dev * sqrt(periods_per_year)`. Higher is better. Penalises all volatility equally.
<!-- /soothfast:bind -->

**Sortino Ratio** — like Sharpe but only penalises downside (negative) returns. More appropriate for skewed return distributions.

**Calmar Ratio** — `annualised_return / max_drawdown`. Measures return per unit of drawdown risk.

<!-- soothfast:bind finance_query::risk::beta::beta -->
**Beta** — covariance of returns with the benchmark divided by the benchmark variance. A beta > 1 indicates the asset amplifies market moves.
<!-- /soothfast:bind -->

**Maximum Drawdown** — the largest peak-to-trough decline in the return series, expressed as a positive fraction.

## Next Steps

- [Ticker API](ticker.md) - Full Ticker method reference including `risk()`
- [Backtesting](backtesting.md) - Strategy testing with built-in risk metrics
- [Indicators](indicators.md) - Technical indicators that inform risk assessment
