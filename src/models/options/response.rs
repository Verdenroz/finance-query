use super::contract::{Contracts, OptionContract};
/// Options Response module
///
/// Handles parsing of Yahoo Finance options API responses.
/// These types are internal implementation details and not exposed in the public API.
use serde::{Deserialize, Serialize};

/// Response wrapper for options endpoint
///
/// Note: While this type is public for return values, users should not manually construct it.
/// Use `Ticker::options()` to obtain options data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Options {
    /// Option chain container
    pub(crate) option_chain: OptionChainContainer,
}

/// Container for option chain results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct OptionChainContainer {
    /// Results array
    pub result: Vec<OptionChainResult>,

    /// Error if any
    pub error: Option<serde_json::Value>,
}

/// Single option chain result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OptionChainResult {
    /// Underlying symbol
    pub underlying_symbol: Option<String>,

    /// Available expiration dates (Unix timestamps)
    pub expiration_dates: Option<Vec<i64>>,

    /// Available strike prices
    pub strikes: Option<Vec<f64>>,

    /// Whether has mini options
    pub has_mini_options: Option<bool>,

    /// Quote data
    pub quote: Option<serde_json::Value>,

    /// Options data (array of option chains)
    pub options: Vec<OptionChainData>,
}

/// Option chain data for a specific expiration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OptionChainData {
    /// Expiration date (Unix timestamp)
    pub expiration_date: i64,

    /// Whether has mini options
    pub has_mini_options: Option<bool>,

    /// Call options
    pub calls: Option<Vec<OptionContract>>,

    /// Put options
    pub puts: Option<Vec<OptionContract>>,
}

impl Options {
    /// Get the first result
    pub(crate) fn first_result(&self) -> Option<&OptionChainResult> {
        self.option_chain.result.first()
    }

    /// Get available expiration dates
    pub fn expiration_dates(&self) -> Vec<i64> {
        self.first_result()
            .and_then(|r| r.expiration_dates.clone())
            .unwrap_or_default()
    }

    /// Get strike prices
    pub fn strikes(&self) -> Vec<f64> {
        self.first_result()
            .and_then(|r| r.strikes.clone())
            .unwrap_or_default()
    }

    /// Get all call contracts flattened across all expirations.
    ///
    /// Returns a `Contracts` wrapper that supports `.to_dataframe()` when
    /// the `dataframe` feature is enabled.
    ///
    /// # Example
    /// ```ignore
    /// let options = ticker.options(None).await?;
    /// for call in &options.calls {
    ///     println!("{}: strike={}", call.contract_symbol, call.strike);
    /// }
    /// // With dataframe feature:
    /// let df = options.calls.to_dataframe()?;
    /// ```
    pub fn calls(&self) -> Contracts {
        let contracts = self
            .first_result()
            .map(|r| {
                r.options
                    .iter()
                    .flat_map(|chain| chain.calls.as_ref().map(|c| c.iter()).unwrap_or_default())
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();
        Contracts(contracts)
    }

    /// Get all put contracts flattened across all expirations.
    ///
    /// Returns a `Contracts` wrapper that supports `.to_dataframe()` when
    /// the `dataframe` feature is enabled.
    ///
    /// # Example
    /// ```ignore
    /// let options = ticker.options(None).await?;
    /// for put in &options.puts {
    ///     println!("{}: strike={}", put.contract_symbol, put.strike);
    /// }
    /// // With dataframe feature:
    /// let df = options.puts.to_dataframe()?;
    /// ```
    pub fn puts(&self) -> Contracts {
        let contracts = self
            .first_result()
            .map(|r| {
                r.options
                    .iter()
                    .flat_map(|chain| chain.puts.as_ref().map(|p| p.iter()).unwrap_or_default())
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();
        Contracts(contracts)
    }
}

#[cfg(feature = "dataframe")]
impl Options {
    /// Converts all option contracts (calls and puts) to a polars DataFrame.
    ///
    /// Flattens all contracts across all expiration dates into a single DataFrame
    /// with an additional `option_type` column ("call" or "put").
    ///
    /// For separate DataFrames, use `options.calls.to_dataframe()` or
    /// `options.puts.to_dataframe()`.
    pub fn to_dataframe(&self) -> ::polars::prelude::PolarsResult<::polars::prelude::DataFrame> {
        use ::polars::prelude::*;

        let calls = self.calls();
        let puts = self.puts();

        let mut calls_df = calls.to_dataframe()?;
        let mut puts_df = puts.to_dataframe()?;

        // Add option_type column
        let call_types = Series::new("option_type".into(), vec!["call"; calls.len()]);
        let put_types = Series::new("option_type".into(), vec!["put"; puts.len()]);

        calls_df.with_column(call_types.into())?;
        puts_df.with_column(put_types.into())?;

        // Combine into single DataFrame
        calls_df.vstack(&puts_df)
    }
}
