//! Fear & Greed Index Response Model
//!
//! Represents market sentiment from the Alternative.me Fear & Greed Index.

use serde::{Deserialize, Serialize};

/// Classification label for the Fear & Greed Index value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum FearGreedLabel {
    /// 0–24: Extreme Fear
    #[serde(rename = "Extreme Fear")]
    ExtremeFear,
    /// 25–44: Fear
    Fear,
    /// 45–55: Neutral
    Neutral,
    /// 56–75: Greed
    Greed,
    /// 76–100: Extreme Greed
    #[serde(rename = "Extreme Greed")]
    ExtremeGreed,
}

impl FearGreedLabel {
    /// Returns a human-readable string for the label.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ExtremeFear => "Extreme Fear",
            Self::Fear => "Fear",
            Self::Neutral => "Neutral",
            Self::Greed => "Greed",
            Self::ExtremeGreed => "Extreme Greed",
        }
    }
}

/// The current CNN Fear & Greed Index reading from Alternative.me.
///
/// Scale: 0 (Extreme Fear) → 100 (Extreme Greed).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FearAndGreed {
    /// Index value (0–100)
    pub value: u8,
    /// Human-readable classification of the value
    pub classification: FearGreedLabel,
    /// Unix timestamp (seconds) when this reading was recorded
    pub timestamp: i64,
}

// ---- Internal deserialization wrappers (not public) ----

#[derive(Debug, Deserialize)]
pub(crate) struct FearAndGreedApiResponse {
    pub data: Vec<FearAndGreedEntry>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct FearAndGreedEntry {
    pub value: String,
    pub value_classification: String,
    pub timestamp: String,
}

impl FearAndGreed {
    pub(crate) fn from_response(
        resp: FearAndGreedApiResponse,
    ) -> Result<Self, crate::error::FinanceError> {
        let entry = resp.data.into_iter().next().ok_or_else(|| {
            crate::error::FinanceError::ResponseStructureError {
                field: "data".to_string(),
                context: "Alternative.me API returned empty data array".to_string(),
            }
        })?;

        let value = entry.value.parse::<u8>().map_err(|_| {
            crate::error::FinanceError::ResponseStructureError {
                field: "value".to_string(),
                context: format!("Cannot parse '{}' as u8", entry.value),
            }
        })?;

        let classification = parse_classification(&entry.value_classification)?;

        let timestamp = entry.timestamp.parse::<i64>().map_err(|_| {
            crate::error::FinanceError::ResponseStructureError {
                field: "timestamp".to_string(),
                context: format!("Cannot parse '{}' as i64", entry.timestamp),
            }
        })?;

        Ok(Self {
            value,
            classification,
            timestamp,
        })
    }
}

pub(crate) fn parse_classification(s: &str) -> Result<FearGreedLabel, crate::error::FinanceError> {
    match s {
        "Extreme Fear" => Ok(FearGreedLabel::ExtremeFear),
        "Fear" => Ok(FearGreedLabel::Fear),
        "Neutral" => Ok(FearGreedLabel::Neutral),
        "Greed" => Ok(FearGreedLabel::Greed),
        "Extreme Greed" => Ok(FearGreedLabel::ExtremeGreed),
        other => Err(crate::error::FinanceError::ResponseStructureError {
            field: "value_classification".to_string(),
            context: format!("Unknown classification '{other}'"),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_classification() {
        assert_eq!(
            parse_classification("Extreme Fear").unwrap(),
            FearGreedLabel::ExtremeFear
        );
        assert_eq!(parse_classification("Fear").unwrap(), FearGreedLabel::Fear);
        assert_eq!(
            parse_classification("Neutral").unwrap(),
            FearGreedLabel::Neutral
        );
        assert_eq!(
            parse_classification("Greed").unwrap(),
            FearGreedLabel::Greed
        );
        assert_eq!(
            parse_classification("Extreme Greed").unwrap(),
            FearGreedLabel::ExtremeGreed
        );
        assert!(parse_classification("unknown").is_err());
    }

    #[test]
    fn test_fear_greed_from_response() {
        let resp = FearAndGreedApiResponse {
            data: vec![FearAndGreedEntry {
                value: "25".to_string(),
                value_classification: "Fear".to_string(),
                timestamp: "1700000000".to_string(),
            }],
        };
        let fg = FearAndGreed::from_response(resp).unwrap();
        assert_eq!(fg.value, 25);
        assert_eq!(fg.classification, FearGreedLabel::Fear);
        assert_eq!(fg.timestamp, 1700000000);
    }

    #[test]
    fn test_empty_data_returns_error() {
        let resp = FearAndGreedApiResponse { data: vec![] };
        assert!(FearAndGreed::from_response(resp).is_err());
    }
}
