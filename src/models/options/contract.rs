use serde::{Deserialize, Serialize};

/// An options contract (call or put)
#[derive(Debug, Clone, Serialize, Deserialize)]
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
