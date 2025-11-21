/// Yahoo Finance API base URLs
pub mod urls {
    /// Base URL for Yahoo Finance API (query1)
    pub const YAHOO_FINANCE_QUERY1: &str = "https://query1.finance.yahoo.com";

    /// Base URL for Yahoo Finance API (query2)
    pub const YAHOO_FINANCE_QUERY2: &str = "https://query2.finance.yahoo.com";

    /// Yahoo authentication/cookie page
    pub const YAHOO_FC: &str = "https://fc.yahoo.com";
}

/// Yahoo Finance API endpoints
pub mod endpoints {
    use super::urls::*;

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

    /// Simple quotes endpoint (batch quotes)
    #[allow(dead_code)]
    pub const SIMPLE_QUOTES: &str =
        const_format::concatcp!(YAHOO_FINANCE_QUERY1, "/v7/finance/quote");

    /// Historical chart data endpoint
    #[allow(dead_code)]
    pub fn chart(symbol: &str) -> String {
        format!("{}/v8/finance/chart/{}", YAHOO_FINANCE_QUERY1, symbol)
    }

    /// Search endpoint
    #[allow(dead_code)]
    pub const SEARCH: &str = const_format::concatcp!(YAHOO_FINANCE_QUERY1, "/v1/finance/search");

    /// Financial timeseries endpoint (financials)
    #[allow(dead_code)]
    pub fn financials(symbol: &str) -> String {
        format!(
            "{}/ws/fundamentals-timeseries/v1/finance/timeseries/{}",
            YAHOO_FINANCE_QUERY1, symbol
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
}

/// HTTP headers
pub mod headers {
    /// User agent to use for requests (Chrome on Windows)
    pub const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

    /// Accept header
    #[allow(dead_code)]
    pub const ACCEPT: &str = "*/*";

    /// Accept language
    #[allow(dead_code)]
    pub const ACCEPT_LANGUAGE: &str = "en-US,en;q=0.9";

    /// Accept encoding
    #[allow(dead_code)]
    pub const ACCEPT_ENCODING: &str = "gzip, deflate, br";
}

/// Chart intervals
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Interval {
    /// 1 minute
    OneMinute,
    /// 5 minutes
    FiveMinutes,
    /// 15 minutes
    FifteenMinutes,
    /// 30 minutes
    ThirtyMinutes,
    /// 1 hour
    OneHour,
    /// 1 day
    OneDay,
    /// 1 week
    OneWeek,
    /// 1 month
    OneMonth,
    /// 3 months
    ThreeMonths,
}

#[allow(dead_code)]
impl Interval {
    /// Convert interval to Yahoo Finance API format
    pub fn as_str(&self) -> &'static str {
        match self {
            Interval::OneMinute => "1m",
            Interval::FiveMinutes => "5m",
            Interval::FifteenMinutes => "15m",
            Interval::ThirtyMinutes => "30m",
            Interval::OneHour => "1h",
            Interval::OneDay => "1d",
            Interval::OneWeek => "1wk",
            Interval::OneMonth => "1mo",
            Interval::ThreeMonths => "3mo",
        }
    }
}

/// Time ranges for chart data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum TimeRange {
    /// 1 day
    OneDay,
    /// 5 days
    FiveDays,
    /// 1 month
    OneMonth,
    /// 3 months
    ThreeMonths,
    /// 6 months
    SixMonths,
    /// 1 year
    OneYear,
    /// 2 years
    TwoYears,
    /// 5 years
    FiveYears,
    /// 10 years
    TenYears,
    /// Year to date
    YearToDate,
    /// Maximum available
    Max,
}

#[allow(dead_code)]
impl TimeRange {
    /// Convert time range to Yahoo Finance API format
    pub fn as_str(&self) -> &'static str {
        match self {
            TimeRange::OneDay => "1d",
            TimeRange::FiveDays => "5d",
            TimeRange::OneMonth => "1mo",
            TimeRange::ThreeMonths => "3mo",
            TimeRange::SixMonths => "6mo",
            TimeRange::OneYear => "1y",
            TimeRange::TwoYears => "2y",
            TimeRange::FiveYears => "5y",
            TimeRange::TenYears => "10y",
            TimeRange::YearToDate => "ytd",
            TimeRange::Max => "max",
        }
    }
}

/// Authentication constants
pub mod auth {
    use std::time::Duration;

    /// Minimum interval between auth refreshes (prevent excessive refreshing)
    #[allow(dead_code)]
    pub const MIN_REFRESH_INTERVAL: Duration = Duration::from_secs(30);

    /// Maximum age of auth before considering it stale
    #[allow(dead_code)]
    pub const AUTH_MAX_AGE: Duration = Duration::from_secs(3600); // 1 hour
}

/// Default timeouts
pub mod timeouts {
    use std::time::Duration;

    /// Default HTTP request timeout
    pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

    /// Timeout for authentication requests
    pub const AUTH_TIMEOUT: Duration = Duration::from_secs(15);
}

/// Default values for API endpoints
pub mod defaults {
    /// Default number of similar stocks to return
    pub const SIMILAR_STOCKS_LIMIT: u32 = 5;

    /// Default number of search results
    pub const SEARCH_HITS: u32 = 6;

    /// Default server port
    pub const SERVER_PORT: u16 = 8000;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interval_as_str() {
        assert_eq!(Interval::OneMinute.as_str(), "1m");
        assert_eq!(Interval::FiveMinutes.as_str(), "5m");
        assert_eq!(Interval::OneDay.as_str(), "1d");
        assert_eq!(Interval::OneWeek.as_str(), "1wk");
    }

    #[test]
    fn test_time_range_as_str() {
        assert_eq!(TimeRange::OneDay.as_str(), "1d");
        assert_eq!(TimeRange::OneMonth.as_str(), "1mo");
        assert_eq!(TimeRange::OneYear.as_str(), "1y");
        assert_eq!(TimeRange::Max.as_str(), "max");
    }

    #[test]
    fn test_endpoint_construction() {
        assert_eq!(
            endpoints::chart("AAPL"),
            "https://query1.finance.yahoo.com/v8/finance/chart/AAPL"
        );
        assert_eq!(
            endpoints::quote_summary("NVDA"),
            "https://query2.finance.yahoo.com/v10/finance/quoteSummary/NVDA"
        );
    }
}
