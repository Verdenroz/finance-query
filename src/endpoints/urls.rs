//! Yahoo Finance API URLs and endpoint builders.
//!
//! Internal module for constructing Yahoo Finance API URLs.

use crate::constants::screeners::Screener;

/// Yahoo Finance API base URLs
pub mod base {
    /// Base URL for Yahoo Finance API (query1)
    pub const YAHOO_FINANCE_QUERY1: &str = "https://query1.finance.yahoo.com";

    /// Base URL for Yahoo Finance API (query2)
    pub const YAHOO_FINANCE_QUERY2: &str = "https://query2.finance.yahoo.com";

    /// Yahoo authentication/cookie page
    pub const YAHOO_FC: &str = "https://fc.yahoo.com";
}

/// Yahoo Finance API endpoint paths
pub mod api {
    use super::base::*;

    /// Get crumb token (query1)
    pub const CRUMB_QUERY1: &str =
        const_format::concatcp!(YAHOO_FINANCE_QUERY1, "/v1/test/getcrumb");

    /// Quote summary endpoint (detailed quote data)
    pub fn quote_summary(symbol: &str) -> String {
        format!(
            "{}/v10/finance/quoteSummary/{}",
            YAHOO_FINANCE_QUERY2, symbol
        )
    }

    /// Batch quotes endpoint - fetch multiple symbols in one request
    pub const QUOTES: &str = const_format::concatcp!(YAHOO_FINANCE_QUERY1, "/v7/finance/quote");

    /// Historical chart data endpoint
    #[allow(dead_code)]
    pub fn chart(symbol: &str) -> String {
        format!("{}/v8/finance/chart/{}", YAHOO_FINANCE_QUERY1, symbol)
    }

    /// Search endpoint
    #[allow(dead_code)]
    pub const SEARCH: &str = const_format::concatcp!(YAHOO_FINANCE_QUERY1, "/v1/finance/search");

    /// Lookup endpoint (type-filtered symbol discovery)
    #[allow(dead_code)]
    pub const LOOKUP: &str = const_format::concatcp!(YAHOO_FINANCE_QUERY1, "/v1/finance/lookup");

    /// Financial timeseries endpoint (financials)
    #[allow(dead_code)]
    pub fn financials(symbol: &str) -> String {
        format!(
            "{}/ws/fundamentals-timeseries/v1/finance/timeseries/{}",
            YAHOO_FINANCE_QUERY2, symbol
        )
    }

    /// Recommendations endpoint (similar stocks)
    #[allow(dead_code)]
    pub fn recommendations(symbol: &str) -> String {
        format!(
            "{}/v6/finance/recommendationsbysymbol/{}",
            YAHOO_FINANCE_QUERY2, symbol
        )
    }

    /// Quote type endpoint (get quartr ID)
    #[allow(dead_code)]
    pub fn quote_type(symbol: &str) -> String {
        format!("{}/v1/finance/quoteType/{}", YAHOO_FINANCE_QUERY1, symbol)
    }

    /// News endpoint
    #[allow(dead_code)]
    pub const NEWS: &str = const_format::concatcp!(YAHOO_FINANCE_QUERY2, "/v2/finance/news");

    /// Options endpoint
    #[allow(dead_code)]
    pub fn options(symbol: &str) -> String {
        format!("{}/v7/finance/options/{}", YAHOO_FINANCE_QUERY2, symbol)
    }

    /// Market hours/time endpoint
    pub const MARKET_TIME: &str =
        const_format::concatcp!(YAHOO_FINANCE_QUERY1, "/v6/finance/markettime");

    /// Currencies endpoint
    pub const CURRENCIES: &str =
        const_format::concatcp!(YAHOO_FINANCE_QUERY2, "/v1/finance/currencies");

    /// Market summary endpoint
    pub const MARKET_SUMMARY: &str =
        const_format::concatcp!(YAHOO_FINANCE_QUERY2, "/v6/finance/quote/marketSummary");

    /// Trending tickers endpoint (requires region suffix)
    pub fn trending(region: &str) -> String {
        format!("{}/v1/finance/trending/{}", YAHOO_FINANCE_QUERY2, region)
    }

    /// Batch sparkline data endpoint
    pub const SPARK: &str = const_format::concatcp!(YAHOO_FINANCE_QUERY1, "/v7/finance/spark");
}

/// URL builders (functions that construct full URLs with query params)
pub mod builders {
    use super::Screener;
    use super::base::*;

    /// Screener endpoint for predefined screeners
    pub fn screener(screener_type: Screener, count: u32) -> String {
        format!(
            "{}/v1/finance/screener/predefined/saved?count={}&formatted=true&scrIds={}",
            YAHOO_FINANCE_QUERY1,
            count,
            screener_type.as_scr_id()
        )
    }

    /// Custom screener endpoint (POST)
    pub fn custom_screener() -> String {
        format!(
            "{}/v1/finance/screener?formatted=true&useRecordsResponse=true&lang=en-US&region=US",
            YAHOO_FINANCE_QUERY1
        )
    }

    /// Sector details endpoint
    pub fn sector(sector_key: &str) -> String {
        format!(
            "{}/v1/finance/sectors/{}?formatted=true&withReturns=false&lang=en-US&region=US",
            YAHOO_FINANCE_QUERY1, sector_key
        )
    }

    /// Industries endpoint - detailed industry data
    pub fn industry(industry_key: &str) -> String {
        format!(
            "{}/v1/finance/industries/{}?formatted=true&withReturns=false&lang=en-US&region=US",
            YAHOO_FINANCE_QUERY1, industry_key
        )
    }
}
