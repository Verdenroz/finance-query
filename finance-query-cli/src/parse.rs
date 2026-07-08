//! Shared string-to-enum parsing for CLI arguments, backed by the library's
//! `FromStr` impls so every command validates the same set of interval/range
//! codes instead of each hand-rolling its own match arms.

use crate::error::{CliError, Result};
use finance_query::{Interval, TimeRange};

pub fn parse_interval(s: &str) -> Result<Interval> {
    s.parse().map_err(|_| {
        CliError::InvalidArgument(format!(
            "Invalid interval '{s}'. Valid: 1m, 5m, 15m, 30m, 1h, 1d, 1wk, 1mo, 3mo"
        ))
    })
}

pub fn parse_range(s: &str) -> Result<TimeRange> {
    s.parse().map_err(|_| {
        CliError::InvalidArgument(format!(
            "Invalid range '{s}'. Valid: 1d, 5d/1wk, 1mo, 3mo, 6mo, 1y, 2y, 5y, 10y, ytd, max"
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_interval_accepts_every_variant_including_30m_and_3mo() {
        // Regression check: some per-command copies of this parser used to
        // omit 30m/3mo even though `Interval` supports them.
        assert_eq!(parse_interval("30m").unwrap(), Interval::ThirtyMinutes);
        assert_eq!(parse_interval("3mo").unwrap(), Interval::ThreeMonths);
        assert_eq!(parse_interval("1D").unwrap(), Interval::OneDay);
    }

    #[test]
    fn parse_interval_rejects_unknown_code() {
        assert!(parse_interval("bogus").is_err());
    }

    #[test]
    fn parse_range_accepts_1wk_alias_for_five_days() {
        assert_eq!(parse_range("1wk").unwrap(), TimeRange::FiveDays);
        assert_eq!(parse_range("YTD").unwrap(), TimeRange::YearToDate);
    }

    #[test]
    fn parse_range_rejects_unknown_code() {
        assert!(parse_range("bogus").is_err());
    }
}
