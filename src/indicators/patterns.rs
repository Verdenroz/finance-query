//! Candlestick pattern recognition.
//!
//! Detects 20 common single-, double-, and triple-candle reversal and continuation
//! patterns. Each output position maps 1:1 to the corresponding input candle;
//! `None` means no pattern was detected on that bar.
//!
//! When multiple patterns are technically valid for the same bar the most specific
//! (widest lookback) pattern wins: **three-bar → two-bar → one-bar**.
//!
//! # Pattern catalogue
//!
//! | Bars | Pattern | Signal |
//! |------|---------|--------|
//! | 3 | [`CandlePattern::MorningStar`] | Bullish reversal |
//! | 3 | [`CandlePattern::EveningStar`] | Bearish reversal |
//! | 3 | [`CandlePattern::ThreeWhiteSoldiers`] | Bullish continuation |
//! | 3 | [`CandlePattern::ThreeBlackCrows`] | Bearish continuation |
//! | 2 | [`CandlePattern::BullishEngulfing`] | Bullish reversal |
//! | 2 | [`CandlePattern::BearishEngulfing`] | Bearish reversal |
//! | 2 | [`CandlePattern::BullishHarami`] | Bullish reversal |
//! | 2 | [`CandlePattern::BearishHarami`] | Bearish reversal |
//! | 2 | [`CandlePattern::PiercingLine`] | Bullish reversal |
//! | 2 | [`CandlePattern::DarkCloudCover`] | Bearish reversal |
//! | 2 | [`CandlePattern::TweezerBottom`] | Bullish reversal |
//! | 2 | [`CandlePattern::TweezerTop`] | Bearish reversal |
//! | 1 | [`CandlePattern::Hammer`] | Bullish reversal |
//! | 1 | [`CandlePattern::InvertedHammer`] | Bullish reversal |
//! | 1 | [`CandlePattern::HangingMan`] | Bearish reversal |
//! | 1 | [`CandlePattern::ShootingStar`] | Bearish reversal |
//! | 1 | [`CandlePattern::BullishMarubozu`] | Bullish strength |
//! | 1 | [`CandlePattern::BearishMarubozu`] | Bearish strength |
//! | 1 | [`CandlePattern::Doji`] | Indecision |
//! | 1 | [`CandlePattern::SpinningTop`] | Indecision |
//!
//! # Example
//!
//! ```no_run
//! use finance_query::{Ticker, Interval, TimeRange};
//! use finance_query::indicators::{patterns, CandlePattern};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let ticker = Ticker::new("AAPL").await?;
//! let chart = ticker.chart(Interval::OneDay, TimeRange::ThreeMonths).await?;
//!
//! // Via Chart extension method
//! let signals = chart.patterns();
//! for (i, signal) in signals.iter().enumerate() {
//!     if let Some(p) = signal {
//!         println!("Bar {i}: {p:?} — {:?}", p.sentiment());
//!     }
//! }
//!
//! // Or call directly with a candle slice
//! let signals = patterns(&chart.candles);
//! # Ok(())
//! # }
//! ```

use crate::Candle;
use serde::{Deserialize, Serialize};

// ── Thresholds ────────────────────────────────────────────────────────────────

/// Body / range ratio at or below which a candle is a doji.
const DOJI_BODY_RATIO: f64 = 0.05;

/// Body / range ratio at or below which a candle is a spinning top.
const SPINNING_TOP_BODY_RATIO: f64 = 0.30;

/// Body / range ratio at or above which a candle is a marubozu.
const MARUBOZU_BODY_RATIO: f64 = 0.90;

/// Minimum long-wick / body ratio for hammer and shooting-star shapes.
const LONG_WICK_RATIO: f64 = 2.0;

/// Maximum short-wick / body ratio (the opposing, "tiny" wick side).
const SHORT_WICK_RATIO: f64 = 0.50;

/// Number of prior bars used to classify the short-term trend direction.
const TREND_LOOKBACK: usize = 3;

/// Fractional high / low tolerance for tweezer top / bottom matching.
const TWEEZER_TOLERANCE: f64 = 0.001;

/// Minimum effective body size to avoid division-by-zero on flat candles.
const MIN_BODY: f64 = 1e-9;

// ── Sentiment ─────────────────────────────────────────────────────────────────

/// Directional bias of a candlestick pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum PatternSentiment {
    /// Bullish reversal or continuation signal.
    Bullish,
    /// Bearish reversal or continuation signal.
    Bearish,
    /// Indecision / neutral signal.
    Neutral,
}

// ── CandlePattern ─────────────────────────────────────────────────────────────

/// A detected candlestick pattern.
///
/// Returned per-bar by [`patterns`]. Each bar carries at most one pattern;
/// three-bar patterns take precedence over two-bar, which take precedence over
/// one-bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum CandlePattern {
    // ── Three-bar ─────────────────────────────────────────────────────────
    /// Bullish three-bar reversal: large bearish → small indecision star →
    /// large bullish that closes above the first bar's midpoint.
    MorningStar,
    /// Bearish three-bar reversal: large bullish → small indecision star →
    /// large bearish that closes below the first bar's midpoint.
    EveningStar,
    /// Bullish continuation: three consecutive bullish bars, each opening
    /// within the prior bar's body and closing at a new high.
    ThreeWhiteSoldiers,
    /// Bearish continuation: three consecutive bearish bars, each opening
    /// within the prior bar's body and closing at a new low.
    ThreeBlackCrows,

    // ── Two-bar ───────────────────────────────────────────────────────────
    /// Bullish reversal: bearish bar followed by a larger bullish bar whose
    /// body fully engulfs the prior bar's body.
    BullishEngulfing,
    /// Bearish reversal: bullish bar followed by a larger bearish bar whose
    /// body fully engulfs the prior bar's body.
    BearishEngulfing,
    /// Bullish reversal: large bearish bar followed by a smaller bar (any
    /// colour, including Doji) whose body is contained within the prior bar's
    /// body.  A Doji inner bar is a "Harami Cross" — an even stronger signal.
    BullishHarami,
    /// Bearish reversal: large bullish bar followed by a smaller bar (any
    /// colour, including Doji) whose body is contained within the prior bar's
    /// body.  A Doji inner bar is a "Harami Cross" — an even stronger signal.
    BearishHarami,
    /// Bullish reversal: bearish bar followed by a bullish bar that opens
    /// below the prior close and closes above the prior body's midpoint.
    PiercingLine,
    /// Bearish reversal: bullish bar followed by a bearish bar that opens
    /// above the prior close and closes below the prior body's midpoint.
    DarkCloudCover,
    /// Bearish reversal at resistance: two candles sharing the same high.
    TweezerTop,
    /// Bullish reversal at support: two candles sharing the same low.
    TweezerBottom,

    // ── One-bar ───────────────────────────────────────────────────────────
    /// Indecision: open ≈ close (body ≤ 5 % of total range), wicks on both sides.
    Doji,
    /// Indecision: small body (≤ 30 % of range) with meaningful wicks on both sides.
    SpinningTop,
    /// Bullish strength: nearly wick-free bullish candle (body ≥ 90 % of range).
    BullishMarubozu,
    /// Bearish strength: nearly wick-free bearish candle (body ≥ 90 % of range).
    BearishMarubozu,
    /// Potential bullish reversal: hammer shape (long lower wick) after a downtrend.
    Hammer,
    /// Potential bullish reversal: inverted-hammer shape (long upper wick) after a downtrend.
    InvertedHammer,
    /// Potential bearish reversal: hammer shape (long lower wick) after an uptrend.
    HangingMan,
    /// Potential bearish reversal: inverted-hammer shape (long upper wick) after an uptrend.
    ShootingStar,
}

impl CandlePattern {
    /// Returns the directional bias of this pattern.
    ///
    /// # Example
    ///
    /// ```
    /// use finance_query::indicators::{CandlePattern, PatternSentiment};
    ///
    /// assert_eq!(CandlePattern::Hammer.sentiment(), PatternSentiment::Bullish);
    /// assert_eq!(CandlePattern::ShootingStar.sentiment(), PatternSentiment::Bearish);
    /// assert_eq!(CandlePattern::Doji.sentiment(), PatternSentiment::Neutral);
    /// ```
    pub fn sentiment(self) -> PatternSentiment {
        match self {
            Self::MorningStar
            | Self::ThreeWhiteSoldiers
            | Self::BullishEngulfing
            | Self::BullishHarami
            | Self::PiercingLine
            | Self::TweezerBottom
            | Self::BullishMarubozu
            | Self::Hammer
            | Self::InvertedHammer => PatternSentiment::Bullish,

            Self::EveningStar
            | Self::ThreeBlackCrows
            | Self::BearishEngulfing
            | Self::BearishHarami
            | Self::DarkCloudCover
            | Self::TweezerTop
            | Self::BearishMarubozu
            | Self::HangingMan
            | Self::ShootingStar => PatternSentiment::Bearish,

            Self::Doji | Self::SpinningTop => PatternSentiment::Neutral,
        }
    }
}

// ── Candle helpers ────────────────────────────────────────────────────────────

#[inline]
fn body(c: &Candle) -> f64 {
    (c.close - c.open).abs()
}

#[inline]
fn range(c: &Candle) -> f64 {
    c.high - c.low
}

#[inline]
fn upper_wick(c: &Candle) -> f64 {
    c.high - c.open.max(c.close)
}

#[inline]
fn lower_wick(c: &Candle) -> f64 {
    c.open.min(c.close) - c.low
}

#[inline]
fn is_bullish(c: &Candle) -> bool {
    c.close > c.open
}

#[inline]
fn is_bearish(c: &Candle) -> bool {
    c.close < c.open
}

/// Midpoint of a candle's body.
#[inline]
fn body_mid(c: &Candle) -> f64 {
    (c.open + c.close) / 2.0
}

// ── Trend helpers ─────────────────────────────────────────────────────────────

/// `true` when bar `i` follows a short-term downtrend.
///
/// Compares the close `TREND_LOOKBACK` bars back to the close of the bar
/// immediately *before* the signal candle (`i - 1`), so the signal candle's
/// own close cannot self-validate the trend.
#[inline]
fn prior_downtrend(candles: &[Candle], i: usize) -> bool {
    i > TREND_LOOKBACK && candles[i - 1 - TREND_LOOKBACK].close > candles[i - 1].close
}

/// `true` when bar `i` follows a short-term uptrend.
///
/// Same principle as [`prior_downtrend`] — the signal candle is excluded from
/// the trend evaluation.
#[inline]
fn prior_uptrend(candles: &[Candle], i: usize) -> bool {
    i > TREND_LOOKBACK && candles[i - 1 - TREND_LOOKBACK].close < candles[i - 1].close
}

// ── Single-candle shape predicates ───────────────────────────────────────────

fn is_doji(c: &Candle) -> bool {
    let r = range(c);
    // A four-price doji (O=H=L=C) has zero range — still a valid doji.
    r == 0.0 || body(c) <= r * DOJI_BODY_RATIO
}

fn is_spinning_top(c: &Candle) -> bool {
    let r = range(c);
    let b = body(c);
    !is_doji(c)
        && r > 0.0
        && b <= r * SPINNING_TOP_BODY_RATIO
        && upper_wick(c) >= b
        && lower_wick(c) >= b
}

fn is_bullish_marubozu(c: &Candle) -> bool {
    let r = range(c);
    r > 0.0 && is_bullish(c) && body(c) >= r * MARUBOZU_BODY_RATIO
}

fn is_bearish_marubozu(c: &Candle) -> bool {
    let r = range(c);
    r > 0.0 && is_bearish(c) && body(c) >= r * MARUBOZU_BODY_RATIO
}

/// Hammer shape: small body at the top, long lower wick (≥ `LONG_WICK_RATIO` × body),
/// tiny upper wick (≤ `SHORT_WICK_RATIO` × body).
fn is_hammer_shape(c: &Candle) -> bool {
    let b = body(c).max(MIN_BODY);
    range(c) > 0.0 && lower_wick(c) >= b * LONG_WICK_RATIO && upper_wick(c) <= b * SHORT_WICK_RATIO
}

/// Inverted-hammer shape: small body at the bottom, long upper wick (≥ `LONG_WICK_RATIO` × body),
/// tiny lower wick (≤ `SHORT_WICK_RATIO` × body).
fn is_inverted_hammer_shape(c: &Candle) -> bool {
    let b = body(c).max(MIN_BODY);
    range(c) > 0.0 && upper_wick(c) >= b * LONG_WICK_RATIO && lower_wick(c) <= b * SHORT_WICK_RATIO
}

// ── Pattern detectors ─────────────────────────────────────────────────────────

/// Detect three-bar patterns at position `i` (signal bar is `candles[i]`).
fn detect_three_bar(candles: &[Candle], i: usize) -> Option<CandlePattern> {
    if i < 2 {
        return None;
    }
    let (a, b, c) = (&candles[i - 2], &candles[i - 1], &candles[i]);

    // Three White Soldiers — three bullish bars, each opening within the prior
    // body and each closing at a new high.
    if is_bullish(a)
        && is_bullish(b)
        && is_bullish(c)
        && b.close > a.close
        && c.close > b.close
        && b.open > a.open
        && b.open < a.close
        && c.open > b.open
        && c.open < b.close
    {
        return Some(CandlePattern::ThreeWhiteSoldiers);
    }

    // Three Black Crows — three bearish bars, each opening within the prior
    // body and each closing at a new low.
    if is_bearish(a)
        && is_bearish(b)
        && is_bearish(c)
        && b.close < a.close
        && c.close < b.close
        && b.open < a.open
        && b.open > a.close
        && c.open < b.open
        && c.open > b.close
    {
        return Some(CandlePattern::ThreeBlackCrows);
    }

    // Small-body helper used by both star patterns.
    let b_is_small = body(b) <= range(b).max(MIN_BODY) * SPINNING_TOP_BODY_RATIO;

    // Morning Star — large bearish, small star at or below prior close, then
    // large bullish closing above the first bar's midpoint.
    if is_bearish(a)
        && body(a) >= range(a) * 0.5
        && b_is_small
        && b.open.max(b.close) <= a.close
        && is_bullish(c)
        && c.close > body_mid(a)
    {
        return Some(CandlePattern::MorningStar);
    }

    // Evening Star — large bullish, small star at or above prior close, then
    // large bearish closing below the first bar's midpoint.
    if is_bullish(a)
        && body(a) >= range(a) * 0.5
        && b_is_small
        && b.open.min(b.close) >= a.close
        && is_bearish(c)
        && c.close < body_mid(a)
    {
        return Some(CandlePattern::EveningStar);
    }

    None
}

/// Detect two-bar patterns at position `i` (signal bar is `candles[i]`).
fn detect_two_bar(candles: &[Candle], i: usize) -> Option<CandlePattern> {
    if i < 1 {
        return None;
    }
    let (prev, curr) = (&candles[i - 1], &candles[i]);

    // Tweezers — checked first because an exact price match is rare and highly
    // significant; we don't want it masked by a weaker pattern.
    if (curr.high - prev.high).abs() <= prev.high * TWEEZER_TOLERANCE
        && is_bullish(prev)
        && is_bearish(curr)
    {
        return Some(CandlePattern::TweezerTop);
    }
    if (curr.low - prev.low).abs() <= prev.low * TWEEZER_TOLERANCE
        && is_bearish(prev)
        && is_bullish(curr)
    {
        return Some(CandlePattern::TweezerBottom);
    }

    // Engulfing — current body fully covers the prior body AND is strictly larger.
    // Same-size opposite bodies are "meeting lines", not engulfing.
    if is_bearish(prev)
        && is_bullish(curr)
        && curr.open <= prev.close
        && curr.close >= prev.open
        && body(curr) > body(prev)
    {
        return Some(CandlePattern::BullishEngulfing);
    }
    if is_bullish(prev)
        && is_bearish(curr)
        && curr.open >= prev.close
        && curr.close <= prev.open
        && body(curr) > body(prev)
    {
        return Some(CandlePattern::BearishEngulfing);
    }

    // Harami — current body (any colour, including Doji) is contained within
    // the prior body. A Doji inside the prior body is a "Harami Cross" — an
    // even stronger signal per Nison — and is captured here rather than
    // requiring a separate variant.
    let curr_hi = curr.open.max(curr.close);
    let curr_lo = curr.open.min(curr.close);
    if is_bearish(prev) && curr_lo >= prev.close && curr_hi <= prev.open && body(curr) < body(prev)
    {
        return Some(CandlePattern::BullishHarami);
    }
    if is_bullish(prev) && curr_lo >= prev.open && curr_hi <= prev.close && body(curr) < body(prev)
    {
        return Some(CandlePattern::BearishHarami);
    }

    // Piercing Line — bearish prev, bullish curr opens below the prior close
    // (Nison's definition) and closes above the prior body's midpoint.
    if is_bearish(prev)
        && is_bullish(curr)
        && curr.open < prev.close
        && curr.close > body_mid(prev)
        && curr.close < prev.open
    {
        return Some(CandlePattern::PiercingLine);
    }

    // Dark Cloud Cover — bullish prev, bearish curr opens above the prior close
    // (Nison's definition) and closes below the prior body's midpoint.
    if is_bullish(prev)
        && is_bearish(curr)
        && curr.open > prev.close
        && curr.close < body_mid(prev)
        && curr.close > prev.open
    {
        return Some(CandlePattern::DarkCloudCover);
    }

    None
}

/// Detect one-bar patterns at position `i`.
fn detect_one_bar(candles: &[Candle], i: usize) -> Option<CandlePattern> {
    let c = &candles[i];

    // Doji — must be checked before marubozu and spinning top.
    if is_doji(c) {
        return Some(CandlePattern::Doji);
    }

    // Marubozu — very large body, minimal wicks.
    if is_bullish_marubozu(c) {
        return Some(CandlePattern::BullishMarubozu);
    }
    if is_bearish_marubozu(c) {
        return Some(CandlePattern::BearishMarubozu);
    }

    // Hammer / Hanging Man (same shape, opposite trend context).
    if is_hammer_shape(c) {
        if prior_downtrend(candles, i) {
            return Some(CandlePattern::Hammer);
        }
        if prior_uptrend(candles, i) {
            return Some(CandlePattern::HangingMan);
        }
    }

    // Inverted Hammer / Shooting Star (same shape, opposite trend context).
    if is_inverted_hammer_shape(c) {
        if prior_downtrend(candles, i) {
            return Some(CandlePattern::InvertedHammer);
        }
        if prior_uptrend(candles, i) {
            return Some(CandlePattern::ShootingStar);
        }
    }

    // Spinning Top — catch-all for small-body indecision candles.
    if is_spinning_top(c) {
        return Some(CandlePattern::SpinningTop);
    }

    None
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Detect candlestick patterns for each bar in `candles`.
///
/// Returns a `Vec<Option<CandlePattern>>` of the same length as the input.
/// `Some(pattern)` means a pattern was detected on that bar; `None` means no
/// pattern matched. Input must be in chronological order (oldest candle first).
///
/// When multiple patterns are technically valid for the same bar, the most
/// specific (widest lookback) pattern wins: three-bar patterns take precedence
/// over two-bar, which take precedence over one-bar.
///
/// # Example
///
/// ```no_run
/// use finance_query::{Ticker, Interval, TimeRange};
/// use finance_query::indicators::{patterns, CandlePattern, PatternSentiment};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let ticker = Ticker::new("AAPL").await?;
/// let chart = ticker.chart(Interval::OneDay, TimeRange::SixMonths).await?;
/// let signals = patterns(&chart.candles);
///
/// let bullish: Vec<_> = signals
///     .iter()
///     .enumerate()
///     .filter(|(_, s)| s.map(|p| p.sentiment() == PatternSentiment::Bullish).unwrap_or(false))
///     .collect();
///
/// println!("{} bullish patterns detected", bullish.len());
/// # Ok(())
/// # }
/// ```
pub fn patterns(candles: &[Candle]) -> Vec<Option<CandlePattern>> {
    candles
        .iter()
        .enumerate()
        .map(|(i, _)| {
            detect_three_bar(candles, i)
                .or_else(|| detect_two_bar(candles, i))
                .or_else(|| detect_one_bar(candles, i))
        })
        .collect()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // Convenience constructor — valid within this crate despite #[non_exhaustive].
    fn c(o: f64, h: f64, l: f64, close: f64) -> Candle {
        Candle {
            timestamp: 0,
            open: o,
            high: h,
            low: l,
            close,
            volume: 0,
            adj_close: None,
        }
    }

    // ── Output contract ───────────────────────────────────────────────────────

    #[test]
    fn test_empty_input_returns_empty() {
        assert!(patterns(&[]).is_empty());
    }

    #[test]
    fn test_output_length_matches_input() {
        let candles: Vec<Candle> = (0..15)
            .map(|i| c(i as f64 + 0.5, i as f64 + 1.0, i as f64, i as f64 + 0.6))
            .collect();
        assert_eq!(patterns(&candles).len(), candles.len());
    }

    // ── Single-candle ─────────────────────────────────────────────────────────

    #[test]
    fn test_doji_detected() {
        // body=0.1, range=4.0 → ratio=0.025 < DOJI_BODY_RATIO
        let candles = [c(10.0, 12.0, 8.0, 10.1)];
        assert_eq!(patterns(&candles)[0], Some(CandlePattern::Doji));
    }

    #[test]
    fn test_four_price_doji() {
        // O=H=L=C — the ultimate indecision candle (zero range).
        let candles = [c(10.0, 10.0, 10.0, 10.0)];
        assert_eq!(patterns(&candles)[0], Some(CandlePattern::Doji));
    }

    #[test]
    fn test_doji_not_on_normal_candle() {
        // body=1.5, range=3.0 → ratio=0.5 > DOJI_BODY_RATIO
        let candles = [c(10.0, 12.0, 9.0, 11.5)];
        assert_ne!(patterns(&candles)[0], Some(CandlePattern::Doji));
    }

    #[test]
    fn test_bullish_marubozu() {
        // open≈low, close≈high → body/range ≈ 0.95
        let candles = [c(10.0, 20.05, 9.95, 20.0)];
        assert_eq!(patterns(&candles)[0], Some(CandlePattern::BullishMarubozu));
    }

    #[test]
    fn test_bearish_marubozu() {
        // open≈high, close≈low → body/range ≈ 0.95
        let candles = [c(20.0, 20.05, 9.95, 10.0)];
        assert_eq!(patterns(&candles)[0], Some(CandlePattern::BearishMarubozu));
    }

    #[test]
    fn test_hammer_in_downtrend() {
        // Four declining bars establish a downtrend (trend is evaluated on bars
        // *before* the signal candle, so we need TREND_LOOKBACK + 1 = 4 prior bars).
        //
        // Hammer shape requirements:
        //   body/range > DOJI_BODY_RATIO (5%) so it is NOT classified as a Doji
        //   lower_wick >= body * LONG_WICK_RATIO (2×)
        //   upper_wick <= body * SHORT_WICK_RATIO (0.5×)
        //
        // Hammer candle: open=12.0, high=13.5, low=4.5, close=13.0
        //   body = 1.0, range = 9.0, body/range ≈ 0.11 > 0.05  ✓ (not Doji)
        //   upper_wick = 0.5, lower_wick = 7.5
        //   0.5 ≤ 1.0*0.5=0.5 ✓, 7.5 ≥ 1.0*2=2.0 ✓
        //
        // Trend check: candles[0].close=16.0 > candles[3].close=13.5 → downtrend ✓
        let prior = [
            c(16.0, 17.0, 15.0, 16.0),
            c(15.5, 16.0, 14.5, 15.5),
            c(15.0, 15.5, 13.5, 14.5),
            c(14.0, 14.5, 12.5, 13.5),
        ];
        let hammer = c(12.0, 13.5, 4.5, 13.0);
        let mut candles = prior.to_vec();
        candles.push(hammer);
        assert_eq!(patterns(&candles)[4], Some(CandlePattern::Hammer));
    }

    #[test]
    fn test_shooting_star_in_uptrend() {
        // Four rising bars establish an uptrend (trend evaluated on bars before
        // the signal candle, needs TREND_LOOKBACK + 1 = 4 prior bars).
        //
        // Inverted-hammer shape: long upper wick ≥ 2× body, tiny lower wick ≤ 0.5× body,
        // body/range > 5% so it is NOT classified as a Doji.
        //
        // Candle: open=9.5, high=18.5, low=9.0, close=10.5
        //   body = 1.0, range = 9.5, body/range ≈ 0.105 > 0.05  ✓ (not Doji)
        //   upper_wick = 8.0, lower_wick = 0.5
        //   8.0 ≥ 1.0*2=2.0 ✓, 0.5 ≤ 1.0*0.5=0.5 ✓
        //
        // Trend check: candles[0].close=7.5 < candles[3].close=9.5 → uptrend ✓
        let prior = [
            c(7.0, 8.0, 6.5, 7.5),
            c(7.5, 8.5, 7.0, 8.0),
            c(8.0, 9.0, 7.5, 8.5),
            c(8.5, 9.5, 8.0, 9.5),
        ];
        let star = c(9.5, 18.5, 9.0, 10.5);
        let mut candles = prior.to_vec();
        candles.push(star);
        assert_eq!(patterns(&candles)[4], Some(CandlePattern::ShootingStar));
    }

    // ── Two-candle ────────────────────────────────────────────────────────────

    #[test]
    fn test_bullish_engulfing() {
        let prev = c(11.0, 11.5, 9.5, 10.0); // bearish (o>c)
        let curr = c(9.8, 12.0, 9.7, 11.2); // bullish, open ≤ prev.close, close ≥ prev.open
        let result = patterns(&[prev, curr]);
        assert_eq!(result[1], Some(CandlePattern::BullishEngulfing));
    }

    #[test]
    fn test_bearish_engulfing() {
        let prev = c(10.0, 11.5, 9.5, 11.0); // bullish (c>o)
        let curr = c(11.2, 11.3, 9.0, 9.5); // bearish, open ≥ prev.close, close ≤ prev.open
        let result = patterns(&[prev, curr]);
        assert_eq!(result[1], Some(CandlePattern::BearishEngulfing));
    }

    #[test]
    fn test_bullish_harami() {
        let prev = c(12.0, 12.5, 9.0, 10.0); // bearish: open=12, close=10
        let curr = c(10.5, 11.0, 10.4, 10.8); // bullish, inside prev body
        let result = patterns(&[prev, curr]);
        assert_eq!(result[1], Some(CandlePattern::BullishHarami));
    }

    #[test]
    fn test_bearish_harami() {
        let prev = c(10.0, 12.5, 9.0, 12.0); // bullish: open=10, close=12
        let curr = c(11.5, 12.0, 11.2, 11.3); // bearish, inside prev body
        let result = patterns(&[prev, curr]);
        assert_eq!(result[1], Some(CandlePattern::BearishHarami));
    }

    #[test]
    fn test_bullish_harami_cross_doji_inside() {
        // A Doji (body ≈ 0) contained within a large bearish body is a
        // "Harami Cross" — detected as BullishHarami per Nison.
        let prev = c(12.0, 12.5, 9.0, 10.0); // bearish: open=12, close=10
        // Doji inside prev body: open≈close, body/range tiny
        let doji = c(11.0, 11.5, 10.5, 11.05); // body=0.05, range=1.0, ratio=0.05
        let result = patterns(&[prev, doji]);
        assert_eq!(result[1], Some(CandlePattern::BullishHarami));
    }

    #[test]
    fn test_bearish_harami_cross_doji_inside() {
        // A Doji contained within a large bullish body → BearishHarami.
        let prev = c(10.0, 12.5, 9.0, 12.0); // bullish: open=10, close=12
        let doji = c(11.0, 11.5, 10.5, 11.05); // Doji inside prev body
        let result = patterns(&[prev, doji]);
        assert_eq!(result[1], Some(CandlePattern::BearishHarami));
    }

    #[test]
    fn test_piercing_line() {
        // Bearish prev: open=14, close=10 → mid=12
        // Bullish curr: opens below prev.close=10, closes above mid but below prev.open=14
        let prev = c(14.0, 15.0, 9.0, 10.0);
        let curr = c(9.5, 13.0, 9.4, 12.5); // open=9.5 < prev.close=10 ✓, close=12.5 > mid=12 ✓
        let result = patterns(&[prev, curr]);
        assert_eq!(result[1], Some(CandlePattern::PiercingLine));
    }

    #[test]
    fn test_dark_cloud_cover() {
        // Bullish prev: open=10, close=14 → mid=12
        // Bearish curr: opens above prev.close=14, closes below mid=12 but above prev.open=10
        let prev = c(10.0, 15.0, 9.0, 14.0);
        let curr = c(14.5, 16.0, 10.5, 11.5); // open=14.5 > prev.close=14 ✓, close=11.5 < mid=12 ✓
        let result = patterns(&[prev, curr]);
        assert_eq!(result[1], Some(CandlePattern::DarkCloudCover));
    }

    #[test]
    fn test_tweezer_top() {
        // Both share same high (within tolerance), prev bullish, curr bearish
        let prev = c(10.0, 12.0, 9.5, 11.5); // bullish
        let curr = c(11.6, 12.0, 10.8, 11.0); // bearish, same high
        let result = patterns(&[prev, curr]);
        assert_eq!(result[1], Some(CandlePattern::TweezerTop));
    }

    #[test]
    fn test_tweezer_bottom() {
        // Both share same low, prev bearish, curr bullish
        let prev = c(11.5, 12.0, 9.5, 10.0); // bearish
        let curr = c(9.8, 11.0, 9.5, 10.5); // bullish, same low
        let result = patterns(&[prev, curr]);
        assert_eq!(result[1], Some(CandlePattern::TweezerBottom));
    }

    // ── Three-candle ──────────────────────────────────────────────────────────

    #[test]
    fn test_three_white_soldiers() {
        // Each bar is bullish, opens in prior body, closes at new high
        let c1 = c(10.0, 11.2, 9.8, 11.0);
        let c2 = c(10.5, 12.2, 10.4, 12.0); // opens in c1 body (10.0..11.0), closes above c1
        let c3 = c(11.5, 13.2, 11.4, 13.0); // opens in c2 body (10.5..12.0), closes above c2
        let result = patterns(&[c1, c2, c3]);
        assert_eq!(result[2], Some(CandlePattern::ThreeWhiteSoldiers));
    }

    #[test]
    fn test_three_black_crows() {
        // Each bar is bearish, opens in prior body, closes at new low
        let c1 = c(13.0, 13.2, 11.8, 12.0);
        let c2 = c(12.5, 12.6, 10.8, 11.0); // opens in c1 body (12.0..13.0), closes below c1
        let c3 = c(11.5, 11.6, 9.8, 10.0); // opens in c2 body (11.0..12.5), closes below c2
        let result = patterns(&[c1, c2, c3]);
        assert_eq!(result[2], Some(CandlePattern::ThreeBlackCrows));
    }

    #[test]
    fn test_morning_star() {
        // Large bearish → small star below prior close → large bullish above prior mid
        // a: open=110, close=102 → mid=106, body=8, range=12, b/r=0.67 ✓
        // b: small body at 100–101 (below a.close=102) ✓
        // c: bullish, closes at 108 > 106 ✓
        let a = c(110.0, 112.0, 100.0, 102.0);
        let b = c(100.5, 101.0, 99.5, 100.8); // body=0.3, range=1.5, b/r=0.2 ≤ 0.3 ✓
        // b.open.max(b.close) = 100.8 ≤ a.close=102 ✓
        let cc = c(101.0, 112.0, 100.0, 108.0); // close=108 > mid=106 ✓
        let result = patterns(&[a, b, cc]);
        assert_eq!(result[2], Some(CandlePattern::MorningStar));
    }

    #[test]
    fn test_evening_star() {
        // Large bullish → small star above prior close → large bearish below prior mid
        // a: open=100, close=110 → mid=105, body=10, range=12, b/r=0.83 ✓
        // b: small body at 111–112 (above a.close=110) ✓
        // c: bearish, closes at 103 < 105 ✓
        let a = c(100.0, 111.0, 99.0, 110.0);
        let b = c(111.0, 112.5, 110.8, 111.3); // body=0.3, range=1.7, b/r≈0.18 ✓
        // b.open.min(b.close) = 111.0 >= a.close=110 ✓
        let cc = c(110.5, 111.0, 102.0, 103.0); // close=103 < mid=105 ✓
        let result = patterns(&[a, b, cc]);
        assert_eq!(result[2], Some(CandlePattern::EveningStar));
    }

    // ── Sentiment ─────────────────────────────────────────────────────────────

    #[test]
    fn test_sentiment_coverage() {
        assert_eq!(CandlePattern::Hammer.sentiment(), PatternSentiment::Bullish);
        assert_eq!(
            CandlePattern::MorningStar.sentiment(),
            PatternSentiment::Bullish
        );
        assert_eq!(
            CandlePattern::BullishEngulfing.sentiment(),
            PatternSentiment::Bullish
        );
        assert_eq!(
            CandlePattern::ShootingStar.sentiment(),
            PatternSentiment::Bearish
        );
        assert_eq!(
            CandlePattern::EveningStar.sentiment(),
            PatternSentiment::Bearish
        );
        assert_eq!(
            CandlePattern::BearishEngulfing.sentiment(),
            PatternSentiment::Bearish
        );
        assert_eq!(CandlePattern::Doji.sentiment(), PatternSentiment::Neutral);
        assert_eq!(
            CandlePattern::SpinningTop.sentiment(),
            PatternSentiment::Neutral
        );
    }

    // ── Priority (three-bar beats two-bar beats one-bar) ─────────────────────

    #[test]
    fn test_three_bar_takes_priority_over_two_bar() {
        // Construct a sequence where the last two bars also form a BullishEngulfing
        // but the three-bar context makes it a MorningStar confirmation.
        let a = c(110.0, 112.0, 100.0, 102.0);
        let b = c(100.5, 101.0, 99.5, 100.8);
        // Make bar c both a bullish engulfer of b AND complete the morning star
        let cc = c(99.0, 112.0, 98.0, 108.0); // open < b.close=100.8 ✓ (engulfs b) & > mid(a)=106
        let result = patterns(&[a, b, cc]);
        // MorningStar should win
        assert_eq!(result[2], Some(CandlePattern::MorningStar));
    }
}
