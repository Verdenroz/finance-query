//! `CHART` capability for Kraken public market data.

use crate::adapters::common::crypto_chart::{CryptoExchange, crypto_chart};
use crate::constants::{Interval, TimeRange};
use crate::error::Result;
use crate::models::chart::{Candle, Chart};
use crate::providers::{Operation, Provider};

use super::models::KrakenCandle;
use super::symbols;

/// How Kraken identifies itself in canonical chart metadata.
const EXCHANGE: CryptoExchange = CryptoExchange {
    provider: Provider::Kraken,
    code: "KRAKEN",
    name: "Kraken",
};

/// Map a library interval to Kraken's OHLC interval, in minutes.
///
/// Kraken offers 1/5/15/30/60/240/1440/10080/21600. `ThirtyMinutes` and
/// `OneMonth` land exactly; `TwoMinutes`/`NinetyMinutes`/`FiveDays`/
/// `ThreeMonths` have no equivalent and return `None`, so the caller reports
/// `NotSupported` and dispatch falls through.
pub(super) fn interval_minutes(interval: Interval) -> Option<u32> {
    match interval {
        // Kraken's longest bucket is 15 days, the closest thing to a month,
        // so this one does not follow from the interval's nominal span.
        Interval::OneMonth => Some(21_600),
        Interval::TwoMinutes | Interval::NinetyMinutes | Interval::FiveDays => None,
        Interval::ThreeMonths => None,
        other => Some((other.duration_secs() / 60) as u32),
    }
}

/// Turn Kraken candles into canonical candles.
///
/// Kraken already timestamps in seconds. Volume is base-asset and is truncated
/// to an integer to fit `Candle::volume`, so sub-unit volumes round toward zero.
pub(super) fn to_candles(raw: Vec<KrakenCandle>) -> Vec<Candle> {
    raw.into_iter()
        .map(|c| Candle {
            timestamp: c.time,
            open: c.open,
            high: c.high,
            low: c.low,
            close: c.close,
            volume: c.volume as i64,
            // Crypto has no corporate actions, so close is already adjusted.
            adj_close: Some(c.close),
            provider_id: Some(Provider::Kraken),
        })
        .collect()
}

/// Assemble a [`Chart`] around a candle series.
pub(super) fn build_chart(
    symbol: &str,
    pair: &str,
    candles: Vec<Candle>,
    interval: Option<Interval>,
    range: Option<TimeRange>,
) -> Chart {
    let quote_currency =
        symbols::split_pair(pair).map(|(_, quote)| symbols::from_kraken_asset(quote));
    crypto_chart(&EXCHANGE, symbol, quote_currency, candles, interval, range)
}

/// Resolve the requested symbol to a Kraken pair, or report `NotSupported` —
/// an unmappable symbol is an equity/index ticker another provider should
/// serve, not a hard failure.
fn pair_for(symbol: &str, operation: Operation) -> Result<String> {
    symbols::parse_market(symbol).ok_or_else(|| operation.not_supported(Provider::Kraken))
}

/// Fetch candles for `symbol` over `range` at `interval`.
///
/// Kraken caps `/OHLC` at ~720 candles ending at now, and `since` only moves
/// the window's start forward — there is no way to page further back. A range
/// wider than 720 candles therefore returns the most recent 720.
pub(crate) async fn fetch_chart_response(
    symbol: &str,
    interval: Interval,
    range: TimeRange,
) -> Result<Chart> {
    let pair = pair_for(symbol, Operation::Chart)?;
    let minutes = interval_minutes(interval)
        .ok_or_else(|| Operation::Chart.not_supported(Provider::Kraken))?;

    let since = chrono::Utc::now().timestamp() - range.approx_duration_secs();
    let raw = super::client()?.ohlc(&pair, minutes, since).await?;

    Ok(build_chart(
        symbol,
        &pair,
        to_candles(raw),
        Some(interval),
        Some(range),
    ))
}

/// Fetch candles for `symbol` between two unix-second timestamps.
///
/// Kraken has no window end parameter, so candles after `end` are filtered out
/// locally.
pub(crate) async fn fetch_chart_range_response(
    symbol: &str,
    interval: Interval,
    start: i64,
    end: i64,
) -> Result<Chart> {
    let pair = pair_for(symbol, Operation::ChartRange)?;
    let minutes = interval_minutes(interval)
        .ok_or_else(|| Operation::ChartRange.not_supported(Provider::Kraken))?;

    let raw = super::client()?.ohlc(&pair, minutes, start).await?;
    let candles = to_candles(raw)
        .into_iter()
        .filter(|c| c.timestamp <= end)
        .collect();

    Ok(build_chart(symbol, &pair, candles, Some(interval), None))
}
