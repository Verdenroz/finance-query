/// Common utilities for endpoint modules
///
/// This module provides shared functionality used across different endpoint implementations.
use crate::error::Result;

/// Helper to validate symbols are non-empty
pub fn validate_symbols(symbols: &[&str]) -> Result<()> {
    if symbols.is_empty() {
        return Err(crate::error::YahooError::InvalidParameter {
            param: "symbols".to_string(),
            reason: "No symbols provided".to_string(),
        });
    }
    Ok(())
}

/// Helper to validate a single symbol is non-empty
pub fn validate_symbol(symbol: &str) -> Result<()> {
    if symbol.trim().is_empty() {
        return Err(crate::error::YahooError::InvalidParameter {
            param: "symbol".to_string(),
            reason: "Empty symbol provided".to_string(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_symbol_valid() {
        assert!(validate_symbol("AAPL").is_ok());
        assert!(validate_symbol("GOOGL").is_ok());
        assert!(validate_symbol(" MSFT ").is_ok()); // with whitespace
    }

    #[test]
    fn test_validate_symbol_empty() {
        assert!(validate_symbol("").is_err());
        assert!(validate_symbol("   ").is_err()); // only whitespace
    }

    #[test]
    fn test_validate_symbols_valid() {
        assert!(validate_symbols(&["AAPL"]).is_ok());
        assert!(validate_symbols(&["AAPL", "GOOGL", "MSFT"]).is_ok());
    }

    #[test]
    fn test_validate_symbols_empty() {
        assert!(validate_symbols(&[]).is_err());
    }
}
