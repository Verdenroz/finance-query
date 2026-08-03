//! `CHART` capability for Kraken public market data.

use crate::constants::{Interval, TimeRange};
use crate::error::{FinanceError, Result};
use crate::models::chart::{Candle, Chart, ChartMeta};
use crate::providers::{Operation, Provider};

use super::models::KrakenCandle;
use super::symbols;

/// Map a library interval to Kraken's OHLC interval, in minutes.
///
/// Kraken offers 1/5/15/30/60/240/1440/10080/21600. `ThirtyMinutes` and
/// `OneMonth` land exactly; `ThreeMonths` has no equivalent and returns
/// `None`, so the caller reports `NotSupported` and dispatch falls through.
pub(super) fn interval_minutes(interval: Interval) -> Option<u32> {
    Some(match interval {
        Interval::OneMinute => 1,
        Interval::FiveMinutes => 5,
        Interval::FifteenMinutes => 15,
        Interval::ThirtyMinutes => 30,
        Interval::OneHour => 60,
        Interval::OneDay => 1_440,
        Interval::OneWeek => 10_080,
        // Kraken's longest bucket is 15 days, the closest thing to a month.
        Interval::OneMonth => 21_600,
        Interval::ThreeMonths => return None,
    })
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
    Chart {
        symbol: symbol.to_string(),
        meta: ChartMeta {
            symbol: symbol.to_string(),
            currency: quote_currency,
            exchange_name: Some("KRAKEN".to_string()),
            full_exchange_name: Some("Kraken".to_string()),
            instrument_type: Some("CRYPTOCURRENCY".to_string()),
            first_trade_date: candles.first().map(|c| c.timestamp),
            regular_market_time: candles.last().map(|c| c.timestamp),
            regular_market_price: candles.last().map(|c| c.close),
            data_granularity: interval.map(|i| i.as_str().to_string()),
            range: range.map(|r| r.as_str().to_string()),
            provider_id: Some(Provider::Kraken),
            ..Default::default()
        },
        candles,
        interval,
        range,
        provider_id: Some(Provider::Kraken),
    }
}

fn not_supported(operation: Operation) -> FinanceError {
    FinanceError::NotSupported {
        provider: Provider::Kraken,
        operation,
        candidates: operation.capability().candidate_providers(),
    }
}

/// Resolve the requested symbol to a Kraken pair, or report `NotSupported` —
/// an unmappable symbol is an equity/index ticker another provider should
/// serve, not a hard failure.
fn pair_for(symbol: &str, operation: Operation) -> Result<String> {
    symbols::parse_market(symbol).ok_or_else(|| not_supported(operation))
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
    let minutes = interval_minutes(interval).ok_or_else(|| not_supported(Operation::Chart))?;

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
    let minutes = interval_minutes(interval).ok_or_else(|| not_supported(Operation::ChartRange))?;

    let raw = super::client()?.ohlc(&pair, minutes, start).await?;
    let candles = to_candles(raw)
        .into_iter()
        .filter(|c| c.timestamp <= end)
        .collect();

    Ok(build_chart(symbol, &pair, candles, Some(interval), None))
}
