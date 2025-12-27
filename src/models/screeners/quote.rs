use crate::models::quote::FormattedValue;
use serde::{Deserialize, Serialize};

/// Quote data from a Yahoo Finance screener
///
/// This struct contains the fields returned by Yahoo Finance's predefined
/// screener endpoint. It includes comprehensive quote data for filtering
/// and displaying screened stocks/funds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenerQuote {
    // Core identification
    /// Stock symbol (e.g., "NVDA")
    pub symbol: String,
    /// Short display name (e.g., "NVIDIA Corporation")
    pub short_name: String,
    /// Full company name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub long_name: Option<String>,
    /// Display name for UI purposes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Type of quote (e.g., "EQUITY", "ETF", "CRYPTOCURRENCY")
    pub quote_type: String,
    /// Exchange code (e.g., "NMS" for NASDAQ)
    pub exchange: String,

    // Price information
    /// Current regular market price
    pub regular_market_price: FormattedValue<f64>,
    /// Change in price from previous close
    pub regular_market_change: FormattedValue<f64>,
    /// Percent change from previous close
    pub regular_market_change_percent: FormattedValue<f64>,
    /// Regular market open price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regular_market_open: Option<FormattedValue<f64>>,
    /// Day's high price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regular_market_day_high: Option<FormattedValue<f64>>,
    /// Day's low price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regular_market_day_low: Option<FormattedValue<f64>>,
    /// Previous close price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regular_market_previous_close: Option<FormattedValue<f64>>,
    /// Regular market timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regular_market_time: Option<FormattedValue<i64>>,

    // Volume & Market Cap
    /// Regular market volume (may be None for funds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regular_market_volume: Option<FormattedValue<i64>>,
    /// Average daily volume over 3 months
    #[serde(skip_serializing_if = "Option::is_none")]
    pub average_daily_volume3_month: Option<FormattedValue<i64>>,
    /// Average daily volume over 10 days
    #[serde(skip_serializing_if = "Option::is_none")]
    pub average_daily_volume10_day: Option<FormattedValue<i64>>,
    /// Market capitalization
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_cap: Option<FormattedValue<i64>>,
    /// Outstanding shares
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shares_outstanding: Option<FormattedValue<i64>>,

    // 52-Week Range
    /// 52-week high price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fifty_two_week_high: Option<FormattedValue<f64>>,
    /// 52-week low price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fifty_two_week_low: Option<FormattedValue<f64>>,
    /// Change from 52-week low
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fifty_two_week_change: Option<FormattedValue<f64>>,
    /// Percent change from 52-week low
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fifty_two_week_change_percent: Option<FormattedValue<f64>>,

    // Moving Averages
    /// 50-day moving average
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fifty_day_average: Option<FormattedValue<f64>>,
    /// Change from 50-day average
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fifty_day_average_change: Option<FormattedValue<f64>>,
    /// Percent change from 50-day average
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fifty_day_average_change_percent: Option<FormattedValue<f64>>,
    /// 200-day moving average
    #[serde(skip_serializing_if = "Option::is_none")]
    pub two_hundred_day_average: Option<FormattedValue<f64>>,
    /// Change from 200-day average
    #[serde(skip_serializing_if = "Option::is_none")]
    pub two_hundred_day_average_change: Option<FormattedValue<f64>>,
    /// Percent change from 200-day average
    #[serde(skip_serializing_if = "Option::is_none")]
    pub two_hundred_day_average_change_percent: Option<FormattedValue<f64>>,

    // Valuation Metrics
    /// Average analyst rating (e.g., "1.3 - Strong Buy")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub average_analyst_rating: Option<String>,
    /// Trailing 12-month P/E ratio
    #[serde(rename = "trailingPE", skip_serializing_if = "Option::is_none")]
    pub trailing_pe: Option<FormattedValue<f64>>,
    /// Forward P/E ratio
    #[serde(rename = "forwardPE", skip_serializing_if = "Option::is_none")]
    pub forward_pe: Option<FormattedValue<f64>>,
    /// Price to book ratio
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_to_book: Option<FormattedValue<f64>>,
    /// Book value per share
    #[serde(skip_serializing_if = "Option::is_none")]
    pub book_value: Option<FormattedValue<f64>>,

    // Earnings Per Share
    /// Trailing 12-month EPS
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eps_trailing_twelve_months: Option<FormattedValue<f64>>,
    /// Forward EPS
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eps_forward: Option<FormattedValue<f64>>,
    /// Current year EPS
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eps_current_year: Option<FormattedValue<f64>>,
    /// Price to current year EPS
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_eps_current_year: Option<FormattedValue<f64>>,

    // Dividend Information
    /// Dividend yield
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dividend_yield: Option<FormattedValue<f64>>,
    /// Dividend rate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dividend_rate: Option<FormattedValue<f64>>,
    /// Ex-dividend date timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dividend_date: Option<FormattedValue<i64>>,
    /// Trailing annual dividend rate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trailing_annual_dividend_rate: Option<FormattedValue<f64>>,
    /// Trailing annual dividend yield
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trailing_annual_dividend_yield: Option<FormattedValue<f64>>,

    // Bid/Ask
    /// Current bid price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bid: Option<FormattedValue<f64>>,
    /// Bid size
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bid_size: Option<FormattedValue<i64>>,
    /// Current ask price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ask: Option<FormattedValue<f64>>,
    /// Ask size
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ask_size: Option<FormattedValue<i64>>,

    // Post/Pre Market (Optional)
    /// Post-market price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_market_price: Option<FormattedValue<f64>>,
    /// Post-market price change
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_market_change: Option<FormattedValue<f64>>,
    /// Post-market percent change
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_market_change_percent: Option<FormattedValue<f64>>,
    /// Post-market timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_market_time: Option<FormattedValue<i64>>,
    /// Pre-market price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_market_price: Option<FormattedValue<f64>>,
    /// Pre-market price change
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_market_change: Option<FormattedValue<f64>>,
    /// Pre-market percent change
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_market_change_percent: Option<FormattedValue<f64>>,
    /// Pre-market timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_market_time: Option<FormattedValue<i64>>,

    // Earnings Dates
    /// Earnings timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub earnings_timestamp: Option<FormattedValue<i64>>,
    /// Earnings timestamp start
    #[serde(skip_serializing_if = "Option::is_none")]
    pub earnings_timestamp_start: Option<FormattedValue<i64>>,
    /// Earnings timestamp end
    #[serde(skip_serializing_if = "Option::is_none")]
    pub earnings_timestamp_end: Option<FormattedValue<i64>>,

    // Additional Fields
    /// Currency code (e.g., "USD")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
}
