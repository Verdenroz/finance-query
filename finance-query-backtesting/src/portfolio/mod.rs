//! Multi-symbol portfolio backtesting.
//!
//! Runs multiple strategies concurrently on a shared capital pool.
//!
//! # Quick Start
//!
//! ```ignore
//! use finance_query::backtesting::portfolio::{
//!     PortfolioConfig, PortfolioEngine, SymbolData, RebalanceMode,
//! };
//! use finance_query::backtesting::{BacktestConfig, SmaCrossover};
//!
//! let config = PortfolioConfig::new(BacktestConfig::default())
//!     .max_total_positions(3)
//!     .rebalance(RebalanceMode::EqualWeight);
//!
//! let symbol_data = vec![
//!     SymbolData::new("AAPL", aapl_candles),
//!     SymbolData::new("MSFT", msft_candles),
//!     SymbolData::new("GOOG", goog_candles),
//! ];
//!
//! let result = PortfolioEngine::new(config)
//!     .run(&symbol_data, |_sym| SmaCrossover::new(10, 50))
//!     .unwrap();
//!
//! println!("Portfolio return: {:.2}%", result.portfolio_metrics.total_return_pct);
//! ```

mod config;
mod engine;
mod result;

pub use config::{PortfolioConfig, RebalanceMode};
pub use engine::{PortfolioEngine, SymbolData};
pub use result::{AllocationSnapshot, PortfolioResult};
