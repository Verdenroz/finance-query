//! Portfolio backtest configuration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::backtesting::config::BacktestConfig;
use crate::backtesting::error::{BacktestError, Result};

/// Controls how capital is divided among symbols when opening new positions.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum RebalanceMode {
    /// Use the base config's `position_size_pct` of current available cash for each trade.
    ///
    /// Natural "greedy" allocation — new positions are funded from whatever cash is on hand.
    #[default]
    AvailableCapital,

    /// Divide the initial capital equally among all available position slots.
    ///
    /// Slot count = `max_total_positions` if set, else the number of symbols.
    /// If a slot allocation exceeds available cash, available cash is used instead.
    EqualWeight,

    /// Custom per-symbol weight as a fraction of initial capital (0.0 – 1.0).
    ///
    /// Symbols not present in the map receive no allocation.
    /// Weights do not need to sum to 1.0 — they can total less (leaving spare cash).
    CustomWeights(HashMap<String, f64>),
}

/// Configuration for multi-symbol portfolio backtesting.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PortfolioConfig {
    /// Shared per-trade settings (commission, slippage, stop-loss, etc.)
    pub base: BacktestConfig,

    /// Maximum fraction of initial capital that can be allocated to a single symbol (0.0 – 1.0).
    ///
    /// `None` = no per-symbol cap (default).
    pub max_allocation_per_symbol: Option<f64>,

    /// Maximum number of concurrent open positions across all symbols.
    ///
    /// When the limit is reached, new entry signals are rejected until a position
    /// closes. Signals are ranked by strength; ties are broken alphabetically.
    /// `None` = unlimited (default).
    pub max_total_positions: Option<usize>,

    /// Capital allocation strategy when opening new positions.
    pub rebalance: RebalanceMode,
}

impl PortfolioConfig {
    /// Create a portfolio config wrapping the given single-symbol config.
    pub fn new(base: BacktestConfig) -> Self {
        Self {
            base,
            ..Self::default()
        }
    }

    /// Cap the fraction of initial capital allocated to any single symbol.
    pub fn max_allocation_per_symbol(mut self, pct: f64) -> Self {
        self.max_allocation_per_symbol = Some(pct);
        self
    }

    /// Limit the number of concurrent open positions across all symbols.
    pub fn max_total_positions(mut self, max: usize) -> Self {
        self.max_total_positions = Some(max);
        self
    }

    /// Set the capital allocation strategy.
    pub fn rebalance(mut self, mode: RebalanceMode) -> Self {
        self.rebalance = mode;
        self
    }

    /// Validate configuration constraints.
    pub fn validate(&self, num_symbols: usize) -> Result<()> {
        self.base.validate()?;

        if let Some(cap) = self.max_allocation_per_symbol
            && !(0.0..=1.0).contains(&cap)
        {
            return Err(BacktestError::invalid_param(
                "max_allocation_per_symbol",
                "must be between 0.0 and 1.0",
            ));
        }

        if let RebalanceMode::CustomWeights(ref weights) = self.rebalance {
            for (sym, &w) in weights {
                if !(0.0..=1.0).contains(&w) {
                    return Err(BacktestError::invalid_param(
                        sym.as_str(),
                        "custom weight must be between 0.0 and 1.0",
                    ));
                }
            }
        }

        if num_symbols == 0 {
            return Err(BacktestError::invalid_param(
                "symbol_data",
                "at least one symbol is required",
            ));
        }

        Ok(())
    }

    /// Compute the capital target for a new position in `symbol`.
    ///
    /// Returns the amount of capital to commit (before position sizing / commission
    /// adjustment). The caller must not exceed `available_cash`.
    pub(crate) fn allocation_target(
        &self,
        symbol: &str,
        available_cash: f64,
        initial_capital: f64,
        num_symbols: usize,
    ) -> f64 {
        let base = match &self.rebalance {
            RebalanceMode::AvailableCapital => available_cash * self.base.position_size_pct,
            RebalanceMode::EqualWeight => {
                let slots = self
                    .max_total_positions
                    .unwrap_or(num_symbols)
                    .min(num_symbols)
                    .max(1);
                initial_capital / slots as f64
            }
            RebalanceMode::CustomWeights(weights) => {
                let weight = weights.get(symbol).copied().unwrap_or(0.0);
                initial_capital * weight
            }
        };

        // Apply per-symbol cap
        let cap = self
            .max_allocation_per_symbol
            .map(|pct| initial_capital * pct)
            .unwrap_or(f64::MAX);

        base.min(cap).min(available_cash).max(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_validates() {
        let config = PortfolioConfig::default();
        assert!(config.validate(1).is_ok());
    }

    #[test]
    fn test_custom_weights_allocation() {
        let mut weights = HashMap::new();
        weights.insert("AAPL".to_string(), 0.5);
        weights.insert("MSFT".to_string(), 0.3);
        let config = PortfolioConfig::default().rebalance(RebalanceMode::CustomWeights(weights));

        let target = config.allocation_target("AAPL", 10_000.0, 10_000.0, 2);
        assert!((target - 5_000.0).abs() < 0.01);

        // Unknown symbol → 0
        let target_unknown = config.allocation_target("GOOG", 10_000.0, 10_000.0, 2);
        assert!((target_unknown - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_max_allocation_cap() {
        let config = PortfolioConfig::default().max_allocation_per_symbol(0.3);
        // EqualWeight would give 50% for 2 symbols; cap should reduce to 30%
        let config = config
            .rebalance(RebalanceMode::EqualWeight)
            .max_total_positions(2);
        let target = config.allocation_target("AAPL", 10_000.0, 10_000.0, 2);
        assert!((target - 3_000.0).abs() < 0.01, "got {target}");
    }

    #[test]
    fn test_validation_zero_symbols() {
        let config = PortfolioConfig::default();
        assert!(config.validate(0).is_err());
    }

    #[test]
    fn test_validation_invalid_cap() {
        let config = PortfolioConfig::default().max_allocation_per_symbol(1.5);
        assert!(config.validate(1).is_err());
    }
}
