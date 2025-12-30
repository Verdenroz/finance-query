use serde::{Deserialize, Serialize};

use super::FormattedValue;

/// Calendar events including earnings and dividend dates
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarEvents {
    /// Maximum age of the data in seconds
    #[serde(default)]
    pub max_age: Option<i64>,

    /// Upcoming earnings date(s)
    #[serde(default)]
    pub earnings: Option<EarningsCalendar>,

    /// Ex-dividend date (Unix timestamp)
    #[serde(default)]
    pub ex_dividend_date: Option<FormattedValue<i64>>,

    /// Dividend date (Unix timestamp)
    #[serde(default)]
    pub dividend_date: Option<FormattedValue<i64>>,
}

/// Earnings calendar information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EarningsCalendar {
    /// List of earnings dates (usually contains 1-2 dates)
    #[serde(default)]
    pub earnings_date: Option<Vec<FormattedValue<i64>>>,

    /// Average earnings estimate for upcoming quarter
    #[serde(default)]
    pub earnings_average: Option<FormattedValue<f64>>,

    /// Low earnings estimate
    #[serde(default)]
    pub earnings_low: Option<FormattedValue<f64>>,

    /// High earnings estimate
    #[serde(default)]
    pub earnings_high: Option<FormattedValue<f64>>,

    /// Revenue estimate
    #[serde(default)]
    pub revenue_average: Option<FormattedValue<i64>>,

    /// Low revenue estimate
    #[serde(default)]
    pub revenue_low: Option<FormattedValue<i64>>,

    /// High revenue estimate
    #[serde(default)]
    pub revenue_high: Option<FormattedValue<i64>>,
}

impl CalendarEvents {
    /// Returns the next earnings date if available
    pub fn next_earnings_date(&self) -> Option<i64> {
        self.earnings.as_ref()?.earnings_date.as_ref()?.first()?.raw
    }

    /// Returns the ex-dividend date as Unix timestamp
    pub fn ex_dividend_timestamp(&self) -> Option<i64> {
        self.ex_dividend_date.as_ref()?.raw
    }

    /// Returns the dividend date as Unix timestamp
    pub fn dividend_timestamp(&self) -> Option<i64> {
        self.dividend_date.as_ref()?.raw
    }

    /// Returns the earnings estimate average
    pub fn earnings_estimate(&self) -> Option<f64> {
        self.earnings.as_ref()?.earnings_average.as_ref()?.raw
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_calendar_events_deserialize() {
        let json = json!({
            "maxAge": 1,
            "earnings": {
                "earningsDate": [
                    {"fmt": "2026-02-25", "raw": 1772053200}
                ],
                "earningsAverage": {"fmt": "1.52", "raw": 1.52083}
            },
            "exDividendDate": {"fmt": "2025-12-04", "raw": 1764806400},
            "dividendDate": {"fmt": "2025-12-20", "raw": 1766188800}
        });

        let events: CalendarEvents = serde_json::from_value(json).unwrap();
        assert_eq!(events.max_age, Some(1));
        assert_eq!(events.next_earnings_date(), Some(1772053200));
        assert_eq!(events.ex_dividend_timestamp(), Some(1764806400));
        assert_eq!(events.earnings_estimate(), Some(1.52083));
    }

    #[test]
    fn test_calendar_events_with_empty_fields() {
        let json = json!({
            "maxAge": 1,
            "earnings": {},
            "exDividendDate": {},
            "dividendDate": {}
        });

        let events: CalendarEvents = serde_json::from_value(json).unwrap();
        assert_eq!(events.max_age, Some(1));
        assert_eq!(events.next_earnings_date(), None);
        assert_eq!(events.ex_dividend_timestamp(), None);
    }
}
