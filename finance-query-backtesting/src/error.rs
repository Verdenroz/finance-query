//! Error types for backtesting operations.

use thiserror::Error;

/// Backtest-specific errors
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum BacktestError {
    /// Invalid configuration parameter
    #[error("Invalid parameter '{param}': {reason}")]
    InvalidParameter {
        /// Parameter name
        param: String,
        /// Reason for invalidity
        reason: String,
    },

    /// Insufficient data for backtest
    #[error("Insufficient data: need at least {need} candles, got {got}")]
    InsufficientData {
        /// Minimum required candles
        need: usize,
        /// Actual candles provided
        got: usize,
    },

    /// Strategy error during execution
    #[error("Strategy error: {0}")]
    StrategyError(String),

    /// Indicator calculation failed
    #[error("Indicator calculation failed: {0}")]
    IndicatorError(#[from] crate::indicators::IndicatorError),

    /// Chart data fetch failed
    #[error("Failed to fetch chart data: {0}")]
    ChartError(String),

    /// Position management error
    #[error("Position error: {0}")]
    PositionError(String),
}

/// Result type for backtest operations
pub type Result<T> = std::result::Result<T, BacktestError>;

impl BacktestError {
    /// Create an invalid parameter error
    pub fn invalid_param(param: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidParameter {
            param: param.into(),
            reason: reason.into(),
        }
    }

    /// Create an insufficient data error
    pub fn insufficient_data(need: usize, got: usize) -> Self {
        Self::InsufficientData { need, got }
    }

    /// Create a strategy error
    pub fn strategy(msg: impl Into<String>) -> Self {
        Self::StrategyError(msg.into())
    }

    /// Create a chart error
    pub fn chart(msg: impl Into<String>) -> Self {
        Self::ChartError(msg.into())
    }

    /// Create a position error
    pub fn position(msg: impl Into<String>) -> Self {
        Self::PositionError(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_param_error() {
        let err = BacktestError::invalid_param("capital", "must be positive");
        assert!(err.to_string().contains("capital"));
        assert!(err.to_string().contains("must be positive"));
    }

    #[test]
    fn test_insufficient_data_error() {
        let err = BacktestError::insufficient_data(50, 10);
        assert!(err.to_string().contains("50"));
        assert!(err.to_string().contains("10"));
    }
}
