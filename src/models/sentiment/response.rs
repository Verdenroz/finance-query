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

impl std::fmt::Display for FearGreedLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The current CNN Fear & Greed Index reading from Alternative.me.
///
/// Scale: 0 (Extreme Fear) → 100 (Extreme Greed).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
#[cfg_attr(feature = "dataframe", dataframe(crate = "crate"))]
#[non_exhaustive]
pub struct FearAndGreed {
    /// Index value (0–100)
    pub value: u8,
    /// Human-readable classification of the value
    pub classification: FearGreedLabel,
    /// Unix timestamp (seconds) when this reading was recorded
    #[cfg_attr(feature = "dataframe", dataframe(datetime = "s"))]
    pub timestamp: i64,
}
