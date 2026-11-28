//! Stock-directory date validation shared by the adapters and the bridges.

use crate::error::{FinanceError, Result};

/// Reject anything that is not a real `YYYY-MM-DD` calendar date.
pub fn date(value: &str) -> Result<()> {
    if chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .is_ok_and(|d| d.format("%Y-%m-%d").to_string() == value)
    {
        Ok(())
    } else {
        Err(FinanceError::InvalidParameter {
            param: "date".into(),
            reason: "expected a valid YYYY-MM-DD date".into(),
        })
    }
}
