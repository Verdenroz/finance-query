use serde::{Deserialize, Serialize};

/// Chart intervals
///
/// The `alias`es mirror the spellings [`FromStr`](std::str::FromStr) accepts, so
/// deserializing (axum query extraction, JSON) takes the same spellings parsing does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Interval {
    /// 1 minute
    #[serde(rename = "1m")]
    OneMinute,
    /// 2 minutes
    #[serde(rename = "2m")]
    TwoMinutes,
    /// 5 minutes
    #[serde(rename = "5m")]
    FiveMinutes,
    /// 15 minutes
    #[serde(rename = "15m")]
    FifteenMinutes,
    /// 30 minutes
    #[serde(rename = "30m")]
    ThirtyMinutes,
    /// 1 hour
    #[serde(rename = "1h", alias = "60m")]
    OneHour,
    /// 90 minutes
    #[serde(rename = "90m")]
    NinetyMinutes,
    /// 1 day
    #[serde(rename = "1d")]
    OneDay,
    /// 5 days
    #[serde(rename = "5d")]
    FiveDays,
    /// 1 week
    #[serde(rename = "1wk")]
    OneWeek,
    /// 1 month
    #[serde(rename = "1mo")]
    OneMonth,
    /// 3 months
    #[serde(rename = "3mo")]
    ThreeMonths,
}

impl Interval {
    /// Convert interval to Yahoo Finance API format
    pub fn as_str(&self) -> &'static str {
        match self {
            Interval::OneMinute => "1m",
            Interval::TwoMinutes => "2m",
            Interval::FiveMinutes => "5m",
            Interval::FifteenMinutes => "15m",
            Interval::ThirtyMinutes => "30m",
            Interval::OneHour => "1h",
            Interval::NinetyMinutes => "90m",
            Interval::OneDay => "1d",
            Interval::FiveDays => "5d",
            Interval::OneWeek => "1wk",
            Interval::OneMonth => "1mo",
            Interval::ThreeMonths => "3mo",
        }
    }

    /// Bars of this interval in one trading year, for annualising returns and
    /// prorating financing rates.
    ///
    /// Intraday counts assume a 6.5-hour US session over 252 trading days;
    /// daily and coarser follow the calendar.
    #[cfg(feature = "backtesting")]
    pub const fn bars_per_year(self) -> f64 {
        match self {
            Interval::OneMinute => 98_280.0,
            Interval::TwoMinutes => 49_140.0,
            Interval::FiveMinutes => 19_656.0,
            Interval::FifteenMinutes => 6_552.0,
            Interval::ThirtyMinutes => 3_276.0,
            Interval::OneHour => 1_638.0,
            Interval::NinetyMinutes => 1_092.0,
            Interval::OneDay => 252.0,
            Interval::FiveDays => 50.4,
            Interval::OneWeek => 52.0,
            Interval::OneMonth => 12.0,
            Interval::ThreeMonths => 4.0,
        }
    }

    /// Approximate span one candle of this interval covers, in seconds.
    ///
    /// Calendar approximations match [`TimeRange::approx_duration_secs`](super::TimeRange::approx_duration_secs): a
    /// month is 30 days, so a quarter is 90.
    #[cfg(any(feature = "backtesting", feature = "binance", feature = "kraken"))]
    pub(crate) const fn duration_secs(self) -> i64 {
        match self {
            Interval::OneMinute => 60,
            Interval::TwoMinutes => 120,
            Interval::FiveMinutes => 300,
            Interval::FifteenMinutes => 900,
            Interval::ThirtyMinutes => 1_800,
            Interval::OneHour => 3_600,
            Interval::NinetyMinutes => 5_400,
            Interval::OneDay => 86_400,
            Interval::FiveDays => 432_000,
            Interval::OneWeek => 604_800,
            Interval::OneMonth => 2_592_000,
            Interval::ThreeMonths => 7_776_000,
        }
    }
}

impl std::fmt::Display for Interval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for Interval {
    type Err = ();

    /// Parses the same short codes returned by [`Interval::as_str`] (e.g. `"1d"`,
    /// `"1wk"`), case-insensitively. `"60m"` is also accepted as an alias for
    /// [`Interval::OneHour`] (Yahoo itself normalizes `60m` to `1h`).
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "1m" => Ok(Interval::OneMinute),
            "2m" => Ok(Interval::TwoMinutes),
            "5m" => Ok(Interval::FiveMinutes),
            "15m" => Ok(Interval::FifteenMinutes),
            "30m" => Ok(Interval::ThirtyMinutes),
            "1h" | "60m" => Ok(Interval::OneHour),
            "90m" => Ok(Interval::NinetyMinutes),
            "1d" => Ok(Interval::OneDay),
            "5d" => Ok(Interval::FiveDays),
            "1wk" => Ok(Interval::OneWeek),
            "1mo" => Ok(Interval::OneMonth),
            "3mo" => Ok(Interval::ThreeMonths),
            _ => Err(()),
        }
    }
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
    fn test_interval_from_str_round_trips_as_str() {
        for interval in [
            Interval::OneMinute,
            Interval::TwoMinutes,
            Interval::FiveMinutes,
            Interval::FifteenMinutes,
            Interval::ThirtyMinutes,
            Interval::OneHour,
            Interval::NinetyMinutes,
            Interval::OneDay,
            Interval::FiveDays,
            Interval::OneWeek,
            Interval::OneMonth,
            Interval::ThreeMonths,
        ] {
            assert_eq!(interval.as_str().parse(), Ok(interval));
        }
        assert_eq!("1D".parse(), Ok(Interval::OneDay));
        assert_eq!(" 1d ".parse(), Ok(Interval::OneDay));
        assert_eq!("bogus".parse::<Interval>(), Err(()));
    }

    #[cfg(any(feature = "backtesting", feature = "binance", feature = "kraken"))]
    #[test]
    fn test_interval_duration_secs() {
        use super::super::TimeRange;

        assert_eq!(Interval::OneMinute.duration_secs(), 60);
        assert_eq!(Interval::OneHour.duration_secs(), 3_600);
        assert_eq!(Interval::OneDay.duration_secs(), 86_400);
        assert_eq!(Interval::OneWeek.duration_secs(), 604_800);
        // Calendar approximations agree with TimeRange's: 30-day months.
        assert_eq!(
            Interval::OneMonth.duration_secs(),
            TimeRange::OneMonth.approx_duration_secs()
        );
        assert_eq!(
            Interval::ThreeMonths.duration_secs(),
            TimeRange::ThreeMonths.approx_duration_secs()
        );
    }
}
