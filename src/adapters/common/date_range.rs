//! `YYYY-MM-DD` date-range parsing shared by calendar-style adapters.

use chrono::NaiveDate;

use crate::error::{FinanceError, Result};

/// Parse and validate a `from`/`to` date-range pair as `YYYY-MM-DD`.
pub(crate) fn parse_date_range(from: &str, to: &str) -> Result<(NaiveDate, NaiveDate)> {
    let start = NaiveDate::parse_from_str(from, "%Y-%m-%d").map_err(|_| {
        FinanceError::InvalidParameter {
            param: "from".to_string(),
            reason: format!("expected YYYY-MM-DD, got {from:?}"),
        }
    })?;
    let end =
        NaiveDate::parse_from_str(to, "%Y-%m-%d").map_err(|_| FinanceError::InvalidParameter {
            param: "to".to_string(),
            reason: format!("expected YYYY-MM-DD, got {to:?}"),
        })?;
    if start > end {
        return Err(FinanceError::InvalidParameter {
            param: "to".to_string(),
            reason: "must not be before `from`".to_string(),
        });
    }
    Ok((start, end))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_range_parses() {
        let (start, end) = parse_date_range("2026-01-01", "2026-01-31").unwrap();
        assert_eq!(start, NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
        assert_eq!(end, NaiveDate::from_ymd_opt(2026, 1, 31).unwrap());
    }

    #[test]
    fn malformed_from_errors() {
        assert!(parse_date_range("not-a-date", "2026-01-31").is_err());
    }

    #[test]
    fn malformed_to_errors() {
        assert!(parse_date_range("2026-01-01", "not-a-date").is_err());
    }

    #[test]
    fn inverted_range_errors() {
        assert!(parse_date_range("2026-01-31", "2026-01-01").is_err());
    }
}
