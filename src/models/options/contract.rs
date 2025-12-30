use serde::{Deserialize, Serialize};
use std::ops::Deref;

/// A collection of option contracts with DataFrame support.
///
/// This wrapper allows `options.calls.to_dataframe()` syntax while still
/// acting like a `Vec<OptionContract>` for iteration, indexing, etc.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Contracts(pub Vec<OptionContract>);

impl Deref for Contracts {
    type Target = Vec<OptionContract>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl IntoIterator for Contracts {
    type Item = OptionContract;
    type IntoIter = std::vec::IntoIter<OptionContract>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Contracts {
    type Item = &'a OptionContract;
    type IntoIter = std::slice::Iter<'a, OptionContract>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

#[cfg(feature = "dataframe")]
impl Contracts {
    /// Converts the contracts to a polars DataFrame.
    pub fn to_dataframe(&self) -> ::polars::prelude::PolarsResult<::polars::prelude::DataFrame> {
        OptionContract::vec_to_dataframe(&self.0)
    }
}

/// An options contract (call or put)
///
/// Note: This struct cannot be manually constructed - obtain via `Ticker::options()`.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
#[serde(rename_all = "camelCase")]
pub struct OptionContract {
    /// Contract symbol (e.g., "AAPL250117C00150000")
    pub contract_symbol: String,

    /// Strike price
    pub strike: f64,

    /// Contract currency
    pub currency: Option<String>,

    /// Last trade price
    pub last_price: Option<f64>,

    /// Price change
    pub change: Option<f64>,

    /// Percent change
    pub percent_change: Option<f64>,

    /// Trading volume
    pub volume: Option<i64>,

    /// Open interest
    pub open_interest: Option<i64>,

    /// Bid price
    pub bid: Option<f64>,

    /// Ask price
    pub ask: Option<f64>,

    /// Contract size (usually 100)
    pub contract_size: Option<String>,

    /// Expiration date (Unix timestamp)
    pub expiration: Option<i64>,

    /// Last trade date (Unix timestamp)
    pub last_trade_date: Option<i64>,

    /// Implied volatility
    pub implied_volatility: Option<f64>,

    /// Whether the option is in the money
    pub in_the_money: Option<bool>,
}
