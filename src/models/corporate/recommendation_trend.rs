//! Recommendation Trend Module
//!
//! Contains analyst recommendation trends over time.

use serde::{Deserialize, Serialize};

/// Analyst recommendation trends
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecommendationTrend {
    /// List of recommendation trends by period
    #[serde(default)]
    pub trend: Vec<RecommendationPeriod>,

    /// Maximum age of this data in seconds
    #[serde(default)]
    pub max_age: Option<i64>,
}

/// Recommendations for a specific time period
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecommendationPeriod {
    /// Time period (e.g., "0m", "-1m", "-2m")
    #[serde(default)]
    pub period: Option<String>,

    /// Number of strong buy recommendations
    #[serde(default)]
    pub strong_buy: Option<i32>,

    /// Number of buy recommendations
    #[serde(default)]
    pub buy: Option<i32>,

    /// Number of hold recommendations
    #[serde(default)]
    pub hold: Option<i32>,

    /// Number of sell recommendations
    #[serde(default)]
    pub sell: Option<i32>,

    /// Number of strong sell recommendations
    #[serde(default)]
    pub strong_sell: Option<i32>,
}

impl RecommendationPeriod {
    /// Calculate total number of analyst recommendations
    pub fn total(&self) -> i32 {
        self.strong_buy.unwrap_or(0)
            + self.buy.unwrap_or(0)
            + self.hold.unwrap_or(0)
            + self.sell.unwrap_or(0)
            + self.strong_sell.unwrap_or(0)
    }

    /// Calculate average recommendation score (1.0 = strong buy, 5.0 = strong sell)
    pub fn average_score(&self) -> Option<f64> {
        let total = self.total();
        if total == 0 {
            return None;
        }

        let score = (self.strong_buy.unwrap_or(0) as f64 * 1.0)
            + (self.buy.unwrap_or(0) as f64 * 2.0)
            + (self.hold.unwrap_or(0) as f64 * 3.0)
            + (self.sell.unwrap_or(0) as f64 * 4.0)
            + (self.strong_sell.unwrap_or(0) as f64 * 5.0);

        Some(score / total as f64)
    }
}
