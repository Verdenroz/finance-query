//! Market sentiment models.
//!
//! Contains the Fear & Greed Index data from Alternative.me (market-wide gauge)
//! and per-symbol sentiment from provider news analysis (Polygon, etc.).

pub(crate) mod response;

#[cfg(feature = "sentiment")]
mod score;

pub use response::{FearAndGreed, FearGreedLabel};

#[cfg(feature = "sentiment")]
pub use score::{Sentiment, SentimentLabel, analyze};

#[cfg(feature = "sentiment")]
pub(crate) use score::{aggregate, aggregate_weighted};

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

#[cfg(feature = "python")]
pub use response::{PyFearAndGreed, PyFearGreedLabel};

#[cfg(all(feature = "python", feature = "sentiment"))]
pub use score::{PySentiment, PySentimentLabel};
