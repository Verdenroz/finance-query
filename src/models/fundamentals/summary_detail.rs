use crate::models::format::{Both, Format};
use finance_query_derive::FormatConvert;
/// Summary Detail module
///
/// Contains detailed trading and valuation metrics for the symbol.
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[cfg(feature = "python")]
use finance_query_derive::PyModel;

/// Summary detail trading and valuation metrics
///
/// Contains detailed information about price, volume, market cap, and other trading data.
///
/// The type parameter `F` controls how numeric fields are represented:
/// - `SummaryDetail` / `SummaryDetail<Both>` — **default**; fields hold `FormattedValue<T>`
/// - `SummaryDetail<Raw>` — fields hold `T` directly (e.g. `Option<f64>`)
/// - `SummaryDetail<Pretty>` — fields hold `Option<String>` (human-readable)
///
/// Obtain converted views via [`Quote::as_raw`](crate::Quote::as_raw) or call
/// `.as_raw()` / `.into_raw()` on a `SummaryDetail<Both>` directly.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FormatConvert)]
#[cfg_attr(feature = "python", derive(PyModel))]
#[serde(rename_all = "camelCase", bound = "")]
pub struct SummaryDetail<F: Format = Both> {
    /// Algorithm (for crypto/special assets)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub algorithm: Option<Value>,

    /// All-time high price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_time_high: Option<F::Value<f64>>,

    /// All-time low price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_time_low: Option<F::Value<f64>>,

    /// Current ask price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ask: Option<F::Value<f64>>,

    /// Ask size (shares)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ask_size: Option<F::Value<i64>>,

    /// Average daily trading volume (10 day)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub average_daily_volume10_day: Option<F::Value<i64>>,

    /// Average trading volume
    #[serde(skip_serializing_if = "Option::is_none")]
    pub average_volume: Option<F::Value<i64>>,

    /// Average trading volume (10 days)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub average_volume10days: Option<F::Value<i64>>,

    /// Beta coefficient (volatility vs market)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub beta: Option<F::Value<f64>>,

    /// Current bid price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bid: Option<F::Value<f64>>,

    /// Bid size (shares)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bid_size: Option<F::Value<i64>>,

    /// Circulating supply (for crypto)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub circulating_supply: Option<Value>,

    /// CoinMarketCap link (for crypto)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coin_market_cap_link: Option<Value>,

    /// Currency code
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,

    /// Day's high price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub day_high: Option<F::Value<f64>>,

    /// Day's low price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub day_low: Option<F::Value<f64>>,

    /// Annual dividend rate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dividend_rate: Option<F::Value<f64>>,

    /// Dividend yield percentage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dividend_yield: Option<F::Value<f64>>,

    /// Ex-dividend date
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ex_dividend_date: Option<F::Value<i64>>,

    /// Expiration date (for options/futures)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expire_date: Option<Value>,

    /// 50-day moving average
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fifty_day_average: Option<F::Value<f64>>,

    /// 52-week high price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fifty_two_week_high: Option<F::Value<f64>>,

    /// 52-week low price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fifty_two_week_low: Option<F::Value<f64>>,

    /// 5-year average dividend yield
    #[serde(skip_serializing_if = "Option::is_none")]
    pub five_year_avg_dividend_yield: Option<F::Value<f64>>,

    /// Forward price-to-earnings ratio
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forward_pe: Option<F::Value<f64>>,

    /// From currency (for currency pairs)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_currency: Option<String>,

    /// Last market (for crypto)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_market: Option<String>,

    /// Market capitalization
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_cap: Option<F::Value<i64>>,

    /// Maximum age of data in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_age: Option<i64>,

    /// Maximum supply (for crypto)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_supply: Option<Value>,

    /// Net asset value price (for funds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nav_price: Option<F::Value<f64>>,

    /// Opening price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open: Option<F::Value<f64>>,

    /// Open interest (for options/futures)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_interest: Option<Value>,

    /// Dividend payout ratio
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payout_ratio: Option<F::Value<f64>>,

    /// Previous closing price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_close: Option<F::Value<f64>>,

    /// Price hint (decimal places)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_hint: Option<F::Value<i64>>,

    /// Price to sales ratio (trailing 12 months)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_to_sales_trailing12_months: Option<F::Value<f64>>,

    /// Quarter-to-date return
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qtd_return: Option<Value>,

    /// Regular market day high
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regular_market_day_high: Option<F::Value<f64>>,

    /// Regular market day low
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regular_market_day_low: Option<F::Value<f64>>,

    /// Regular market opening price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regular_market_open: Option<F::Value<f64>>,

    /// Regular market previous close
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regular_market_previous_close: Option<F::Value<f64>>,

    /// Regular market trading volume
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regular_market_volume: Option<F::Value<i64>>,

    /// Start date (for funds/special assets)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<Value>,

    /// Strike price (for options)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strike_price: Option<Value>,

    /// To currency (for currency pairs)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_currency: Option<String>,

    /// Total assets (for funds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_assets: Option<F::Value<i64>>,

    /// Whether the security is tradeable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tradeable: Option<bool>,

    /// Trailing annual dividend rate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trailing_annual_dividend_rate: Option<F::Value<f64>>,

    /// Trailing annual dividend yield
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trailing_annual_dividend_yield: Option<F::Value<f64>>,

    /// Trailing price-to-earnings ratio
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trailing_pe: Option<F::Value<f64>>,

    /// 200-day moving average
    #[serde(skip_serializing_if = "Option::is_none")]
    pub two_hundred_day_average: Option<F::Value<f64>>,

    /// Trading volume
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<F::Value<i64>>,

    /// 24-hour trading volume (for crypto)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume24_hr: Option<Value>,

    /// Volume across all currencies (for crypto)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume_all_currencies: Option<Value>,

    /// Yield (for bonds/funds)
    #[serde(rename = "yield", skip_serializing_if = "Option::is_none")]
    pub yield_value: Option<F::Value<f64>>,

    /// Year-to-date return
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ytd_return: Option<Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_summary_detail() {
        let json = r#"{
            "currency": "USD",
            "previousClose": {"fmt": "275.00", "raw": 275.0},
            "marketCap": {"fmt": "4.09T", "longFmt": "4,090,000,000,000", "raw": 4090000000000},
            "beta": {"fmt": "1.11", "raw": 1.109},
            "tradeable": true
        }"#;

        let detail: SummaryDetail = serde_json::from_str(json).unwrap();
        assert_eq!(detail.currency.as_deref(), Some("USD"));
        assert_eq!(
            detail.previous_close.as_ref().map(|v| v.raw),
            Some(Some(275.0))
        );
        assert_eq!(
            detail.market_cap.as_ref().map(|v| v.raw),
            Some(Some(4090000000000))
        );
        assert_eq!(detail.tradeable, Some(true));
    }

    #[test]
    fn test_into_raw() {
        let json = r#"{
            "currency": "USD",
            "previousClose": {"fmt": "275.00", "raw": 275.0},
            "beta": {"fmt": "1.11", "raw": 1.109}
        }"#;

        let detail: SummaryDetail = serde_json::from_str(json).unwrap();
        let raw = detail.into_raw();
        assert_eq!(raw.currency.as_deref(), Some("USD"));
        assert_eq!(raw.previous_close, Some(275.0));
        assert_eq!(raw.beta, Some(1.109));
    }

    #[test]
    fn test_into_pretty() {
        let json = r#"{
            "previousClose": {"fmt": "275.00", "raw": 275.0},
            "marketCap": {"fmt": "4.09T", "longFmt": "4,090,000,000,000", "raw": 4090000000000}
        }"#;

        let detail: SummaryDetail = serde_json::from_str(json).unwrap();
        let pretty = detail.into_pretty();
        assert_eq!(pretty.previous_close.as_deref(), Some("275.00"));
        assert_eq!(pretty.market_cap.as_deref(), Some("4.09T"));
    }
}
