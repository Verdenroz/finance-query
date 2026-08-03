//! `CHART` capability for Binance public market data.

use crate::constants::{Interval, TimeRange};
use crate::error::{FinanceError, Result};
use crate::models::chart::{Candle, Chart, ChartMeta};
use crate::providers::{Operation, Provider};

use super::client::MAX_KLINES;
use super::models::Kline;
use super::symbols;

/// Requests one chart may spend walking forward through Binance's 1000-candle
/// page cap. Enough for five years of daily or a year of hourly candles.
const MAX_PAGES: u32 = 10;

/// Map a library interval to Binance's kline interval code.
///
/// `ThreeMonths` has no Binance equivalent, so it returns `None` and the
/// caller reports `NotSupported` — dispatch then falls through to the next
/// routed provider.
pub(super) fn interval_code(interval: Interval) -> Option<&'static str> {
    Some(match interval {
        Interval::OneMinute => "1m",
        Interval::FiveMinutes => "5m",
        Interval::FifteenMinutes => "15m",
        Interval::ThirtyMinutes => "30m",
        Interval::OneHour => "1h",
        Interval::OneDay => "1d",
        Interval::OneWeek => "1w",
        Interval::OneMonth => "1M",
        Interval::ThreeMonths => return None,
    })
}

/// The span one candle covers, in seconds. Month candles use a 30-day
/// approximation, matching [`TimeRange::approx_duration_secs`].
pub(super) fn interval_secs(interval: Interval) -> i64 {
    match interval {
        Interval::OneMinute => 60,
        Interval::FiveMinutes => 300,
        Interval::FifteenMinutes => 900,
        Interval::ThirtyMinutes => 1_800,
        Interval::OneHour => 3_600,
        Interval::OneDay => 86_400,
        Interval::OneWeek => 604_800,
        Interval::OneMonth | Interval::ThreeMonths => 2_592_000,
    }
}

/// Turn Binance klines into canonical candles.
///
/// Binance timestamps candles in **milliseconds**; the canonical model uses
/// seconds. Volume is the base-asset amount and is truncated to an integer to
/// fit `Candle::volume`, so sub-unit crypto volumes round toward zero.
pub(super) fn to_candles(klines: Vec<Kline>) -> Vec<Candle> {
    klines
        .into_iter()
        .map(|k| Candle {
            timestamp: k.open_time / 1_000,
            open: k.open,
            high: k.high,
            low: k.low,
            close: k.close,
            volume: k.volume as i64,
            // Crypto has no corporate actions, so close is already adjusted.
            adj_close: Some(k.close),
            provider_id: Some(Provider::Binance),
        })
        .collect()
}

/// Assemble a [`Chart`] around a candle series.
pub(super) fn build_chart(
    symbol: &str,
    market: &str,
    candles: Vec<Candle>,
    interval: Option<Interval>,
    range: Option<TimeRange>,
) -> Chart {
    let quote_currency = symbols::split_pair(market).map(|(_, quote)| quote.to_string());
    Chart {
        symbol: symbol.to_string(),
        meta: ChartMeta {
            symbol: symbol.to_string(),
            currency: quote_currency,
            exchange_name: Some("BINANCE".to_string()),
            full_exchange_name: Some("Binance".to_string()),
            instrument_type: Some("CRYPTOCURRENCY".to_string()),
            first_trade_date: candles.first().map(|c| c.timestamp),
            regular_market_time: candles.last().map(|c| c.timestamp),
            regular_market_price: candles.last().map(|c| c.close),
            data_granularity: interval.map(|i| i.as_str().to_string()),
            range: range.map(|r| r.as_str().to_string()),
            provider_id: Some(Provider::Binance),
            ..Default::default()
        },
        candles,
        interval,
        range,
        provider_id: Some(Provider::Binance),
    }
}

/// Resolve the requested symbol to a Binance market, or explain why it can't
/// be — an unmappable symbol is an equity/index ticker that another provider
/// should serve, so it reports `NotSupported` rather than a hard failure.
fn market_for(symbol: &str) -> Result<String> {
    symbols::parse_market(symbol).ok_or(FinanceError::NotSupported {
        provider: Provider::Binance,
        operation: Operation::Chart,
        candidates: Operation::Chart.capability().candidate_providers(),
    })
}

/// Walk Binance's paginated kline endpoint from `start_ms` to `end_ms`.
async fn collect_klines(
    market: &str,
    code: &str,
    interval: Interval,
    start_ms: i64,
    end_ms: i64,
) -> Result<Vec<Kline>> {
    let client = super::client()?;
    let step_ms = interval_secs(interval) * 1_000;
    let mut cursor = start_ms;
    let mut out: Vec<Kline> = Vec::new();

    for _ in 0..MAX_PAGES {
        let page = client
            .klines(market, code, cursor, end_ms, MAX_KLINES)
            .await?;
        let Some(last) = page.last() else { break };
        // Binance's window is inclusive at both ends, so resume one step past
        // the last candle or the same page would repeat forever.
        cursor = last.open_time + step_ms;
        let full_page = page.len() as u32 == MAX_KLINES;
        out.extend(page);
        if !full_page || cursor > end_ms {
            break;
        }
    }
    Ok(out)
}

/// Fetch candles for `symbol` over `range` at `interval`.
pub(crate) async fn fetch_chart_response(
    symbol: &str,
    interval: Interval,
    range: TimeRange,
) -> Result<Chart> {
    let market = market_for(symbol)?;
    let code = interval_code(interval).ok_or(FinanceError::NotSupported {
        provider: Provider::Binance,
        operation: Operation::Chart,
        candidates: Operation::Chart.capability().candidate_providers(),
    })?;

    let end_ms = chrono::Utc::now().timestamp_millis();
    let start_ms = end_ms - range.approx_duration_secs() * 1_000;
    let klines = collect_klines(&market, code, interval, start_ms, end_ms).await?;

    Ok(build_chart(
        symbol,
        &market,
        to_candles(klines),
        Some(interval),
        Some(range),
    ))
}

/// Fetch candles for `symbol` between two unix-second timestamps.
pub(crate) async fn fetch_chart_range_response(
    symbol: &str,
    interval: Interval,
    start: i64,
    end: i64,
) -> Result<Chart> {
    let market = market_for(symbol)?;
    let code = interval_code(interval).ok_or(FinanceError::NotSupported {
        provider: Provider::Binance,
        operation: Operation::ChartRange,
        candidates: Operation::ChartRange.capability().candidate_providers(),
    })?;

    let klines = collect_klines(&market, code, interval, start * 1_000, end * 1_000).await?;
    Ok(build_chart(
        symbol,
        &market,
        to_candles(klines),
        Some(interval),
        None,
    ))
}
