//! Heikin-Ashi candle transform.
//!
//! Unlike the rest of `indicators`, this doesn't compute an oscillator or
//! overlay value — it transforms OHLC candles into smoothed "average bar"
//! candles that make the prevailing trend easier to read.

use super::{IndicatorError, Result};
use crate::Candle;

/// Dense open/high/low/close series for a Heikin-Ashi transform.
///
/// Used internally where only price arrays are available (e.g. the
/// backtesting engine's custom-strategy indicator dispatch), rather than
/// full [`Candle`]s.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct HeikinAshiSeries {
    pub open: Vec<f64>,
    pub high: Vec<f64>,
    pub low: Vec<f64>,
    pub close: Vec<f64>,
}

/// Compute the Heikin-Ashi open/high/low/close series from raw price arrays.
///
/// # Formula
///
/// - `ha_close = (open + high + low + close) / 4`
/// - `ha_open = (prev_ha_open + prev_ha_close) / 2` (first bar: `(open + close) / 2`)
/// - `ha_high = max(high, ha_open, ha_close)`
/// - `ha_low = min(low, ha_open, ha_close)`
pub(crate) fn heikin_ashi_raw(
    opens: &[f64],
    highs: &[f64],
    lows: &[f64],
    closes: &[f64],
) -> Result<HeikinAshiSeries> {
    let len = opens.len();
    if highs.len() != len || lows.len() != len || closes.len() != len {
        return Err(IndicatorError::InvalidPeriod(
            "opens, highs, lows, and closes must have the same length".to_string(),
        ));
    }
    if len == 0 {
        return Err(IndicatorError::InsufficientData { need: 1, got: 0 });
    }

    let mut ha_open = Vec::with_capacity(len);
    let mut ha_high = Vec::with_capacity(len);
    let mut ha_low = Vec::with_capacity(len);
    let mut ha_close = Vec::with_capacity(len);

    for i in 0..len {
        let close = (opens[i] + highs[i] + lows[i] + closes[i]) / 4.0;
        let open = if i == 0 {
            (opens[i] + closes[i]) / 2.0
        } else {
            (ha_open[i - 1] + ha_close[i - 1]) / 2.0
        };
        let high = highs[i].max(open).max(close);
        let low = lows[i].min(open).min(close);

        ha_open.push(open);
        ha_high.push(high);
        ha_low.push(low);
        ha_close.push(close);
    }

    Ok(HeikinAshiSeries {
        open: ha_open,
        high: ha_high,
        low: ha_low,
        close: ha_close,
    })
}

/// Transform standard OHLC candles into Heikin-Ashi candles.
///
/// Volume, timestamp, adjusted close, and provider id pass through
/// unchanged — only open/high/low/close are recomputed.
///
/// # Example
///
/// ```no_run
/// use finance_query::{Ticker, Interval, TimeRange};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let ticker = Ticker::new("AAPL").await?;
/// let chart = ticker.chart(Interval::OneDay, TimeRange::ThreeMonths).await?;
///
/// let ha_candles = chart.heikin_ashi()?;
/// # Ok(())
/// # }
/// ```
pub fn heikin_ashi(candles: &[Candle]) -> Result<Vec<Candle>> {
    if candles.is_empty() {
        return Err(IndicatorError::InsufficientData { need: 1, got: 0 });
    }
    let opens: Vec<f64> = candles.iter().map(|c| c.open).collect();
    let highs: Vec<f64> = candles.iter().map(|c| c.high).collect();
    let lows: Vec<f64> = candles.iter().map(|c| c.low).collect();
    let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();

    let series = heikin_ashi_raw(&opens, &highs, &lows, &closes)?;

    Ok(candles
        .iter()
        .enumerate()
        .map(|(i, c)| Candle {
            timestamp: c.timestamp,
            open: series.open[i],
            high: series.high[i],
            low: series.low[i],
            close: series.close[i],
            volume: c.volume,
            adj_close: c.adj_close,
            provider_id: c.provider_id,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_candle(open: f64, high: f64, low: f64, close: f64) -> Candle {
        serde_json::from_value(serde_json::json!({
            "timestamp": 0,
            "open": open,
            "high": high,
            "low": low,
            "close": close,
            "volume": 1_000_000,
            "adjClose": close,
        }))
        .unwrap()
    }

    #[test]
    fn test_heikin_ashi_basic() {
        let candles = vec![
            make_candle(10.0, 12.0, 9.0, 11.0),
            make_candle(11.0, 13.0, 10.5, 12.5),
        ];
        let ha = heikin_ashi(&candles).unwrap();
        assert_eq!(ha.len(), 2);

        // First bar: ha_close = (10+12+9+11)/4 = 10.5, ha_open = (10+11)/2 = 10.5
        assert!((ha[0].close - 10.5).abs() < 1e-9);
        assert!((ha[0].open - 10.5).abs() < 1e-9);
        assert!(ha[0].high >= ha[0].open.max(ha[0].close));
        assert!(ha[0].low <= ha[0].open.min(ha[0].close));

        // Second bar's ha_open is the average of the first bar's ha_open/ha_close
        let expected_open = (ha[0].open + ha[0].close) / 2.0;
        assert!((ha[1].open - expected_open).abs() < 1e-9);

        // Volume/timestamp pass through unchanged
        assert_eq!(ha[0].volume, candles[0].volume);
        assert_eq!(ha[1].timestamp, candles[1].timestamp);
    }

    #[test]
    fn test_heikin_ashi_empty() {
        assert!(heikin_ashi(&[]).is_err());
    }

    #[test]
    fn test_heikin_ashi_smooths_gaps() {
        // A single volatile bar shouldn't produce a HA high/low outside the real range.
        let candles = vec![
            make_candle(10.0, 10.0, 10.0, 10.0),
            make_candle(10.0, 50.0, 5.0, 10.0),
        ];
        let ha = heikin_ashi(&candles).unwrap();
        assert!(ha[1].high <= 50.0);
        assert!(ha[1].low >= 5.0);
    }
}
