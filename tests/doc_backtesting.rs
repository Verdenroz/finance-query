//! Compile and runtime tests for docs/library/backtesting.md
//!
//! Requires the `backtesting` feature flag:
//!   cargo test --test doc_backtesting --features backtesting
//!   cargo test --test doc_backtesting --features backtesting -- --ignored   (network tests)

#![cfg(feature = "backtesting")]

use finance_query::backtesting::condition::*;
use finance_query::backtesting::refs::*;
use finance_query::backtesting::{BacktestConfig, StrategyBuilder};

// ---------------------------------------------------------------------------
// Compile-time — BacktestConfig builder documented in backtesting.md
// ---------------------------------------------------------------------------

#[test]
fn test_backtest_config_builder() {
    // From backtesting.md "Configuration" section
    let config = BacktestConfig::builder()
        .initial_capital(50_000.0)
        .commission_pct(0.001)
        .slippage_pct(0.0005)
        .stop_loss_pct(0.05)
        .take_profit_pct(0.15)
        .allow_short(true)
        .build()
        .unwrap();

    let _ = config;
}

#[test]
fn test_backtest_config_defaults() {
    // From backtesting.md "Trading Modes" section
    let config = BacktestConfig::builder()
        .initial_capital(10_000.0)
        .commission_pct(0.001)
        .slippage_pct(0.0005)
        .allow_short(false)
        .build()
        .unwrap();

    let _ = config;
}

#[test]
fn test_backtest_config_no_costs() {
    // From backtesting.md "Best Practices" — less robust example
    let bad_config = BacktestConfig::builder()
        .initial_capital(10_000.0)
        .commission_pct(0.0)
        .slippage_pct(0.0)
        .build()
        .unwrap();

    let _ = bad_config;
}

// ---------------------------------------------------------------------------
// Compile-time — StrategyBuilder documented in backtesting.md
// ---------------------------------------------------------------------------

#[test]
fn test_strategy_builder_rsi_mean_reversion() {
    // From backtesting.md "Custom Strategies" section
    let strategy = StrategyBuilder::new("RSI Mean Reversion")
        .entry(rsi(14).crosses_below(30.0).and(price().above_ref(sma(200))))
        .exit(rsi(14).crosses_above(70.0).or(stop_loss(0.05)))
        .build();

    let _ = strategy;
}

#[test]
fn test_strategy_builder_validated() {
    // From backtesting.md "Best Practices" section
    let strategy = StrategyBuilder::new("Validated Strategy")
        .entry(
            rsi(14)
                .crosses_below(30.0)
                .and(price().above_ref(sma(200))) // Trend filter
                .and(volume().above_ref(sma(20))), // Volume confirmation
        )
        .exit(
            rsi(14)
                .crosses_above(70.0)
                .or(stop_loss(0.05))
                .or(take_profit(0.15)),
        )
        .build();

    let _ = strategy;
}

#[test]
fn test_strategy_builder_simple() {
    // From backtesting.md "Best Practices" — single indicator (less robust) example
    let simple_strategy = StrategyBuilder::new("Simple")
        .entry(rsi(14).crosses_below(30.0))
        .exit(rsi(14).crosses_above(70.0))
        .build();

    let _ = simple_strategy;
}

// ---------------------------------------------------------------------------
// Network tests — pre-built strategies from backtesting.md
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_sma_crossover() {
    use finance_query::backtesting::SmaCrossover;
    use finance_query::{Interval, Ticker, TimeRange};

    // From backtesting.md "SMA Crossover" section
    let ticker = Ticker::new("AAPL").await.unwrap();
    let result = ticker
        .backtest(
            SmaCrossover::new(10, 20),
            Interval::OneDay,
            TimeRange::OneYear,
            None,
        )
        .await
        .unwrap();

    println!("Total Return: {:.2}%", result.metrics.total_return_pct);
    println!("Sharpe Ratio: {:.2}", result.metrics.sharpe_ratio);
    println!(
        "Max Drawdown: {:.2}%",
        result.metrics.max_drawdown_pct * 100.0
    );
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_rsi_reversal() {
    use finance_query::backtesting::RsiReversal;
    use finance_query::{Interval, Ticker, TimeRange};

    // From backtesting.md "RSI Mean Reversion" section
    let ticker = Ticker::new("AAPL").await.unwrap();

    let result = ticker
        .backtest(
            RsiReversal::new(14),
            Interval::OneDay,
            TimeRange::OneYear,
            None,
        )
        .await
        .unwrap();
    let _ = result;

    // Or with custom thresholds:
    let result = ticker
        .backtest(
            RsiReversal::new(14).with_thresholds(30.0, 70.0),
            Interval::OneDay,
            TimeRange::OneYear,
            None,
        )
        .await
        .unwrap();
    let _ = result;
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_macd_signal() {
    use finance_query::backtesting::MacdSignal;
    use finance_query::{Interval, Ticker, TimeRange};

    // From backtesting.md "MACD Signal Crossover" section
    let ticker = Ticker::new("AAPL").await.unwrap();
    let result = ticker
        .backtest(
            MacdSignal::new(12, 26, 9),
            Interval::OneDay,
            TimeRange::OneYear,
            None,
        )
        .await
        .unwrap();
    let _ = result;
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_bollinger_mean_reversion() {
    use finance_query::backtesting::BollingerMeanReversion;
    use finance_query::{Interval, Ticker, TimeRange};

    // From backtesting.md "Bollinger Band Mean Reversion" section
    let ticker = Ticker::new("AAPL").await.unwrap();
    let result = ticker
        .backtest(
            BollingerMeanReversion::new(20, 2.0),
            Interval::OneDay,
            TimeRange::OneYear,
            None,
        )
        .await
        .unwrap();
    let _ = result;
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_supertrend_follow() {
    use finance_query::backtesting::SuperTrendFollow;
    use finance_query::{Interval, Ticker, TimeRange};

    // From backtesting.md "SuperTrend Trend Following" section
    let ticker = Ticker::new("AAPL").await.unwrap();
    let result = ticker
        .backtest(
            SuperTrendFollow::new(10, 3.0),
            Interval::OneDay,
            TimeRange::OneYear,
            None,
        )
        .await
        .unwrap();
    let _ = result;
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_donchian_breakout() {
    use finance_query::backtesting::DonchianBreakout;
    use finance_query::{Interval, Ticker, TimeRange};

    // From backtesting.md "Donchian Breakout" section
    let ticker = Ticker::new("AAPL").await.unwrap();
    let result = ticker
        .backtest(
            DonchianBreakout::new(20),
            Interval::OneDay,
            TimeRange::OneYear,
            None,
        )
        .await
        .unwrap();
    let _ = result;
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_performance_metrics_access() {
    use finance_query::backtesting::SmaCrossover;
    use finance_query::{Interval, Ticker, TimeRange};

    // From backtesting.md "Performance Metrics" section
    let ticker = Ticker::new("AAPL").await.unwrap();
    let result = ticker
        .backtest(
            SmaCrossover::new(10, 20),
            Interval::OneDay,
            TimeRange::OneYear,
            None,
        )
        .await
        .unwrap();

    // Overall returns
    println!("Total Return: {:.2}%", result.metrics.total_return_pct);
    println!(
        "Annualized Return: {:.2}%",
        result.metrics.annualized_return_pct
    );

    // Risk metrics
    println!("Sharpe Ratio: {:.2}", result.metrics.sharpe_ratio);
    println!("Sortino Ratio: {:.2}", result.metrics.sortino_ratio);
    println!(
        "Max Drawdown: {:.2}%",
        result.metrics.max_drawdown_pct * 100.0
    );
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
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_complete_strategy_example() {
    use finance_query::backtesting::condition::*;
    use finance_query::backtesting::refs::*;
    use finance_query::backtesting::{BacktestConfig, StrategyBuilder};
    use finance_query::{Interval, Ticker, TimeRange};

    // From backtesting.md "Example: Complete Strategy" section
    let ticker = Ticker::new("AAPL").await.unwrap();

    // Custom momentum strategy
    let strategy = StrategyBuilder::new("Momentum with Risk Management")
        .entry({
            let m = macd(12, 26, 9);
            m.line()
                .crosses_above_ref(m.signal_line())
                .and(price().above_ref(ema(50)))
                .and(volume().above_ref(sma(20)))
        })
        .exit({
            let m = macd(12, 26, 9);
            m.line()
                .crosses_below_ref(m.signal_line())
                .or(stop_loss(0.08))
                .or(take_profit(0.15))
        })
        .build();

    // Configuration
    let config = BacktestConfig::builder()
        .initial_capital(100_000.0)
        .commission_pct(0.001)
        .slippage_pct(0.0005)
        .allow_short(false)
        .build()
        .unwrap();

    // Run backtest
    let result = ticker
        .backtest(strategy, Interval::OneDay, TimeRange::OneYear, Some(config))
        .await
        .unwrap();

    // Print results
    println!("Backtest Results for AAPL");
    println!("=========================");
    println!("Total Return: {:.2}%", result.metrics.total_return_pct);
    println!("Sharpe Ratio: {:.2}", result.metrics.sharpe_ratio);
    println!("Win Rate: {:.2}%", result.metrics.win_rate * 100.0);
    println!("Total Trades: {}", result.metrics.total_trades);
    println!(
        "Max Drawdown: {:.2}%",
        result.metrics.max_drawdown_pct * 100.0
    );
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_custom_strategy_backtest() {
    use finance_query::{Interval, Ticker, TimeRange};

    // From backtesting.md "Custom Strategies" section — the ticker.backtest() call
    let ticker = Ticker::new("AAPL").await.unwrap();

    let strategy = StrategyBuilder::new("RSI Mean Reversion")
        .entry(rsi(14).crosses_below(30.0).and(price().above_ref(sma(200))))
        .exit(rsi(14).crosses_above(70.0).or(stop_loss(0.05)))
        .build();

    let result = ticker
        .backtest(strategy, Interval::OneDay, TimeRange::OneYear, None)
        .await
        .unwrap();

    let _ = result;
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_backtest_with_full_config() {
    use finance_query::backtesting::SmaCrossover;
    use finance_query::{Interval, Ticker, TimeRange};

    // From backtesting.md "Configuration" section — the ticker.backtest() call
    let ticker = Ticker::new("AAPL").await.unwrap();

    let config = BacktestConfig::builder()
        .initial_capital(50_000.0)
        .commission_pct(0.001)
        .slippage_pct(0.0005)
        .stop_loss_pct(0.05)
        .take_profit_pct(0.15)
        .allow_short(true)
        .build()
        .unwrap();

    let result = ticker
        .backtest(
            SmaCrossover::new(10, 20),
            Interval::OneDay,
            TimeRange::OneYear,
            Some(config),
        )
        .await
        .unwrap();

    let _ = result;
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_trading_modes_allow_short() {
    use finance_query::backtesting::SmaCrossover;
    use finance_query::{Interval, Ticker, TimeRange};

    // From backtesting.md "Trading Modes" section
    let ticker = Ticker::new("AAPL").await.unwrap();

    // Long only (default)
    let strategy = SmaCrossover::new(10, 20);

    // With configuration for short selling
    let config = BacktestConfig::builder().allow_short(true).build().unwrap();

    let result = ticker
        .backtest(strategy, Interval::OneDay, TimeRange::OneYear, Some(config))
        .await
        .unwrap();

    let _ = result;
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_strategy_multiple_periods() {
    use finance_query::backtesting::condition::*;
    use finance_query::backtesting::refs::*;
    use finance_query::backtesting::{BacktestConfig, StrategyBuilder};
    use finance_query::{Interval, Ticker, TimeRange};

    // From backtesting.md "Best Practices" section
    let ticker = Ticker::new("AAPL").await.unwrap();

    let config = BacktestConfig::builder()
        .initial_capital(10_000.0)
        .commission_pct(0.001)
        .slippage_pct(0.0005)
        .allow_short(false)
        .build()
        .unwrap();

    let strategy = StrategyBuilder::new("Validated Strategy")
        .entry(
            rsi(14)
                .crosses_below(30.0)
                .and(price().above_ref(sma(200)))
                .and(volume().above_ref(sma(20))),
        )
        .exit(
            rsi(14)
                .crosses_above(70.0)
                .or(stop_loss(0.05))
                .or(take_profit(0.15)),
        )
        .build();

    // Run backtest
    let result = ticker
        .backtest(strategy, Interval::OneDay, TimeRange::OneYear, Some(config))
        .await
        .unwrap();

    println!("1Y Return: {:.2}%", result.metrics.total_return_pct);
}
