//! Price Module
//!
//! Contains detailed pricing data for a stock including pre/post market data,
//! exchange information, and market state.

use serde::{Deserialize, Serialize};

/// Detailed pricing data for a stock
///
/// Includes current price, pre/post market data, volume, market cap, and exchange information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub struct Price {
    /// Maximum age of the data in seconds
    #[serde(default)]
    pub max_age: Option<i64>,

    /// Pre-market change percentage
    #[serde(default)]
    pub pre_market_change_percent: Option<super::FormattedValue<f64>>,

    /// Pre-market change value
    #[serde(default)]
    pub pre_market_change: Option<super::FormattedValue<f64>>,

    /// Pre-market time as Unix timestamp
    #[serde(default)]
    pub pre_market_time: Option<i64>,

    /// Pre-market price
    #[serde(default)]
    pub pre_market_price: Option<super::FormattedValue<f64>>,

    /// Pre-market data source
    #[serde(default)]
    pub pre_market_source: Option<String>,

    /// Post-market change percentage
    #[serde(default)]
    pub post_market_change_percent: Option<super::FormattedValue<f64>>,

    /// Post-market change value
    #[serde(default)]
    pub post_market_change: Option<super::FormattedValue<f64>>,

    /// Post-market time as Unix timestamp
    #[serde(default)]
    pub post_market_time: Option<i64>,

    /// Post-market price
    #[serde(default)]
    pub post_market_price: Option<super::FormattedValue<f64>>,

    /// Post-market data source
    #[serde(default)]
    pub post_market_source: Option<String>,

    /// Regular market change percentage
    #[serde(default)]
    pub regular_market_change_percent: Option<super::FormattedValue<f64>>,

    /// Regular market change value
    #[serde(default)]
    pub regular_market_change: Option<super::FormattedValue<f64>>,

    /// Regular market time as Unix timestamp
    #[serde(default)]
    pub regular_market_time: Option<i64>,

    /// Price hint for decimal places
    #[serde(default)]
    pub price_hint: Option<super::FormattedValue<i64>>,

    /// Current regular market price
    #[serde(default)]
    pub regular_market_price: Option<super::FormattedValue<f64>>,

    /// Regular market day high
    #[serde(default)]
    pub regular_market_day_high: Option<super::FormattedValue<f64>>,

    /// Regular market day low
    #[serde(default)]
    pub regular_market_day_low: Option<super::FormattedValue<f64>>,

    /// Regular market volume
    #[serde(default)]
    pub regular_market_volume: Option<super::FormattedValue<i64>>,

    /// Average daily volume over 10 days
    #[serde(default)]
    pub average_daily_volume10_day: Option<super::FormattedValue<i64>>,

    /// Average daily volume over 3 months
    #[serde(default)]
    pub average_daily_volume3_month: Option<super::FormattedValue<i64>>,

    /// Regular market previous close
    #[serde(default)]
    pub regular_market_previous_close: Option<super::FormattedValue<f64>>,

    /// Regular market data source
    #[serde(default)]
    pub regular_market_source: Option<String>,

    /// Regular market open price
    #[serde(default)]
    pub regular_market_open: Option<super::FormattedValue<f64>>,

    /// Exchange code (e.g., "NMS" for NASDAQ)
    #[serde(default)]
    pub exchange: Option<String>,

    /// Exchange name (e.g., "NasdaqGS")
    #[serde(default)]
    pub exchange_name: Option<String>,

    /// Exchange data delay in seconds
    #[serde(default)]
    pub exchange_data_delayed_by: Option<i32>,

    /// Current market state (e.g., "REGULAR", "POST", "PRE")
    #[serde(default)]
    pub market_state: Option<String>,

    /// Quote type (e.g., "EQUITY", "ETF", "MUTUALFUND")
    #[serde(default)]
    pub quote_type: Option<String>,

    /// Stock symbol
    #[serde(default)]
    pub symbol: Option<String>,

    /// Underlying symbol (for derivatives)
    #[serde(default)]
    pub underlying_symbol: Option<String>,

    /// Short name of the security
    #[serde(default)]
    pub short_name: Option<String>,

    /// Long name of the security
    #[serde(default)]
    pub long_name: Option<String>,

    /// Currency code (e.g., "USD")
    #[serde(default)]
    pub currency: Option<String>,

    /// Quote source name
    #[serde(default)]
    pub quote_source_name: Option<String>,

    /// Currency symbol (e.g., "$")
    #[serde(default)]
    pub currency_symbol: Option<String>,

    /// From currency (for currency pairs)
    #[serde(default)]
    pub from_currency: Option<String>,

    /// To currency (for currency pairs)
    #[serde(default)]
    pub to_currency: Option<String>,

    /// Last market
    #[serde(default)]
    pub last_market: Option<String>,

    /// Market capitalization
    #[serde(default)]
    pub market_cap: Option<super::FormattedValue<i64>>,
}

impl Price {
    /// Returns the current price (regular market price)
    ///
    /// This is the most commonly used price value.
    pub fn current_price(&self) -> Option<f64> {
        self.regular_market_price.as_ref()?.raw
    }

    /// Returns the day's change in price
    pub fn day_change(&self) -> Option<f64> {
        self.regular_market_change.as_ref()?.raw
    }

    /// Returns the day's change as a percentage
    pub fn day_change_percent(&self) -> Option<f64> {
        self.regular_market_change_percent.as_ref()?.raw
    }

    /// Returns the day's trading range as (low, high)
    pub fn day_range(&self) -> Option<(f64, f64)> {
        let low = self.regular_market_day_low.as_ref()?.raw?;
        let high = self.regular_market_day_high.as_ref()?.raw?;
        Some((low, high))
    }

    /// Returns whether the market is currently open
    pub fn is_market_open(&self) -> bool {
        self.market_state.as_deref() == Some("REGULAR")
    }

    /// Returns whether this is in pre-market trading
    pub fn is_pre_market(&self) -> bool {
        self.market_state.as_deref() == Some("PRE")
    }

    /// Returns whether this is in post-market trading
    pub fn is_post_market(&self) -> bool {
        self.market_state.as_deref() == Some("POST")
    }

    /// Returns the most relevant current price based on market state
    ///
    /// Returns post-market price if in post-market, pre-market price if in pre-market,
    /// otherwise regular market price.
    pub fn live_price(&self) -> Option<f64> {
        if self.is_post_market() {
            self.post_market_price
                .as_ref()
                .and_then(|p| p.raw)
                .or_else(|| self.regular_market_price.as_ref()?.raw)
        } else if self.is_pre_market() {
            self.pre_market_price
                .as_ref()
                .and_then(|p| p.raw)
                .or_else(|| self.regular_market_price.as_ref()?.raw)
        } else {
            self.regular_market_price.as_ref()?.raw
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_price_deserialize() {
        let json = json!({
            "maxAge": 1,
            "regularMarketPrice": {"raw": 150.25, "fmt": "150.25"},
            "regularMarketChange": {"raw": 2.50, "fmt": "2.50"},
            "regularMarketChangePercent": {"raw": 0.0169, "fmt": "1.69%"},
            "regularMarketDayHigh": {"raw": 151.00, "fmt": "151.00"},
            "regularMarketDayLow": {"raw": 148.50, "fmt": "148.50"},
            "regularMarketVolume": {"raw": 50000000, "fmt": "50M"},
            "marketCap": {"raw": 2500000000000i64, "fmt": "2.5T"},
            "symbol": "AAPL",
            "shortName": "Apple Inc.",
            "longName": "Apple Inc.",
            "currency": "USD",
            "exchange": "NMS",
            "marketState": "REGULAR"
        });

        let price: Price = serde_json::from_value(json).unwrap();
        assert_eq!(
            price.regular_market_price.as_ref().and_then(|v| v.raw),
            Some(150.25)
        );
        assert_eq!(price.symbol, Some("AAPL".to_string()));
        assert_eq!(price.current_price(), Some(150.25));
        assert_eq!(price.day_change(), Some(2.50));
        assert!(price.is_market_open());
    }

    #[test]
    fn test_price_helpers() {
        use super::super::FormattedValue;

        let price = Price {
            regular_market_price: Some(FormattedValue::new(100.0)),
            regular_market_day_low: Some(FormattedValue::new(98.0)),
            regular_market_day_high: Some(FormattedValue::new(102.0)),
            market_state: Some("REGULAR".to_string()),
            post_market_price: Some(FormattedValue::new(101.0)),
            pre_market_price: Some(FormattedValue::new(99.0)),
            ..Default::default()
        };

        assert_eq!(price.day_range(), Some((98.0, 102.0)));
        assert_eq!(price.live_price(), Some(100.0));
        assert!(price.is_market_open());
    }

    #[test]
    fn test_live_price_post_market() {
        use super::super::FormattedValue;

        let price = Price {
            regular_market_price: Some(FormattedValue::new(100.0)),
            post_market_price: Some(FormattedValue::new(101.0)),
            market_state: Some("POST".to_string()),
            ..Default::default()
        };

        assert_eq!(price.live_price(), Some(101.0));
        assert!(price.is_post_market());
    }
}
