//! Backtesting engine for trading strategy simulation.
//!
//! This module provides a complete backtesting framework with:
//! - Expression-based strategy builder for custom entry/exit conditions
//! - Pre-built strategies (SMA, RSI, MACD, Bollinger, SuperTrend, Donchian)
//! - Full technical indicator coverage (40+ indicators)
//! - Position tracking with long/short support
//! - Stop-loss, take-profit, and trailing stop management
//! - Comprehensive performance metrics
//!
//! # Quick Start
//!
//! ```no_run
//! use finance_query::{Ticker, Interval, TimeRange};
//! use finance_query::backtesting::{SmaCrossover, BacktestConfig};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let ticker = Ticker::new("AAPL").await?;
//! let result = ticker.backtest(
//!     SmaCrossover::new(10, 20),
//!     Interval::OneDay,
//!     TimeRange::OneYear,
//!     None,
//! ).await?;
//!
//! println!("Return: {:.2}%", result.metrics.total_return_pct);
//! println!("Sharpe: {:.2}", result.metrics.sharpe_ratio);
//! println!("Max Drawdown: {:.2}%", result.metrics.max_drawdown_pct * 100.0);
//! # Ok(())
//! # }
//! ```
//!
//! # Custom Strategies with StrategyBuilder
//!
//! Build custom strategies using the fluent builder API:
//!
//! ```ignore
//! use finance_query::backtesting::StrategyBuilder;
//! use finance_query::backtesting::refs::*;
//! use finance_query::backtesting::condition::*;
//!
//! let strategy = StrategyBuilder::new("RSI Mean Reversion")
//!     .entry(
//!         rsi(14).crosses_below(30.0)
//!             .and(price().above_ref(sma(200)))
//!     )
//!     .exit(
//!         rsi(14).crosses_above(70.0)
//!             .or(stop_loss(0.05))
//!     )
//!     .build();
//! ```
//!
//! # Configuration
//!
//! ```no_run
//! use finance_query::backtesting::BacktestConfig;
//!
//! let config = BacktestConfig::builder()
//!     .initial_capital(50_000.0)
//!     .commission_pct(0.001)      // 0.1% per trade
//!     .slippage_pct(0.0005)       // 0.05% slippage
//!     .stop_loss_pct(0.05)        // 5% stop-loss
//!     .take_profit_pct(0.15)      // 15% take-profit
//!     .allow_short(true)
//!     .build()
//!     .unwrap();
//! ```
//!
//! # Pre-built Strategies
//!
//! | Strategy | Parameters | Description |
//! |----------|------------|-------------|
//! | [`SmaCrossover`] | fast, slow periods | Dual SMA crossover |
//! | [`RsiReversal`] | period, oversold, overbought | Mean reversion on RSI |
//! | [`MacdSignal`] | fast, slow, signal periods | MACD line crossover |
//! | [`BollingerMeanReversion`] | period, std_dev | Buy at lower band |
//! | [`SuperTrendFollow`] | period, multiplier | Trend following |
//! | [`DonchianBreakout`] | period | Channel breakout |
//!
//! # Available Indicators
//!
//! The strategy builder supports all indicators via [`refs`]:
//!
//! - **Moving Averages**: `sma`, `ema`, `wma`, `dema`, `tema`, `hma`, `vwma`, `alma`, `mcginley`
//! - **Oscillators**: `rsi`, `stochastic`, `stochastic_rsi`, `cci`, `williams_r`, `cmo`, `awesome_oscillator`
//! - **Trend**: `macd`, `adx`, `aroon`, `supertrend`, `ichimoku`, `parabolic_sar`
//! - **Volatility**: `atr`, `bollinger`, `keltner`, `donchian`, `choppiness_index`
//! - **Volume**: `obv`, `vwap`, `mfi`, `cmf`, `chaikin_oscillator`, `accumulation_distribution`, `balance_of_power`
//!
//! # Available Conditions
//!
//! Build conditions via [`condition`]:
//!
//! - **Comparisons**: `above()`, `below()`, `crosses_above()`, `crosses_below()`, `between()`, `equals()`
//! - **Composites**: `and()`, `or()`, `not()`
//! - **Position Management**: `stop_loss()`, `take_profit()`, `trailing_stop()`, `trailing_take_profit()`
//! - **Position State**: `has_position()`, `no_position()`, `is_long()`, `is_short()`, `in_profit()`, `in_loss()`

pub mod condition;
mod config;
mod engine;
mod error;
pub mod monte_carlo;
pub mod optimizer;
pub mod portfolio;
mod position;
pub mod refs;
mod result;
mod signal;
pub mod strategy;
pub mod walk_forward;

// Re-export main types
pub use config::{BacktestConfig, BacktestConfigBuilder};
pub use engine::BacktestEngine;
pub use error::{BacktestError, Result};
pub use position::{Position, PositionSide, Trade};
pub use result::{BacktestResult, BenchmarkMetrics, EquityPoint, PerformanceMetrics, SignalRecord};
pub use signal::{Signal, SignalDirection, SignalMetadata, SignalStrength};

// Re-export strategy types
pub use strategy::{Strategy, StrategyContext};

// Re-export strategy builder
pub use strategy::StrategyBuilder;

// Re-export pre-built strategies
pub use strategy::{
    BollingerMeanReversion, DonchianBreakout, MacdSignal, RsiReversal, SmaCrossover,
    SuperTrendFollow,
};

// Re-export optimiser types for convenience
pub use optimizer::{
    GridSearch, OptimizationReport, OptimizationResult, OptimizeMetric, ParamRange, ParamValue,
};

// Re-export walk-forward types
pub use walk_forward::{WalkForwardConfig, WalkForwardReport, WindowResult};

// Re-export Monte Carlo types
pub use monte_carlo::{MonteCarloConfig, MonteCarloResult, PercentileStats};
