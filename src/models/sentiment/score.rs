//! Offline lexicon-based sentiment scoring (VADER)

use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use vader_sentiment::SentimentIntensityAnalyzer;

#[cfg(feature = "python")]
use finance_query_derive::PyModel;

#[cfg(feature = "python")]
pub use py::PySentimentLabel;

/// Standard VADER decision threshold on the compound score.
const THRESHOLD: f64 = 0.05;

/// Directional sentiment classification for a piece of text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SentimentLabel {
    /// Net positive tone (`compound >= 0.05`).
    Bullish,
    /// Tone within the neutral band (`-0.05 < compound < 0.05`).
    Neutral,
    /// Net negative tone (`compound <= -0.05`).
    Bearish,
}

impl SentimentLabel {
    /// Human-readable label.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Bullish => "Bullish",
            Self::Neutral => "Neutral",
            Self::Bearish => "Bearish",
        }
    }
}

/// Sentiment score for a news article or transcript segment.
///
/// Only present when the `sentiment` feature is enabled.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
#[cfg_attr(feature = "python", derive(PyModel))]
#[non_exhaustive]
pub struct Sentiment {
    /// Directional classification.
    pub label: SentimentLabel,
    /// Compound score: -1.0 (most bearish) to +1.0 (most bullish).
    pub score: f64,
    /// Confidence: 0.0 to 1.0 (magnitude of the compound score).
    pub confidence: f64,
}

impl Sentiment {
    /// Build a [`Sentiment`] from a VADER compound score in `[-1.0, 1.0]`.
    pub fn from_compound(score: f64) -> Self {
        let label = if score >= THRESHOLD {
            SentimentLabel::Bullish
        } else if score <= -THRESHOLD {
            SentimentLabel::Bearish
        } else {
            SentimentLabel::Neutral
        };
        Self {
            label,
            score,
            confidence: score.abs().clamp(0.0, 1.0),
        }
    }

    /// A neutral, zero-confidence score (used as the empty aggregate).
    pub fn neutral() -> Self {
        Self {
            label: SentimentLabel::Neutral,
            score: 0.0,
            confidence: 0.0,
        }
    }
}

/// Single shared analyzer — `new()` rebinds the lexicon refs, so reuse it.
fn analyzer() -> &'static SentimentIntensityAnalyzer<'static> {
    static ANALYZER: OnceLock<SentimentIntensityAnalyzer<'static>> = OnceLock::new();
    ANALYZER.get_or_init(SentimentIntensityAnalyzer::new)
}

/// Score a single piece of text.
///
/// Lexicon lookup is O(tokens) — typically well under a millisecond per
/// headline. Empty/whitespace text scores [`Sentiment::neutral`].
pub fn analyze(text: &str) -> Sentiment {
    if text.trim().is_empty() {
        return Sentiment::neutral();
    }
    let scores = analyzer().polarity_scores(text);
    let compound = scores.get("compound").copied().unwrap_or(0.0);
    Sentiment::from_compound(compound)
}

/// Aggregate compound scores into a single [`Sentiment`] (simple mean).
///
/// Returns `None` when there is nothing to aggregate.
pub(crate) fn aggregate(scores: &[f64]) -> Option<Sentiment> {
    if scores.is_empty() {
        return None;
    }
    let mean = scores.iter().sum::<f64>() / scores.len() as f64;
    Some(Sentiment::from_compound(mean))
}

/// Aggregate texts weighted by length (longer segments count more).
///
/// Used for transcript-level scoring where paragraphs differ widely in length.
pub(crate) fn aggregate_weighted(texts: &[&str]) -> Option<Sentiment> {
    let mut weighted_sum = 0.0;
    let mut total_weight = 0.0;
    for text in texts {
        let weight = text.trim().len() as f64;
        if weight == 0.0 {
            continue;
        }
        weighted_sum += analyze(text).score * weight;
        total_weight += weight;
    }
    if total_weight == 0.0 {
        return None;
    }
    Some(Sentiment::from_compound(weighted_sum / total_weight))
}

#[cfg(feature = "python")]
mod py {
    use super::SentimentLabel;
    use pyo3::prelude::*;

    #[pyclass(eq, eq_int, hash, frozen, name = "SentimentLabel")]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub enum PySentimentLabel {
        Bullish,
        Neutral,
        Bearish,
    }

    impl ::core::convert::From<PySentimentLabel> for SentimentLabel {
        fn from(v: PySentimentLabel) -> Self {
            match v {
                PySentimentLabel::Bullish => SentimentLabel::Bullish,
                PySentimentLabel::Neutral => SentimentLabel::Neutral,
                PySentimentLabel::Bearish => SentimentLabel::Bearish,
            }
        }
    }

    impl ::core::convert::From<SentimentLabel> for PySentimentLabel {
        fn from(v: SentimentLabel) -> Self {
            match v {
                SentimentLabel::Bullish => PySentimentLabel::Bullish,
                SentimentLabel::Neutral => PySentimentLabel::Neutral,
                SentimentLabel::Bearish => PySentimentLabel::Bearish,
                _ => unreachable!(
                    "SentimentLabel is #[non_exhaustive] but all known variants covered"
                ),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bullish_headline() {
        let s = analyze("Apple stock surges to record high on blockbuster earnings");
        assert_eq!(s.label, SentimentLabel::Bullish);
        assert!(s.score > 0.0);
    }

    #[test]
    fn bearish_headline() {
        let s = analyze("Recession fears mount as inflation disappoints investors");
        assert_eq!(s.label, SentimentLabel::Bearish);
        assert!(s.score < 0.0);
    }

    #[test]
    fn neutral_empty() {
        let s = analyze("   ");
        assert_eq!(s.label, SentimentLabel::Neutral);
        assert_eq!(s.score, 0.0);
        assert_eq!(s.confidence, 0.0);
    }

    #[test]
    fn from_compound_thresholds() {
        assert_eq!(
            Sentiment::from_compound(0.05).label,
            SentimentLabel::Bullish
        );
        assert_eq!(
            Sentiment::from_compound(-0.05).label,
            SentimentLabel::Bearish
        );
        assert_eq!(Sentiment::from_compound(0.0).label, SentimentLabel::Neutral);
        assert_eq!(
            Sentiment::from_compound(0.04).label,
            SentimentLabel::Neutral
        );
    }

    #[test]
    fn aggregate_mean() {
        let agg = aggregate(&[0.8, 0.6, -0.2]).unwrap();
        assert!((agg.score - 0.4).abs() < 1e-9);
        assert_eq!(agg.label, SentimentLabel::Bullish);
        assert!(aggregate(&[]).is_none());
    }

    #[test]
    fn confidence_is_magnitude() {
        assert!((Sentiment::from_compound(-0.8).confidence - 0.8).abs() < 1e-9);
    }
}
