# Backtesting

!!! abstract "Cargo Docs"
    [docs.rs/finance-query — backtesting](https://docs.rs/finance-query/latest/finance_query/backtesting/index.html)

Test trading strategies against historical data. The backtesting engine provides pre-built strategies, a strategy builder for custom logic, and comprehensive performance metrics.

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

Build custom strategies with the `StrategyBuilder`:

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

## Configuration

Customize backtesting behavior:

```rust
use finance_query::backtesting::BacktestConfig;

let config = BacktestConfig::builder()
    .initial_capital(50_000.0)        // Starting balance
    .commission_pct(0.001)            // 0.1% per trade
    .slippage_pct(0.0005)             // 0.05% slippage
    .stop_loss_pct(0.05)              // 5% stop-loss
    .take_profit_pct(0.15)            // 15% take-profit
    .allow_short(true)                // Allow short selling
    .build()?;

let result = ticker.backtest(
    SmaCrossover::new(10, 20),
    Interval::OneDay,
    TimeRange::OneYear,
    Some(config),
).await?;
```

## Performance Metrics

Access comprehensive backtest results:

```rust
let result = ticker.backtest(...).await?;

// Overall returns
println!("Total Return: {:.2}%", result.metrics.total_return_pct);
println!("Annualized Return: {:.2}%", result.metrics.annualized_return_pct);

// Risk metrics
println!("Sharpe Ratio: {:.2}", result.metrics.sharpe_ratio);
println!("Sortino Ratio: {:.2}", result.metrics.sortino_ratio);
println!("Max Drawdown: {:.2}%", result.metrics.max_drawdown_pct * 100.0);
println!("Calmar Ratio: {:.2}", result.metrics.calmar_ratio);

// Trade statistics
println!("Total Trades: {}", result.metrics.total_trades);
println!("Win Rate: {:.2}%", result.metrics.win_rate * 100.0);
println!("Profit Factor: {:.2}", result.metrics.profit_factor);
println!("Average Win: {:.2}%", result.metrics.avg_win_pct);
println!("Average Loss: {:.2}%", result.metrics.avg_loss_pct);

// Trading activity
println!("Long Trades: {}", result.metrics.long_trades);
println!("Short Trades: {}", result.metrics.short_trades);
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

- `above(threshold)` - Price above value
- `below(threshold)` - Price below value
- `crosses_above(threshold)` - Indicator crosses above value
- `crosses_below(threshold)` - Indicator crosses below value
- `between(lower, upper)` - Indicator between values
- `equals(value)` - Indicator equals value

**Composites:**

- `and()` - Both conditions true
- `or()` - Either condition true
- `not()` - Negate condition

**Position Management:**

- `stop_loss(pct)` - Exit on loss percentage
- `take_profit(pct)` - Exit on profit percentage
- `trailing_stop(pct)` - Exit if price retraces
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

## Trading Modes

Strategies support both long and short:

```rust
// Long only (default)
let strategy = SmaCrossover::new(10, 20);

// With configuration for short selling
let config = BacktestConfig::builder()
    .allow_short(true)
    .build()?;

let result = ticker.backtest(strategy, interval, range, Some(config)).await?;
```

## Example: Complete Strategy

```rust
use finance_query::{Ticker, Interval, TimeRange};
use finance_query::backtesting::{StrategyBuilder, BacktestConfig};
use finance_query::backtesting::refs::*;
use finance_query::backtesting::condition::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ticker = Ticker::new("AAPL").await?;

    // Custom momentum strategy
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
        .build();

    // Configuration
    let config = BacktestConfig::builder()
        .initial_capital(100_000.0)
        .commission_pct(0.001)
        .slippage_pct(0.0005)
        .allow_short(false)
        .build()?;

    // Run backtest
    let result = ticker.backtest(
        strategy,
        Interval::OneDay,
        TimeRange::OneYear,
        Some(config),
    ).await?;

    // Print results
    println!("Backtest Results for AAPL");
    println!("=========================");
    println!("Total Return: {:.2}%", result.metrics.total_return_pct);
    println!("Sharpe Ratio: {:.2}", result.metrics.sharpe_ratio);
    println!("Win Rate: {:.2}%", result.metrics.win_rate * 100.0);
    println!("Total Trades: {}", result.metrics.total_trades);
    println!("Max Drawdown: {:.2}%", result.metrics.max_drawdown_pct * 100.0);

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
        .slippage_pct(0.0005)          // 0.05% slippage
        .allow_short(false)            // Match your actual trading permissions
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

    // Run backtest
    let result = ticker.backtest(strategy, Interval::OneDay, TimeRange::OneYear, Some(config)).await?;

    // Less robust: Unrealistic assumptions, single indicator
    let bad_config = BacktestConfig::builder()
        .initial_capital(10_000.0)
        .commission_pct(0.0)           // No commission (unrealistic)
        .slippage_pct(0.0)             // No slippage (unrealistic)
        .build()?;

    let simple_strategy = StrategyBuilder::new("Simple")
        .entry(rsi(14).crosses_below(30.0))  // Single indicator, no confirmation
        .exit(rsi(14).crosses_above(70.0))
        .build();
    ```

!!! warning "Common Pitfalls"
    - **Overfitting** - Strategies that work perfectly on historical data often fail in live trading. Use simple rules and validate on multiple periods.
    - **Ignoring costs** - Commission and slippage significantly impact returns, especially for high-frequency strategies.
    - **Position sizing** - Default 100% capital allocation is aggressive. Consider using smaller position sizes.
    - **Survivor bias** - Backtesting on current index constituents ignores delisted/bankrupt companies.
    - **Data quality** - Yahoo Finance data may have gaps or inaccuracies. Validate important results.

## Next Steps

- [Technical Indicators](indicators.md) - Complete reference for all 52+ available indicators
- [Ticker API](ticker.md) - Learn how to fetch data and run backtests
- [DataFrame Support](dataframe.md) - Analyze backtest results in Polars DataFrames
- [Models Reference](models.md) - Understanding backtest result structures
