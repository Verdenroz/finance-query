//! Yahoo Finance API URLs and endpoint builders.
//!
//! Internal module for constructing Yahoo Finance API URLs.

use crate::constants::screeners::Screener;
use crate::error::{FinanceError, Result};
use std::borrow::Cow;

const HOSTS: [&str; 4] = [
    base::YAHOO_FINANCE_QUERY1,
    base::YAHOO_FINANCE_QUERY2,
    base::YAHOO_FC,
    "https://finance.yahoo.com",
];

/// Swap the Yahoo host in `url` for `origin`, leaving the URL untouched when
/// there is no origin. A host outside the known set is refused so an override
/// never lets a request fall through to live Yahoo.
pub(crate) fn resolve<'a>(origin: Option<&str>, url: &'a str) -> Result<Cow<'a, str>> {
    let Some(origin) = origin else {
        return Ok(Cow::Borrowed(url));
    };
    HOSTS
        .iter()
        .find_map(|host| url.strip_prefix(host))
        .filter(|rest| matches!(rest.chars().next(), None | Some('/' | '?')))
        .map(|rest| Cow::Owned(format!("{origin}{rest}")))
        .ok_or_else(|| FinanceError::InvalidParameter {
            param: "endpoint".into(),
            reason: format!("`{url}` is not a Yahoo URL the endpoint override can serve"),
        })
}

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
    pub fn chart(symbol: &str) -> String {
        format!("{}/v8/finance/chart/{}", YAHOO_FINANCE_QUERY1, symbol)
    }

    /// Search endpoint
    pub const SEARCH: &str = const_format::concatcp!(YAHOO_FINANCE_QUERY1, "/v1/finance/search");

    /// Lookup endpoint (type-filtered symbol discovery)
    pub const LOOKUP: &str = const_format::concatcp!(YAHOO_FINANCE_QUERY1, "/v1/finance/lookup");

    /// Financial timeseries endpoint (financials)
    pub fn financials(symbol: &str) -> String {
        format!(
            "{}/ws/fundamentals-timeseries/v1/finance/timeseries/{}",
            YAHOO_FINANCE_QUERY2, symbol
        )
    }

    /// Recommendations endpoint (similar stocks)
    pub fn recommendations(symbol: &str) -> String {
        format!(
            "{}/v6/finance/recommendationsbysymbol/{}",
            YAHOO_FINANCE_QUERY2, symbol
        )
    }

    /// Quote type endpoint (get quartr ID)
    pub fn quote_type(symbol: &str) -> String {
        format!("{}/v1/finance/quoteType/{}", YAHOO_FINANCE_QUERY1, symbol)
    }

    /// Options endpoint
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

#[cfg(test)]
mod tests {
    use super::*;

    const ORIGIN: &str = "http://127.0.0.1:9";

    #[test]
    fn no_origin_returns_every_url_unchanged() {
        let urls = [
            api::CRUMB_QUERY1.to_string(),
            api::quote_summary("AAPL"),
            api::QUOTES.to_string(),
            api::chart("AAPL"),
            api::SEARCH.to_string(),
            api::LOOKUP.to_string(),
            api::financials("AAPL"),
            api::recommendations("AAPL"),
            api::quote_type("AAPL"),
            api::options("AAPL"),
            api::MARKET_TIME.to_string(),
            api::CURRENCIES.to_string(),
            api::MARKET_SUMMARY.to_string(),
            api::trending("US"),
            api::SPARK.to_string(),
            builders::screener(Screener::DayGainers, 5),
            builders::custom_screener(),
            builders::sector("technology"),
            builders::industry("software"),
            base::YAHOO_FC.to_string(),
            "https://finance.yahoo.com/xhr/transcript".to_string(),
        ];
        for url in &urls {
            assert_eq!(resolve(None, url).unwrap(), url.as_str());
        }
    }

    #[test]
    fn every_known_host_swaps_to_the_origin() {
        assert_eq!(
            resolve(Some(ORIGIN), &api::chart("AAPL")).unwrap(),
            "http://127.0.0.1:9/v8/finance/chart/AAPL"
        );
        assert_eq!(
            resolve(Some(ORIGIN), &api::quote_summary("AAPL")).unwrap(),
            "http://127.0.0.1:9/v10/finance/quoteSummary/AAPL"
        );
        assert_eq!(resolve(Some(ORIGIN), base::YAHOO_FC).unwrap(), ORIGIN);
        assert_eq!(
            resolve(Some(ORIGIN), "https://finance.yahoo.com/xhr/transcript").unwrap(),
            "http://127.0.0.1:9/xhr/transcript"
        );
        assert_eq!(
            resolve(Some(ORIGIN), &builders::sector("energy")).unwrap(),
            "http://127.0.0.1:9/v1/finance/sectors/energy?formatted=true&withReturns=false&lang=en-US&region=US"
        );
    }

    #[test]
    fn an_unknown_host_is_an_error() {
        assert!(resolve(Some(ORIGIN), "https://example.com/v8/finance/chart/AAPL").is_err());
    }

    #[test]
    fn no_rewrite_across_a_non_boundary() {
        assert!(resolve(Some(ORIGIN), "https://query1.finance.yahoo.com.evil/x").is_err());
        assert!(resolve(Some(ORIGIN), "https://fc.yahoo.comx").is_err());
        assert!(resolve(Some(ORIGIN), "https://finance.yahoo.com@evil.test/").is_err());
    }
}
