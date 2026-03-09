# Backtesting

!!! abstract "Cargo Docs"
    [docs.rs/finance-query — backtesting](https://docs.rs/finance-query/latest/finance_query/backtesting/index.html)

Test trading strategies against historical data. The backtesting engine provides pre-built strategies, a custom strategy builder, ensemble composition, parameter optimization, walk-forward validation, Monte Carlo simulation, and portfolio-level backtesting.

## Enable Feature

Backtesting requires the `backtesting` feature (which depends on `indicators`):

```toml
[dependencies]
finance-query = { version = "2.0", features = ["backtesting"] }
```

## Pre-built Strategies

### SMA Crossover

Dual Simple Moving Average crossover:

```rust
use finance_query::{Ticker, Interval, TimeRange};
use finance_query::backtesting::SmaCrossover;

let ticker = Ticker::new("AAPL").await?;
let result = ticker.backtest(
    SmaCrossover::new(10, 20),  // fast=10, slow=20
    Interval::OneDay,
    TimeRange::OneYear,
    None,
).await?;

println!("Total Return: {:.2}%", result.metrics.total_return_pct);
println!("Sharpe Ratio: {:.2}", result.metrics.sharpe_ratio);
println!("Max Drawdown: {:.2}%", result.metrics.max_drawdown_pct * 100.0);
```

### RSI Mean Reversion

Reversal strategy using Relative Strength Index:

```rust
use finance_query::backtesting::RsiReversal;

let result = ticker.backtest(
    RsiReversal::new(14),  // period (uses default thresholds: 30/70)
    Interval::OneDay,
    TimeRange::OneYear,
    None,
).await?;

// Or with custom thresholds:
let result = ticker.backtest(
    RsiReversal::new(14).with_thresholds(30.0, 70.0),
    Interval::OneDay,
    TimeRange::OneYear,
    None,
).await?;
```

### MACD Signal Crossover

MACD line crosses signal line:

```rust
use finance_query::backtesting::MacdSignal;

let result = ticker.backtest(
    MacdSignal::new(12, 26, 9),  // fast, slow, signal
    Interval::OneDay,
    TimeRange::OneYear,
    None,
).await?;
```

### Bollinger Band Mean Reversion

Buy at lower band, sell at upper band:

```rust
use finance_query::backtesting::BollingerMeanReversion;

let result = ticker.backtest(
    BollingerMeanReversion::new(20, 2.0),  // period, std_dev
    Interval::OneDay,
    TimeRange::OneYear,
    None,
).await?;
```

### SuperTrend Trend Following

Follow trends using ATR-based SuperTrend:

```rust
use finance_query::backtesting::SuperTrendFollow;

let result = ticker.backtest(
    SuperTrendFollow::new(10, 3.0),  // period, multiplier
    Interval::OneDay,
    TimeRange::OneYear,
    None,
).await?;
```

### Donchian Breakout

Channel breakout strategy:

```rust
use finance_query::backtesting::DonchianBreakout;

let result = ticker.backtest(
    DonchianBreakout::new(20),  // lookback period
    Interval::OneDay,
    TimeRange::OneYear,
    None,
).await?;
```

## Custom Strategies

Build custom strategies with `StrategyBuilder`. Entry conditions are combined with AND; exit conditions with OR (any exit triggers):

```rust
use finance_query::backtesting::StrategyBuilder;
use finance_query::backtesting::refs::*;
use finance_query::backtesting::condition::*;

let strategy = StrategyBuilder::new("RSI Mean Reversion")
    .entry(
        rsi(14)
            .crosses_below(30.0)
            .and(price().above_ref(sma(200)))
    )
    .exit(
        rsi(14)
            .crosses_above(70.0)
            .or(stop_loss(0.05))
    )
    .build();

let result = ticker.backtest(
    strategy,
    Interval::OneDay,
    TimeRange::OneYear,
    None,
).await?;
```

### Regime Filter

Suppress entry signals unless a regime condition passes (e.g., only trade in uptrends):

```rust
let strategy = StrategyBuilder::new("Trend-Filtered RSI")
    .entry(rsi(14).crosses_below(30.0))
    .exit(rsi(14).crosses_above(70.0))
    .regime_filter(price().above_ref(sma(200)))  // only enter if price > SMA(200)
    .build();
```

### Separate Short Leg

Define independent entry/exit conditions for short positions:

```rust
let strategy = StrategyBuilder::new("Long-Short RSI")
    .entry(rsi(14).crosses_below(30.0))           // long entry
    .exit(rsi(14).crosses_above(70.0))            // long exit
    .with_short(
        rsi(14).crosses_above(70.0),              // short entry
        rsi(14).crosses_below(30.0),              // short exit
    )
    .build();
```

### Warmup Period

Skip the first N bars before generating signals (useful when indicators need time to stabilize):

```rust
let strategy = StrategyBuilder::new("SMA with Warmup")
    .entry(price().crosses_above_ref(sma(200)))
    .exit(price().crosses_below_ref(sma(200)))
    .warmup(200)  // skip first 200 bars
    .build();
```

## Configuration

Customize backtesting behavior with `BacktestConfig`:

```rust
use finance_query::backtesting::BacktestConfig;

let config = BacktestConfig::builder()
    .initial_capital(50_000.0)
    .commission_pct(0.001)          // 0.1% per trade
    .commission(1.0)                // $1 flat fee per trade
    .slippage_pct(0.0005)           // 0.05% slippage
    .spread_pct(0.0002)             // 0.02% bid-ask spread (half each side)
    .transaction_tax_pct(0.005)     // 0.5% stamp duty on buys
    .stop_loss_pct(0.05)            // 5% global stop-loss
    .take_profit_pct(0.15)          // 15% global take-profit
    .trailing_stop_pct(0.03)        // 3% trailing stop
    .allow_short(true)
    .position_size_pct(0.5)         // use 50% of capital per trade
    .max_positions(3)               // at most 3 concurrent positions
    .bars_per_year(252.0)           // for annualized metric calculations
    .risk_free_rate(0.04)           // 4% annual risk-free rate
    .reinvest_dividends(true)
    .close_at_end(true)             // close open positions at final bar
    .build()?;

let result = ticker.backtest(
    SmaCrossover::new(10, 20),
    Interval::OneDay,
    TimeRange::OneYear,
    Some(config),
).await?;
```

### Zero-Cost Config

Convenience constructor with all friction zeroed — useful for theoretical comparisons:

```rust
let config = BacktestConfig::zero_cost();
```

### Custom Commission Function

Replace flat + percentage commission with a custom function:

```rust
let config = BacktestConfig::builder()
    .initial_capital(10_000.0)
    .commission_fn(|size, price| {
        // Example: tiered commission
        let value = size * price;
        if value < 1_000.0 { 1.0 } else { value * 0.0005 }
    })
    .build()?;
```

## Performance Metrics

Access the full set of performance metrics from `result.metrics`:

```rust
let result = ticker.backtest(SmaCrossover::new(10, 20), Interval::OneDay, TimeRange::OneYear, None).await?;

// Returns
println!("Total Return:      {:.2}%", result.metrics.total_return_pct);
println!("Annualized Return: {:.2}%", result.metrics.annualized_return_pct);
println!("Final Equity:      ${:.2}", result.final_equity);

// Risk-adjusted
println!("Sharpe Ratio:    {:.2}", result.metrics.sharpe_ratio);
println!("Sortino Ratio:   {:.2}", result.metrics.sortino_ratio);
println!("Calmar Ratio:    {:.2}", result.metrics.calmar_ratio);
println!("Max Drawdown:    {:.2}%", result.metrics.max_drawdown_pct * 100.0);

// Trade statistics
println!("Total Trades:    {}", result.metrics.total_trades);
println!("Winning Trades:  {}", result.metrics.winning_trades);
println!("Losing Trades:   {}", result.metrics.losing_trades);
println!("Win Rate:        {:.2}%", result.metrics.win_rate * 100.0);
println!("Profit Factor:   {:.2}", result.metrics.profit_factor);
println!("Avg Trade:       {:.2}%", result.metrics.avg_trade_return_pct);
println!("Avg Win:         {:.2}%", result.metrics.avg_win_pct);
println!("Avg Loss:        {:.2}%", result.metrics.avg_loss_pct);
println!("Largest Win:     {:.2}%", result.metrics.largest_win);
println!("Largest Loss:    {:.2}%", result.metrics.largest_loss);
println!("Max Consec. Wins:   {}", result.metrics.max_consecutive_wins);
println!("Max Consec. Losses: {}", result.metrics.max_consecutive_losses);

// Position breakdown
println!("Long Trades:  {}", result.metrics.long_trades);
println!("Short Trades: {}", result.metrics.short_trades);
println!("Time in Market: {:.1}%", result.metrics.time_in_market_pct * 100.0);

// Signal execution
println!("Total Signals:    {}", result.metrics.total_signals);
println!("Executed Signals: {}", result.metrics.executed_signals);
println!("Total Commission: ${:.2}", result.metrics.total_commission);
println!("Dividend Income:  ${:.2}", result.metrics.total_dividend_income);

// Advanced statistics
println!("Kelly Criterion: {:.2}", result.metrics.kelly_criterion);
println!("SQN:             {:.2}", result.metrics.sqn);
println!("Expectancy:      {:.2}", result.metrics.expectancy);
println!("Omega Ratio:     {:.2}", result.metrics.omega_ratio);
println!("Tail Ratio:      {:.2}", result.metrics.tail_ratio);
println!("Recovery Factor: {:.2}", result.metrics.recovery_factor);
println!("Ulcer Index:     {:.2}", result.metrics.ulcer_index);
println!("Serenity Ratio:  {:.2}", result.metrics.serenity_ratio);
```

## Advanced Result Analysis

### Rolling Analytics

```rust
let sharpe_30 = result.rolling_sharpe(30);      // rolling 30-bar Sharpe ratio
let drawdowns  = result.drawdown_series();       // drawdown at each equity point
let win_rate_20 = result.rolling_win_rate(20);  // rolling 20-trade win rate
```

### Temporal Breakdown

```rust
// Break performance down by calendar period
let by_year  = result.by_year();         // HashMap<i32, PerformanceMetrics>
let by_month = result.by_month();        // HashMap<(i32, u32), PerformanceMetrics>
let by_dow   = result.by_day_of_week();  // HashMap<Weekday, PerformanceMetrics>

for (year, metrics) in &by_year {
    println!("{year}: {:.2}%", metrics.total_return_pct);
}
```

### Tag-Based Filtering

Tag signals and trades to analyze subsets of your strategy:

```rust
let tagged_trades  = result.trades_by_tag("breakout");
let tagged_metrics = result.metrics_by_tag("breakout");
let all_tags       = result.all_tags();
```

### Diagnostics

Engine warnings and notes (e.g., skipped bars, insufficient capital):

```rust
for msg in &result.diagnostics {
    println!("⚠ {msg}");
}
```

## Order Types

By default all signals fill at market. Use limit, stop, and stop-limit orders for more realistic fills:

```rust
use finance_query::backtesting::Signal;

let ts = 0i64;    // use actual candle timestamp in practice
let px = 150.0;   // use actual candle close in practice

// Market (default)
let market_entry = Signal::long(ts, px);

// Buy limit — fill only if price reaches limit_price (below current)
let limit_entry  = Signal::buy_limit(ts, px, 148.0);

// Buy stop — fill when price breaks above stop_price
let stop_entry   = Signal::buy_stop(ts, px, 152.0);

// Buy stop-limit — trigger at stop, fill at limit or better
let stop_limit   = Signal::buy_stop_limit(ts, px, 152.0, 153.0);

// Sell limit / stop for exits
let limit_exit   = Signal::sell_limit(ts, px, 160.0);
let stop_exit    = Signal::sell_stop(ts, px, 145.0);
```

### Order Expiry

Pending orders that haven't filled cancel after N bars:

```rust
let signal = Signal::buy_limit(ts, px, 148.0)
    .expires_in_bars(5);  // cancel if not filled within 5 bars
```

### Per-Trade Bracket Orders

Override global stop-loss / take-profit / trailing-stop on a per-signal basis:

```rust
let signal = Signal::long(ts, px)
    .stop_loss(0.03)       // 3% stop for this trade
    .take_profit(0.10)     // 10% take-profit for this trade
    .trailing_stop(0.02);  // 2% trailing stop for this trade
```

### Scale In / Out

Add to or partially exit an existing position:

```rust
let add_to_position    = Signal::scale_in(0.25, ts, px);   // add 25% of position size
let reduce_position    = Signal::scale_out(0.50, ts, px);  // exit 50% of position
```

### Signal Tags

Label signals for post-backtest filtering with `trades_by_tag` / `metrics_by_tag`:

```rust
let signal = Signal::long(ts, px)
    .tag("breakout")
    .tag("high-volume");
```

## Ensemble Strategy

Combine multiple strategies and aggregate their signals with a voting rule:

```rust
use finance_query::backtesting::{EnsembleStrategy, EnsembleMode, SmaCrossover, RsiReversal};

let ensemble = EnsembleStrategy::new("Ensemble")
    .add(SmaCrossover::new(10, 50), 0.6)
    .add(RsiReversal::new(14), 0.4)
    .mode(EnsembleMode::WeightedMajority)
    .build();

let result = ticker.backtest(ensemble, Interval::OneDay, TimeRange::OneYear, None).await?;
```

**Voting modes:**

| Mode | Description |
|------|-------------|
| `WeightedMajority` | Entry if weighted vote share > 50% (default) |
| `Unanimous` | Entry only if all members agree |
| `AnySignal` | Entry if any member signals |
| `StrongestSignal` | Entry from the member with the highest signal strength |

## Higher-Timeframe Conditions

Evaluate a condition on a coarser timeframe within a lower-timeframe strategy using `htf()`:

```rust
use finance_query::backtesting::refs::*;
use finance_query::backtesting::condition::*;
use finance_query::backtesting::refs::htf;
use finance_query::{Interval, Region};

// Use daily RSI as a filter inside a 15-minute strategy
let strategy = StrategyBuilder::new("HTF RSI Filter")
    .entry(
        rsi(14).crosses_below(30.0)
            .and(htf(Interval::OneDay, rsi(14).above(40.0)))
    )
    .exit(rsi(14).crosses_above(70.0))
    .build();
```

HTF scope applies to computed indicators (RSI, SMA, MACD, etc.). Price-action refs (`price()`, `volume()`, etc.) always stay on the base timeframe.

## Benchmark Comparison

Compare your strategy against a benchmark symbol:

```rust
let result = ticker.backtest_with_benchmark(
    SmaCrossover::new(10, 50),
    Interval::OneDay,
    TimeRange::OneYear,
    None,
    "SPY",  // benchmark symbol
).await?;

if let Some(bench) = &result.benchmark {
    println!("Strategy return:   {:.2}%", result.metrics.total_return_pct);
    println!("Benchmark return:  {:.2}%", bench.benchmark_return_pct);
    println!("Buy & hold return: {:.2}%", bench.buy_and_hold_return_pct);
    println!("Alpha: {:.4}", bench.alpha);
    println!("Beta:  {:.4}", bench.beta);
    println!("Information Ratio: {:.4}", bench.information_ratio);
}
```

## Strategy Comparison

Rank multiple strategy results by a chosen metric:

```rust
use finance_query::backtesting::{BacktestComparison, OptimizeMetric};

let report = BacktestComparison::new()
    .add("SMA Crossover", result_sma)
    .add("RSI Reversal", result_rsi)
    .add("MACD Signal", result_macd)
    .ranked_by(OptimizeMetric::SharpeRatio);

println!("Winner: {}", report.winner());

for row in report.table() {
    println!(
        "#{} {} — Sharpe {:.2}, Return {:.2}%",
        row.rank, row.label, row.sharpe_ratio, row.total_return_pct,
    );
}
```

## Parameter Optimization

### Grid Search

Exhaustive parallel search over all parameter combinations:

```rust
use finance_query::backtesting::{GridSearch, ParamRange, OptimizeMetric, BacktestConfig};

let config = BacktestConfig::zero_cost();

let report = GridSearch::new()
    .param("fast", ParamRange::int_range(5, 20, 5))
    .param("slow", ParamRange::int_range(20, 60, 10))
    .optimize_for(OptimizeMetric::SharpeRatio)
    .run("AAPL", &candles, &config, |params| {
        SmaCrossover::new(
            params["fast"].as_int() as usize,
            params["slow"].as_int() as usize,
        )
    })?;

println!("Best Sharpe: {:.2}", report.best.result.metrics.sharpe_ratio);
println!("Best params: fast={}, slow={}",
    report.best.params["fast"].as_int(),
    report.best.params["slow"].as_int(),
);
println!("Evaluated {} combinations", report.n_evaluations);
```

### Bayesian Search (SAMBO)

Efficient adaptive search using a surrogate model — much faster for larger parameter spaces:

```rust
use finance_query::backtesting::BayesianSearch;

let report = BayesianSearch::new()
    .param("fast", ParamRange::int_bounds(5, 50))
    .param("slow", ParamRange::int_bounds(20, 200))
    .max_evaluations(100)
    .initial_points(10)
    .ucb_beta(2.0)
    .seed(42)
    .optimize_for(OptimizeMetric::SharpeRatio)
    .run("AAPL", &candles, &config, |params| {
        SmaCrossover::new(
            params["fast"].as_int() as usize,
            params["slow"].as_int() as usize,
        )
    })?;

// Convergence curve shows best score at each evaluation
println!("Convergence: {:?}", report.convergence_curve);
```

**`ParamRange` constructors:**

| Constructor | Description |
|-------------|-------------|
| `int_range(s, e, step)` | Integer grid (GridSearch) |
| `float_range(s, e, step)` | Float grid (GridSearch) |
| `int_bounds(s, e)` | Integer bounds, step=1 (BayesianSearch) |
| `float_bounds(s, e)` | Continuous float (BayesianSearch) |

**`OptimizeMetric` variants:** `TotalReturn`, `SharpeRatio`, `SortinoRatio`, `CalmarRatio`, `ProfitFactor`, `WinRate`, `MinDrawdown`

## Walk-Forward Validation

Validate out-of-sample performance by rolling an in-sample optimization window across the data:

```rust
use finance_query::backtesting::{WalkForwardConfig, GridSearch, ParamRange, OptimizeMetric, BacktestConfig};

let grid = GridSearch::new()
    .param("fast", ParamRange::int_range(5, 20, 5))
    .param("slow", ParamRange::int_range(20, 60, 10))
    .optimize_for(OptimizeMetric::SharpeRatio);

let config = BacktestConfig::builder()
    .initial_capital(10_000.0)
    .commission_pct(0.001)
    .build()?;

let report = WalkForwardConfig::new(grid, config)
    .in_sample_bars(252)      // 1 year in-sample
    .out_of_sample_bars(63)   // 1 quarter out-of-sample
    .run("AAPL", &candles, |params| {
        SmaCrossover::new(
            params["fast"].as_int() as usize,
            params["slow"].as_int() as usize,
        )
    })?;

println!("OOS Return:      {:.2}%", report.aggregate_metrics.total_return_pct);
println!("Consistency:     {:.1}%", report.consistency_ratio * 100.0);
println!("Windows tested:  {}", report.windows.len());

for w in &report.windows {
    println!(
        "Window {}: IS {:.1}% → OOS {:.1}%",
        w.window,
        w.in_sample.metrics.total_return_pct,
        w.out_of_sample.metrics.total_return_pct,
    );
}
```

## Monte Carlo Simulation

Stress-test a backtest result by running thousands of randomised trade-sequence simulations:

```rust
use finance_query::backtesting::{MonteCarloConfig, MonteCarloMethod};

let mc = MonteCarloConfig::new()
    .seed(42)
    .num_simulations(1_000)
    .method(MonteCarloMethod::IidShuffle)
    .run(&result);

println!("Return p5:  {:.2}%", mc.total_return.p5);
println!("Return p50: {:.2}%", mc.total_return.p50);
println!("Return p95: {:.2}%", mc.total_return.p95);
println!("Drawdown p95: {:.2}%", mc.max_drawdown.p95);
println!("Sharpe p50:   {:.2}", mc.sharpe_ratio.p50);
```

**`MonteCarloMethod` variants:**

| Method | Description |
|--------|-------------|
| `IidShuffle` (default) | Randomly shuffle trade returns (i.i.d. assumption) |
| `BlockBootstrap { block_size }` | Resample contiguous blocks to preserve autocorrelation |
| `StationaryBootstrap { mean_block_size }` | Random-length blocks (geometric distribution) |
| `Parametric` | Fit normal distribution to trade returns and sample |

## Portfolio Backtesting

Run the same strategy across multiple symbols with a shared capital pool:

```rust
use finance_query::backtesting::portfolio::{PortfolioConfig, PortfolioEngine, RebalanceMode, SymbolData};

let config = PortfolioConfig::new(BacktestConfig::builder()
    .initial_capital(50_000.0)
    .commission_pct(0.001)
    .build()?
)
.max_total_positions(3)
.rebalance(RebalanceMode::EqualWeight);

let symbol_data = vec![
    SymbolData::new("AAPL", aapl_candles),
    SymbolData::new("MSFT", msft_candles),
    SymbolData::new("GOOGL", googl_candles),
];

let result = PortfolioEngine::new(config)
    .run(&symbol_data, |_sym| Box::new(SmaCrossover::new(10, 50)))?;

println!("Portfolio Return: {:.2}%", result.portfolio_metrics.total_return_pct);
println!("Final Equity:     ${:.2}", result.final_equity);

for (sym, sym_result) in &result.symbols {
    println!("{}: {:.2}%", sym, sym_result.metrics.total_return_pct);
}
```

**`RebalanceMode` variants:**

| Mode | Description |
|------|-------------|
| `AvailableCapital` (default) | Each symbol uses `position_size_pct` of available cash |
| `EqualWeight` | Split initial capital equally among symbols |
| `CustomWeights(HashMap<String, f64>)` | Specify weight per symbol (fractions of initial capital) |

Via `Tickers::backtest()` — fetches charts and dividends automatically, then runs `PortfolioEngine`:

```rust
use finance_query::Tickers;
use finance_query::backtesting::{SmaCrossover, BacktestConfig};
use finance_query::backtesting::portfolio::{PortfolioConfig, RebalanceMode};

let tickers = Tickers::new(vec!["AAPL", "MSFT", "GOOGL"]).await?;
let config = PortfolioConfig::new(BacktestConfig::default())
    .max_total_positions(3)
    .rebalance(RebalanceMode::EqualWeight);

let result = tickers.backtest(
    Interval::OneDay,
    TimeRange::OneYear,
    Some(config),
    |_sym| SmaCrossover::new(10, 50),
).await?;
```

## Available Indicators

Use any of 40+ indicators in strategy conditions:

**Moving Averages:**
`sma`, `ema`, `wma`, `dema`, `tema`, `hma`, `vwma`, `alma`, `mcginley`

**Oscillators:**
`rsi`, `stochastic`, `stochastic_rsi`, `cci`, `williams_r`, `cmo`, `awesome_oscillator`

**Trend:**
`macd`, `adx`, `aroon`, `supertrend`, `ichimoku`, `parabolic_sar`

**Volatility:**
`atr`, `bollinger`, `keltner`, `donchian`, `choppiness_index`

**Volume:**
`obv`, `vwap`, `mfi`, `cmf`, `chaikin_oscillator`, `accumulation_distribution`, `balance_of_power`

## Available Conditions

**Comparisons:**

- `above(threshold)` - Value above threshold
- `below(threshold)` - Value below threshold
- `crosses_above(threshold)` - Crosses from below to above
- `crosses_below(threshold)` - Crosses from above to below
- `above_ref(indicator)` - Value above another indicator
- `crosses_above_ref(indicator)` - Crosses above another indicator
- `between(lower, upper)` - Value between two thresholds
- `equals(value)` - Value equals threshold

**Composites:**

- `and(condition)` - Both conditions must be true
- `or(condition)` - Either condition must be true
- `not()` - Negate condition

**Position Management:**

- `stop_loss(pct)` - Exit on loss percentage
- `take_profit(pct)` - Exit on profit percentage
- `trailing_stop(pct)` - Exit if price retraces by percentage
- `trailing_take_profit(pct)` - Exit if profit retraces

**Position State:**

- `has_position()` - Currently holding position
- `no_position()` - Not holding position
- `is_long()` - Currently long
- `is_short()` - Currently short
- `in_profit()` - Position is profitable
- `in_loss()` - Position is in loss

## Reference Signals

Access price and indicator values in conditions:

- `price()` - Close price
- `open()` - Open price
- `high()` - High price
- `low()` - Low price
- `volume()` - Volume

## Example: Complete Strategy

```rust
use finance_query::{Ticker, Interval, TimeRange};
use finance_query::backtesting::{StrategyBuilder, BacktestConfig};
use finance_query::backtesting::refs::*;
use finance_query::backtesting::condition::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ticker = Ticker::new("AAPL").await?;

    // Custom momentum strategy with regime filter and warmup
    let strategy = StrategyBuilder::new("Momentum with Risk Management")
        .entry(
            {
                let m = macd(12, 26, 9);
                m.line().crosses_above_ref(m.signal_line())
                    .and(price().above_ref(ema(50)))
                    .and(volume().above_ref(sma(20)))
            }
        )
        .exit(
            {
                let m = macd(12, 26, 9);
                m.line().crosses_below_ref(m.signal_line())
                    .or(stop_loss(0.08))
                    .or(take_profit(0.15))
            }
        )
        .regime_filter(price().above_ref(sma(200)))
        .warmup(200)
        .build();

    let config = BacktestConfig::builder()
        .initial_capital(100_000.0)
        .commission_pct(0.001)
        .slippage_pct(0.0005)
        .allow_short(false)
        .build()?;

    let result = ticker.backtest(
        strategy,
        Interval::OneDay,
        TimeRange::TwoYears,
        Some(config),
    ).await?;

    println!("Backtest Results for AAPL");
    println!("=========================");
    println!("Total Return: {:.2}%",  result.metrics.total_return_pct);
    println!("Sharpe Ratio: {:.2}",   result.metrics.sharpe_ratio);
    println!("Win Rate:     {:.2}%",  result.metrics.win_rate * 100.0);
    println!("Total Trades: {}",      result.metrics.total_trades);
    println!("Max Drawdown: {:.2}%",  result.metrics.max_drawdown_pct * 100.0);

    Ok(())
}
```

## Best Practices

!!! tip "Design Robust Strategies"
    - **Test multiple timeframes** - Validate strategies on different intervals and date ranges to avoid overfitting
    - **Use realistic assumptions** - Set appropriate commission, slippage, and position sizing
    - **Avoid lookahead bias** - Only use data that would have been available at the time of each trade
    - **Validate with walk-forward testing** - Test on out-of-sample data to ensure strategy generalizes
    - **Combine indicators** - Use multiple confirming signals rather than single indicator strategies

    ```rust
    // Good: Realistic configuration with multiple confirmations
    let config = BacktestConfig::builder()
        .initial_capital(10_000.0)
        .commission_pct(0.001)        // 0.1% per trade (realistic for retail)
        .slippage_pct(0.0005)         // 0.05% slippage
        .allow_short(false)           // Match your actual trading permissions
        .build()?;

    let strategy = StrategyBuilder::new("Validated Strategy")
        .entry(
            rsi(14).crosses_below(30.0)
                .and(price().above_ref(sma(200)))  // Trend filter
                .and(volume().above_ref(sma(20)))   // Volume confirmation
        )
        .exit(
            rsi(14).crosses_above(70.0)
                .or(stop_loss(0.05))               // Risk management
                .or(take_profit(0.15))
        )
        .build();

    let result = ticker.backtest(strategy, Interval::OneDay, TimeRange::OneYear, Some(config)).await?;
    ```

!!! warning "Common Pitfalls"
    - **Overfitting** - Strategies that work perfectly on historical data often fail in live trading. Use simple rules and validate on multiple periods.
    - **Ignoring costs** - Commission and slippage significantly impact returns, especially for high-frequency strategies.
    - **Position sizing** - Default 100% capital allocation is aggressive. Consider using smaller position sizes.
    - **Survivor bias** - Backtesting on current index constituents ignores delisted/bankrupt companies.
    - **Data quality** - Yahoo Finance data may have gaps or inaccuracies. Validate important results.

## Next Steps

- [Technical Indicators](indicators.md) - Complete reference for all 40+ available indicators
- [Ticker API](ticker.md) - Fetch historical data and run single-symbol backtests
- [Batch Tickers](tickers.md) - Portfolio backtesting across multiple symbols
- [Risk Analytics](risk.md) - Standalone VaR, Sharpe, and drawdown metrics
- [DataFrame Support](dataframe.md) - Analyze backtest results in Polars DataFrames
