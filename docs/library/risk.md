# Risk Analytics

!!! info "Feature flag required"
    The `risk` feature implies `indicators`.
    ```toml
    finance-query = { version = "...", features = ["risk"] }
    ```

The `risk` module computes portfolio risk metrics from historical price data: Value at Risk, Sharpe/Sortino/Calmar ratios, beta, and maximum drawdown. All metrics are available through the `Ticker` API or as standalone functions.

## Via Ticker

```rust
use finance_query::{Ticker, Interval, TimeRange};

let ticker = Ticker::new("AAPL").await?;

// Risk summary over the past year vs S&P 500 as benchmark
let summary = ticker.risk(Interval::OneDay, TimeRange::OneYear, Some("^GSPC")).await?;

// Without a benchmark (beta will be None)
let summary = ticker.risk(Interval::OneDay, TimeRange::OneYear, None).await?;
```

## `RiskSummary` Fields

| Field | Type | Description |
|-------|------|-------------|
| `var_95` | `f64` | 1-day historical VaR at 95% confidence (positive loss fraction) |
| `var_99` | `f64` | 1-day historical VaR at 99% confidence |
| `parametric_var_95` | `f64` | 1-day parametric VaR at 95% (assumes normal distribution) |
| `sharpe` | `Option<f64>` | Annualised Sharpe ratio (risk-free rate = 0, 252 days/year); `None` if fewer than 2 periods or zero volatility |
| `sortino` | `Option<f64>` | Annualised Sortino ratio (penalises downside only); `None` if insufficient data |
| `calmar` | `Option<f64>` | Calmar ratio (annualised return / max drawdown); `None` if drawdown is zero |
| `beta` | `Option<f64>` | Beta vs benchmark; `None` if no benchmark provided or insufficient data |
| `max_drawdown` | `f64` | Maximum drawdown as a positive fraction (e.g., `0.30` = 30%) |
| `max_drawdown_recovery_periods` | `Option<u64>` | Trading periods to recover from max drawdown; `None` if no recovery in window |

## Example: Full Risk Report

```rust
use finance_query::{Ticker, Interval, TimeRange};

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
```

## Standalone Functions

The individual metric functions are available in `finance_query::risk` for direct use on raw return series:

```rust
use finance_query::risk::{
    historical_var,
    parametric_var,
    sharpe_ratio,
    sortino_ratio,
    calmar_ratio,
    beta,
    max_drawdown,
};

// Prepare your own returns (close-to-close, e.g. [0.01, -0.02, ...])
let returns: Vec<f64> = vec![/* ... */];
let benchmark_returns: Vec<f64> = vec![/* ... */];

// Value at Risk
let var_95 = historical_var(&returns, 0.95)?;
let var_99 = historical_var(&returns, 0.99)?;
let pvar   = parametric_var(&returns, 0.95)?;

// Risk-adjusted returns (annualised, rf=0, 252 trading days)
let sharpe  = sharpe_ratio(&returns, 0.0, 252.0);
let sortino = sortino_ratio(&returns, 0.0, 252.0);

// Max drawdown & Calmar
let dd = max_drawdown(&returns);
// dd.max_drawdown: f64 (positive fraction)
// dd.recovery_periods: Option<u64>
let annualised_return = 0.25;  // your computed value
let years = returns.len() as f64 / 252.0;
let calmar = calmar_ratio(annualised_return, years, dd.max_drawdown);

// Beta
let b = beta(&returns, &benchmark_returns);
```

## Metric Definitions

**Value at Risk (VaR)** — the maximum expected loss over one trading day at the given confidence level.
- *Historical VaR*: computed from the empirical return distribution (no distributional assumption).
- *Parametric VaR*: assumes normally distributed returns; uses mean and standard deviation.

**Sharpe Ratio** — `(mean_return - rf) / std_dev * sqrt(periods_per_year)`. Higher is better. Penalises all volatility equally.

**Sortino Ratio** — like Sharpe but only penalises downside (negative) returns. More appropriate for skewed return distributions.

**Calmar Ratio** — `annualised_return / max_drawdown`. Measures return per unit of drawdown risk.

**Beta** — covariance of returns with the benchmark divided by the benchmark variance. A beta > 1 indicates the asset amplifies market moves.

**Maximum Drawdown** — the largest peak-to-trough decline in the return series, expressed as a positive fraction.

## Next Steps

- [Ticker API](ticker.md) - Full Ticker method reference including `risk()`
- [Backtesting](backtesting.md) - Strategy testing with built-in risk metrics
- [Indicators](indicators.md) - Technical indicators that inform risk assessment
