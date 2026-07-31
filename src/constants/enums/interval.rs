use serde::{Deserialize, Serialize};

/// Chart intervals
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Interval {
    /// 1 minute
    #[serde(rename = "1m")]
    OneMinute,
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
    #[serde(rename = "1h")]
    OneHour,
    /// 1 day
    #[serde(rename = "1d")]
    OneDay,
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

impl std::fmt::Display for Interval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for Interval {
    type Err = ();

    /// Parses the same short codes returned by [`Interval::as_str`] (e.g. `"1d"`,
    /// `"1wk"`), case-insensitively.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "1m" => Ok(Interval::OneMinute),
            "5m" => Ok(Interval::FiveMinutes),
            "15m" => Ok(Interval::FifteenMinutes),
            "30m" => Ok(Interval::ThirtyMinutes),
            "1h" => Ok(Interval::OneHour),
            "1d" => Ok(Interval::OneDay),
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
            Interval::FiveMinutes,
            Interval::FifteenMinutes,
            Interval::ThirtyMinutes,
            Interval::OneHour,
            Interval::OneDay,
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
}
