//! CoinGecko historical OHLC candles — the keyless `CHART` route for crypto.
//!
//! CoinGecko's public tier serves `/coins/{id}/ohlc`, which picks its own bar
//! granularity from the requested day span (30m for 1–2 days, 4h for 3–30,
//! 4d beyond that) and carries no volume. The requested [`Interval`] is
//! therefore advisory, and candles come back with `volume: 0` rather than a
//! number invented from a differently-bucketed series.

use serde::Deserialize;

use crate::constants::{Interval, TimeRange};
use crate::error::{FinanceError, Result};
use crate::models::chart::{Candle, Chart};
use crate::providers::Provider;

/// `/coins/{id}/ohlc` rows are `[timestamp_ms, open, high, low, close]`.
#[derive(Debug, Clone, Deserialize)]
pub struct OhlcRowDTO(pub f64, pub f64, pub f64, pub f64, pub f64);

/// Day spans the public tier accepts on `/ohlc`.
pub(crate) fn range_to_days(range: TimeRange) -> &'static str {
    match range {
        TimeRange::OneDay => "1",
        TimeRange::FiveDays => "7",
        TimeRange::OneMonth => "30",
        TimeRange::ThreeMonths => "90",
        TimeRange::SixMonths => "180",
        // The public tier's `days` is an enum, so a partial year rounds up to
        // the nearest accepted span rather than being rejected.
        TimeRange::OneYear | TimeRange::YearToDate => "365",
        TimeRange::TwoYears | TimeRange::FiveYears | TimeRange::TenYears | TimeRange::Max => "max",
    }
}

/// Split a chart symbol such as `"BTC-USD"` or `"USD-COIN-USD"` back into the
/// CoinGecko coin id and quote currency.
///
/// Splitting on the *last* hyphen matters: CoinGecko ids frequently contain
/// hyphens (`usd-coin`, `staked-ether`), and splitting on the first would
/// truncate them.
pub(crate) fn split_chart_symbol(symbol: &str) -> Result<(String, String)> {
    symbol
        .rsplit_once('-')
        .filter(|(id, vs)| !id.is_empty() && !vs.is_empty())
        .map(|(id, vs)| (id.to_ascii_lowercase(), vs.to_ascii_lowercase()))
        .ok_or_else(|| FinanceError::InvalidParameter {
            param: "symbol".to_string(),
            reason: format!(
                "CoinGecko charts need a '{{id}}-{{vs_currency}}' symbol (e.g. 'bitcoin-usd'), got '{symbol}'"
            ),
        })
}

pub(crate) fn rows_to_candles(rows: Vec<OhlcRowDTO>) -> Vec<Candle> {
    rows.into_iter()
        .map(|OhlcRowDTO(ts_ms, open, high, low, close)| Candle {
            timestamp: (ts_ms / 1000.0) as i64,
            open,
            high,
            low,
            close,
            volume: 0,
            adj_close: None,
            provider_id: Some(Provider::CoinGecko),
        })
        .collect()
}

/// Fetch OHLC candles for a `"{id}-{vs_currency}"` chart symbol.
pub async fn fetch_chart_response(
    symbol: &str,
    interval: Interval,
    range: TimeRange,
) -> Result<Chart> {
    let (id, vs_currency) = split_chart_symbol(symbol)?;
    let rows = super::client()?
        .ohlc(&id, &vs_currency, range_to_days(range))
        .await?;

    Ok(Chart {
        symbol: symbol.to_string(),
        meta: Default::default(),
        candles: rows_to_candles(rows),
        interval: Some(interval),
        range: Some(range),
        provider_id: Some(Provider::CoinGecko),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_on_the_last_hyphen_so_hyphenated_ids_survive() {
        assert_eq!(
            split_chart_symbol("BTC-USD").unwrap(),
            ("btc".to_string(), "usd".to_string())
        );
        assert_eq!(
            split_chart_symbol("USD-COIN-USD").unwrap(),
            ("usd-coin".to_string(), "usd".to_string())
        );
        assert_eq!(
            split_chart_symbol("staked-ether-eur").unwrap(),
            ("staked-ether".to_string(), "eur".to_string())
        );
    }

    #[test]
    fn rejects_symbols_without_a_quote_currency() {
        for bad in ["BTC", "", "-USD", "BTC-"] {
            assert!(
                split_chart_symbol(bad).is_err(),
                "{bad} should not parse as a CoinGecko chart symbol"
            );
        }
    }

    #[test]
    fn maps_ranges_onto_the_public_tiers_day_enum() {
        assert_eq!(range_to_days(TimeRange::OneDay), "1");
        assert_eq!(range_to_days(TimeRange::FiveDays), "7");
        assert_eq!(range_to_days(TimeRange::OneMonth), "30");
        assert_eq!(range_to_days(TimeRange::ThreeMonths), "90");
        assert_eq!(range_to_days(TimeRange::SixMonths), "180");
        assert_eq!(range_to_days(TimeRange::OneYear), "365");
        assert_eq!(range_to_days(TimeRange::YearToDate), "365");
        assert_eq!(range_to_days(TimeRange::Max), "max");
        assert_eq!(range_to_days(TimeRange::TenYears), "max");
    }

    #[test]
    fn converts_millisecond_rows_into_candles() {
        let rows: Vec<OhlcRowDTO> = serde_json::from_value(serde_json::json!([
            [1704067200000.0, 42000.0, 42500.0, 41800.0, 42300.0],
            [1704081600000.0, 42300.0, 42900.0, 42200.0, 42800.0]
        ]))
        .unwrap();

        let candles = rows_to_candles(rows);
        assert_eq!(candles.len(), 2);
        assert_eq!(candles[0].timestamp, 1_704_067_200);
        assert_eq!(candles[0].open, 42000.0);
        assert_eq!(candles[0].high, 42500.0);
        assert_eq!(candles[0].low, 41800.0);
        assert_eq!(candles[0].close, 42300.0);
        // `/ohlc` carries no volume; 0 is honest, an invented figure would not be.
        assert_eq!(candles[0].volume, 0);
        assert_eq!(candles[0].provider_id, Some(Provider::CoinGecko));
    }

    #[test]
    fn empty_response_yields_no_candles() {
        let rows: Vec<OhlcRowDTO> = serde_json::from_value(serde_json::json!([])).unwrap();
        assert!(rows_to_candles(rows).is_empty());
    }
}
