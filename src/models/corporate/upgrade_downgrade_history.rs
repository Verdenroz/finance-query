//! Upgrade/Downgrade History Module
//!
//! Contains analyst rating changes and price target updates.

use serde::{Deserialize, Serialize};

/// Analyst upgrade/downgrade history
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpgradeDowngradeHistory {
    /// List of rating changes
    #[serde(default)]
    pub history: Vec<GradeChange>,

    /// Maximum age of this data in seconds
    #[serde(default)]
    pub max_age: Option<i64>,
}

/// Individual analyst rating change
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GradeChange {
    /// Timestamp of the grade change (epoch)
    #[serde(default)]
    pub epoch_grade_date: Option<i64>,

    /// Name of the firm making the change
    #[serde(default)]
    pub firm: Option<String>,

    /// Previous rating
    #[serde(default)]
    pub from_grade: Option<String>,

    /// New rating
    #[serde(default)]
    pub to_grade: Option<String>,

    /// Type of action (e.g., "main", "init", "down", "up", "reit")
    #[serde(default)]
    pub action: Option<String>,

    /// Previous price target
    #[serde(default)]
    pub prior_price_target: Option<f64>,

    /// New price target
    #[serde(default)]
    pub current_price_target: Option<f64>,

    /// Price target action description (e.g., "Raises", "Maintains", "Lowers")
    #[serde(default)]
    pub price_target_action: Option<String>,
}

impl GradeChange {
    /// Returns true if this was an upgrade
    pub fn is_upgrade(&self) -> bool {
        matches!(
            self.action.as_deref(),
            Some("up") | Some("init-upgrade") | Some("reit-up")
        )
    }

    /// Returns true if this was a downgrade
    pub fn is_downgrade(&self) -> bool {
        matches!(
            self.action.as_deref(),
            Some("down") | Some("init-downgrade") | Some("reit-down")
        )
    }

    /// Returns true if the price target increased
    pub fn price_target_raised(&self) -> bool {
        self.price_target_action.as_deref() == Some("Raises")
    }

    /// Returns true if the price target decreased
    pub fn price_target_lowered(&self) -> bool {
        self.price_target_action.as_deref() == Some("Lowers")
    }

    /// Calculate the price target change
    pub fn price_target_change(&self) -> Option<f64> {
        match (self.current_price_target, self.prior_price_target) {
            (Some(current), Some(prior)) => Some(current - prior),
            _ => None,
        }
    }

    /// Calculate the price target change percentage
    pub fn price_target_change_percent(&self) -> Option<f64> {
        match (self.current_price_target, self.prior_price_target) {
            (Some(current), Some(prior)) if prior != 0.0 => {
                Some(((current - prior) / prior) * 100.0)
            }
            _ => None,
        }
    }
}
