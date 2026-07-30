use serde::{Deserialize, Serialize};

use super::Interval;

/// Time ranges for chart data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TimeRange {
    /// 1 day
    #[serde(rename = "1d")]
    OneDay,
    /// 5 days
    #[serde(rename = "5d")]
    FiveDays,
    /// 1 month
    #[serde(rename = "1mo")]
    OneMonth,
    /// 3 months
    #[serde(rename = "3mo")]
    ThreeMonths,
    /// 6 months
    #[serde(rename = "6mo")]
    SixMonths,
    /// 1 year
    #[serde(rename = "1y")]
    OneYear,
    /// 2 years
    #[serde(rename = "2y")]
    TwoYears,
    /// 5 years
    #[serde(rename = "5y")]
    FiveYears,
    /// 10 years
    #[serde(rename = "10y")]
    TenYears,
    /// Year to date
    #[serde(rename = "ytd")]
    YearToDate,
    /// Maximum available
    #[serde(rename = "max")]
    Max,
}

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

    /// A sensible default candle interval for this range, used by the
    /// `history(range)` convenience on domain handles: finer granularity for
    /// short ranges, coarser for long ones.
    pub fn default_interval(&self) -> Interval {
        match self {
            TimeRange::OneDay => Interval::FiveMinutes,
            TimeRange::FiveDays => Interval::FifteenMinutes,
            TimeRange::OneMonth
            | TimeRange::ThreeMonths
            | TimeRange::SixMonths
            | TimeRange::OneYear
            | TimeRange::YearToDate => Interval::OneDay,
            TimeRange::TwoYears | TimeRange::FiveYears => Interval::OneWeek,
            TimeRange::TenYears | TimeRange::Max => Interval::OneMonth,
        }
    }

    /// Approximate span of this range in seconds.
    ///
    /// Calendar approximations: a month is 30 days, a year 365. `YearToDate` is
    /// approximated as one year and `Max` as a far-future horizon.
    pub const fn approx_duration_secs(&self) -> i64 {
        const DAY: i64 = 86_400;
        match self {
            TimeRange::OneDay => DAY,
            TimeRange::FiveDays => 5 * DAY,
            TimeRange::OneMonth => 30 * DAY,
            TimeRange::ThreeMonths => 90 * DAY,
            TimeRange::SixMonths => 180 * DAY,
            TimeRange::OneYear | TimeRange::YearToDate => 365 * DAY,
            TimeRange::TwoYears => 730 * DAY,
            TimeRange::FiveYears => 1_825 * DAY,
            TimeRange::TenYears => 3_650 * DAY,
            TimeRange::Max => 36_500 * DAY,
        }
    }
}

impl std::fmt::Display for TimeRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for TimeRange {
    type Err = ();

    /// Parses the same short codes returned by [`TimeRange::as_str`] (e.g. `"3mo"`,
    /// `"ytd"`), case-insensitively. `"1wk"` is also accepted as an alias for
    /// [`TimeRange::FiveDays`] (a trading week).
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "1d" => Ok(TimeRange::OneDay),
            "5d" | "1wk" => Ok(TimeRange::FiveDays),
            "1mo" => Ok(TimeRange::OneMonth),
            "3mo" => Ok(TimeRange::ThreeMonths),
            "6mo" => Ok(TimeRange::SixMonths),
            "1y" => Ok(TimeRange::OneYear),
            "2y" => Ok(TimeRange::TwoYears),
            "5y" => Ok(TimeRange::FiveYears),
            "10y" => Ok(TimeRange::TenYears),
            "ytd" => Ok(TimeRange::YearToDate),
            "max" => Ok(TimeRange::Max),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_range_as_str() {
        assert_eq!(TimeRange::OneDay.as_str(), "1d");
        assert_eq!(TimeRange::OneMonth.as_str(), "1mo");
        assert_eq!(TimeRange::OneYear.as_str(), "1y");
        assert_eq!(TimeRange::Max.as_str(), "max");
    }

    #[test]
    fn test_time_range_from_str_round_trips_as_str() {
        for range in [
            TimeRange::OneDay,
            TimeRange::FiveDays,
            TimeRange::OneMonth,
            TimeRange::ThreeMonths,
            TimeRange::SixMonths,
            TimeRange::OneYear,
            TimeRange::TwoYears,
            TimeRange::FiveYears,
            TimeRange::TenYears,
            TimeRange::YearToDate,
            TimeRange::Max,
        ] {
            assert_eq!(range.as_str().parse(), Ok(range));
        }
        assert_eq!("1wk".parse(), Ok(TimeRange::FiveDays));
        assert_eq!("YTD".parse(), Ok(TimeRange::YearToDate));
        assert_eq!("bogus".parse::<TimeRange>(), Err(()));
    }

    #[test]
    fn test_default_interval_buckets() {
        // Intraday ranges → sub-day candles; long ranges → coarse candles.
        assert_eq!(TimeRange::OneDay.default_interval(), Interval::FiveMinutes);
        assert_eq!(
            TimeRange::FiveDays.default_interval(),
            Interval::FifteenMinutes
        );
        assert_eq!(TimeRange::OneMonth.default_interval(), Interval::OneDay);
        assert_eq!(TimeRange::OneYear.default_interval(), Interval::OneDay);
        assert_eq!(TimeRange::TwoYears.default_interval(), Interval::OneWeek);
        assert_eq!(TimeRange::Max.default_interval(), Interval::OneMonth);
    }
}
