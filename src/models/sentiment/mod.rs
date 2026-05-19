//! Market sentiment models.
//!
//! Contains the Fear & Greed Index data from Alternative.me (market-wide gauge)
//! and per-symbol sentiment from provider news analysis (Polygon, etc.).

pub(crate) mod response;

pub use response::{FearAndGreed, FearGreedLabel};

use serde::{Deserialize, Serialize};

/// Per-symbol sentiment from provider news analysis (Polygon, etc.).
///
/// Scores range from -1.0 (very negative) to 1.0 (very positive).
/// Unlike [`FearAndGreed`] (a market-wide 0—100 gauge from Alternative.me),
/// this reflects news sentiment for a specific stock.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SymbolSentiment {
    /// Sentiment score (-1.0 = very negative, 0.0 = neutral, 1.0 = very positive).
    pub score: Option<f64>,
    /// Human-readable label (e.g., "Bullish", "Bearish", "Neutral").
    pub label: Option<String>,
}
