//! Core stock time series endpoints: intraday, daily, weekly, monthly, quotes, search, market status.

use crate::error::{FinanceError, Result};
use crate::models::chart::{Candle, Chart};
use crate::models::quote::{FormattedValue, Price, QuoteSummaryResponse};

use super::build_client;
use super::models::*;

/// Helper to parse OHLCV entries from an AV time series JSON response.
fn parse_time_series(
    json: &serde_json::Value,
    series_key: &str,
) -> Result<Vec<TimeSeriesEntryDTO>> {
    let series = json
        .as_object()
        .and_then(|obj| {
            obj.iter()
                .find(|(k, _)| k.contains(series_key) || k.starts_with("Time Series"))
                .map(|(_, v)| v)
        })
        .and_then(|v| v.as_object())
        .ok_or_else(|| FinanceError::ResponseStructureError {
            field: "Time Series".to_string(),
            context: "Missing time series data in response".to_string(),
        })?;

    let mut entries: Vec<TimeSeriesEntryDTO> = series
        .iter()
        .filter_map(|(timestamp, values)| {
            Some(TimeSeriesEntryDTO {
                timestamp: timestamp.clone(),
                open: values.get("1. open")?.as_str()?.parse().ok()?,
                high: values.get("2. high")?.as_str()?.parse().ok()?,
                low: values.get("3. low")?.as_str()?.parse().ok()?,
                close: values.get("4. close")?.as_str()?.parse().ok()?,
                volume: values.get("5. volume")?.as_str()?.parse().ok()?,
            })
        })
        .collect();

    entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    Ok(entries)
}

/// Helper to parse adjusted OHLCV entries.
#[allow(dead_code)] // unrouted: remaining Alpha Vantage endpoints land with #264
fn parse_adjusted_time_series(
    json: &serde_json::Value,
    series_key: &str,
) -> Result<Vec<AdjustedTimeSeriesEntryDTO>> {
    let series = json
        .as_object()
        .and_then(|obj| {
            obj.iter()
                .find(|(k, _)| k.contains(series_key) || k.starts_with("Time Series"))
                .map(|(_, v)| v)
        })
        .and_then(|v| v.as_object())
        .ok_or_else(|| FinanceError::ResponseStructureError {
            field: "Time Series".to_string(),
            context: "Missing adjusted time series data in response".to_string(),
        })?;

    let mut entries: Vec<AdjustedTimeSeriesEntryDTO> = series
        .iter()
        .filter_map(|(timestamp, values)| {
            Some(AdjustedTimeSeriesEntryDTO {
                timestamp: timestamp.clone(),
                open: values.get("1. open")?.as_str()?.parse().ok()?,
                high: values.get("2. high")?.as_str()?.parse().ok()?,
                low: values.get("3. low")?.as_str()?.parse().ok()?,
                close: values.get("4. close")?.as_str()?.parse().ok()?,
                adjusted_close: values.get("5. adjusted close")?.as_str()?.parse().ok()?,
                volume: values.get("6. volume")?.as_str()?.parse().ok()?,
                dividend_amount: values
                    .get("7. dividend amount")
                    .and_then(|v| v.as_str()?.parse().ok())
                    .unwrap_or(0.0),
                split_coefficient: values
                    .get("8. split coefficient")
                    .and_then(|v| v.as_str()?.parse().ok()),
            })
        })
        .collect();

    entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    Ok(entries)
}

/// Extract symbol from AV Meta Data.
fn extract_symbol(json: &serde_json::Value) -> String {
    json.get("Meta Data")
        .and_then(|m| m.get("2. Symbol"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string()
}

/// Extract last refreshed from AV Meta Data.
fn extract_last_refreshed(json: &serde_json::Value) -> String {
    json.get("Meta Data")
        .and_then(|m| m.get("3. Last Refreshed"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

/// Fetch intraday time series data.
///
/// Returns OHLCV data at the specified interval (1min, 5min, 15min, 30min, 60min).
///
/// # Arguments
///
/// * `symbol` - Ticker symbol (e.g., `"AAPL"`)
/// * `interval` - Time interval
/// * `output_size` - `None` for compact (100 points), or specify `OutputSize::Full`
pub async fn time_series_intraday(
    symbol: &str,
    interval: AvInterval,
    output_size: Option<OutputSize>,
) -> Result<TimeSeriesDTO> {
    let client = build_client()?;
    let mut params = vec![("symbol", symbol), ("interval", interval.as_str())];
    let os_str;
    if let Some(os) = output_size {
        os_str = os.as_str().to_string();
        params.push(("outputsize", &os_str));
    }
    let json = client.get("TIME_SERIES_INTRADAY", &params).await?;
    let entries = parse_time_series(&json, "Intraday")?;

    Ok(TimeSeriesDTO {
        symbol: extract_symbol(&json),
        last_refreshed: extract_last_refreshed(&json),
        entries,
    })
}

/// Fetch daily time series (raw, unadjusted).
pub async fn time_series_daily(
    symbol: &str,
    output_size: Option<OutputSize>,
) -> Result<TimeSeriesDTO> {
    let client = build_client()?;
    let mut params = vec![("symbol", symbol)];
    let os_str;
    if let Some(os) = output_size {
        os_str = os.as_str().to_string();
        params.push(("outputsize", &os_str));
    }
    let json = client.get("TIME_SERIES_DAILY", &params).await?;
    let entries = parse_time_series(&json, "Daily")?;

    Ok(TimeSeriesDTO {
        symbol: extract_symbol(&json),
        last_refreshed: extract_last_refreshed(&json),
        entries,
    })
}

/// Fetch daily adjusted time series (includes dividends and splits).
#[allow(dead_code)] // unrouted: remaining Alpha Vantage endpoints land with #264
pub async fn time_series_daily_adjusted(
    symbol: &str,
    output_size: Option<OutputSize>,
) -> Result<AdjustedTimeSeriesDTO> {
    let client = build_client()?;
    let mut params = vec![("symbol", symbol)];
    let os_str;
    if let Some(os) = output_size {
        os_str = os.as_str().to_string();
        params.push(("outputsize", &os_str));
    }
    let json = client.get("TIME_SERIES_DAILY_ADJUSTED", &params).await?;
    let entries = parse_adjusted_time_series(&json, "Daily")?;

    Ok(AdjustedTimeSeriesDTO {
        symbol: extract_symbol(&json),
        last_refreshed: extract_last_refreshed(&json),
        entries,
    })
}

/// Fetch weekly time series (raw, unadjusted).
#[allow(dead_code)] // unrouted: remaining Alpha Vantage endpoints land with #264
pub async fn time_series_weekly(symbol: &str) -> Result<TimeSeriesDTO> {
    let client = build_client()?;
    let json = client
        .get("TIME_SERIES_WEEKLY", &[("symbol", symbol)])
        .await?;
    let entries = parse_time_series(&json, "Weekly")?;

    Ok(TimeSeriesDTO {
        symbol: extract_symbol(&json),
        last_refreshed: extract_last_refreshed(&json),
        entries,
    })
}

/// Fetch weekly adjusted time series.
#[allow(dead_code)] // unrouted: remaining Alpha Vantage endpoints land with #264
pub async fn time_series_weekly_adjusted(symbol: &str) -> Result<AdjustedTimeSeriesDTO> {
    let client = build_client()?;
    let json = client
        .get("TIME_SERIES_WEEKLY_ADJUSTED", &[("symbol", symbol)])
        .await?;
    let entries = parse_adjusted_time_series(&json, "Weekly")?;

    Ok(AdjustedTimeSeriesDTO {
        symbol: extract_symbol(&json),
        last_refreshed: extract_last_refreshed(&json),
        entries,
    })
}

/// Fetch monthly time series (raw, unadjusted).
#[allow(dead_code)] // unrouted: remaining Alpha Vantage endpoints land with #264
pub async fn time_series_monthly(symbol: &str) -> Result<TimeSeriesDTO> {
    let client = build_client()?;
    let json = client
        .get("TIME_SERIES_MONTHLY", &[("symbol", symbol)])
        .await?;
    let entries = parse_time_series(&json, "Monthly")?;

    Ok(TimeSeriesDTO {
        symbol: extract_symbol(&json),
        last_refreshed: extract_last_refreshed(&json),
        entries,
    })
}

/// Fetch monthly adjusted time series.
#[allow(dead_code)] // unrouted: remaining Alpha Vantage endpoints land with #264
pub async fn time_series_monthly_adjusted(symbol: &str) -> Result<AdjustedTimeSeriesDTO> {
    let client = build_client()?;
    let json = client
        .get("TIME_SERIES_MONTHLY_ADJUSTED", &[("symbol", symbol)])
        .await?;
    let entries = parse_adjusted_time_series(&json, "Monthly")?;

    Ok(AdjustedTimeSeriesDTO {
        symbol: extract_symbol(&json),
        last_refreshed: extract_last_refreshed(&json),
        entries,
    })
}

/// Fetch the latest price and volume for a single ticker.
pub async fn global_quote(symbol: &str) -> Result<GlobalQuoteDTO> {
    let client = build_client()?;
    let json = client.get("GLOBAL_QUOTE", &[("symbol", symbol)]).await?;

    let q = json
        .get("Global Quote")
        .ok_or_else(|| FinanceError::ResponseStructureError {
            field: "Global Quote".to_string(),
            context: "Missing Global Quote in response".to_string(),
        })?;

    Ok(GlobalQuoteDTO {
        symbol: q
            .get("01. symbol")
            .and_then(|v| v.as_str())
            .unwrap_or(symbol)
            .to_string(),
        open: q
            .get("02. open")
            .and_then(|v| v.as_str()?.parse().ok())
            .unwrap_or(0.0),
        high: q
            .get("03. high")
            .and_then(|v| v.as_str()?.parse().ok())
            .unwrap_or(0.0),
        low: q
            .get("04. low")
            .and_then(|v| v.as_str()?.parse().ok())
            .unwrap_or(0.0),
        price: q
            .get("05. price")
            .and_then(|v| v.as_str()?.parse().ok())
            .unwrap_or(0.0),
        volume: q
            .get("06. volume")
            .and_then(|v| v.as_str()?.parse().ok())
            .unwrap_or(0.0),
        latest_trading_day: q
            .get("07. latest trading day")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        previous_close: q
            .get("08. previous close")
            .and_then(|v| v.as_str()?.parse().ok())
            .unwrap_or(0.0),
        change: q
            .get("09. change")
            .and_then(|v| v.as_str()?.parse().ok())
            .unwrap_or(0.0),
        change_percent: q
            .get("10. change percent")
            .and_then(|v| v.as_str())
            .unwrap_or("0%")
            .to_string(),
    })
}

/// Fetch realtime bulk quotes for up to 100 symbols.
///
/// # Arguments
///
/// * `symbols` - Comma-separated symbols or a slice (e.g., `"AAPL,MSFT,GOOG"`)
pub async fn realtime_bulk_quotes(symbols: &str) -> Result<Vec<BulkQuoteDTO>> {
    let client = build_client()?;
    let json = client
        .get("REALTIME_BULK_QUOTES", &[("symbol", symbols)])
        .await?;

    let data = json.get("data").and_then(|v| v.as_array()).ok_or_else(|| {
        FinanceError::ResponseStructureError {
            field: "data".to_string(),
            context: "Missing data array in bulk quotes response".to_string(),
        }
    })?;

    Ok(data
        .iter()
        .filter_map(|item| {
            Some(BulkQuoteDTO {
                symbol: item.get("symbol")?.as_str()?.to_string(),
                open: item.get("open").and_then(|v| v.as_str()?.parse().ok()),
                high: item.get("high").and_then(|v| v.as_str()?.parse().ok()),
                low: item.get("low").and_then(|v| v.as_str()?.parse().ok()),
                price: item.get("price").and_then(|v| v.as_str()?.parse().ok()),
                volume: item.get("volume").and_then(|v| v.as_str()?.parse().ok()),
                latest_trading_day: item
                    .get("latest_trading_day")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                previous_close: item
                    .get("previous_close")
                    .and_then(|v| v.as_str()?.parse().ok()),
                change: item.get("change").and_then(|v| v.as_str()?.parse().ok()),
                change_percent: item
                    .get("change_percent")
                    .and_then(|v| v.as_str())
                    .map(String::from),
            })
        })
        .collect())
}

/// Search for symbols matching the given keywords.
pub async fn symbol_search(keywords: &str) -> Result<Vec<SymbolMatchDTO>> {
    let client = build_client()?;
    let json = client
        .get("SYMBOL_SEARCH", &[("keywords", keywords)])
        .await?;

    let matches = json
        .get("bestMatches")
        .and_then(|v| v.as_array())
        .ok_or_else(|| FinanceError::ResponseStructureError {
            field: "bestMatches".to_string(),
            context: "Missing bestMatches in search response".to_string(),
        })?;

    Ok(matches
        .iter()
        .filter_map(|m| {
            Some(SymbolMatchDTO {
                symbol: m.get("1. symbol")?.as_str()?.to_string(),
                name: m.get("2. name")?.as_str()?.to_string(),
                asset_type: m.get("3. type")?.as_str()?.to_string(),
                region: m.get("4. region")?.as_str()?.to_string(),
                market_open: m.get("5. marketOpen")?.as_str()?.to_string(),
                market_close: m.get("6. marketClose")?.as_str()?.to_string(),
                timezone: m.get("7. timezone")?.as_str()?.to_string(),
                currency: m.get("8. currency")?.as_str()?.to_string(),
                match_score: m.get("9. matchScore")?.as_str()?.parse().ok()?,
            })
        })
        .collect())
}

/// Fetch the current market status of major global exchanges.
#[allow(dead_code)] // unrouted: remaining Alpha Vantage endpoints land with #264
pub async fn market_status() -> Result<Vec<MarketStatusDTO>> {
    let client = build_client()?;
    let json = client.get("MARKET_STATUS", &[]).await?;

    let markets = json
        .get("markets")
        .and_then(|v| v.as_array())
        .ok_or_else(|| FinanceError::ResponseStructureError {
            field: "markets".to_string(),
            context: "Missing markets array in response".to_string(),
        })?;

    Ok(markets
        .iter()
        .filter_map(|m| {
            Some(MarketStatusDTO {
                market_type: m.get("market_type")?.as_str()?.to_string(),
                region: m.get("region")?.as_str()?.to_string(),
                primary_exchanges: m.get("primary_exchanges")?.as_str()?.to_string(),
                local_open: m.get("local_open")?.as_str()?.to_string(),
                local_close: m.get("local_close")?.as_str()?.to_string(),
                current_status: m.get("current_status")?.as_str()?.to_string(),
                notes: m
                    .get("notes")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            })
        })
        .collect())
}

// ============================================================================
// Canonical model conversion functions
// ============================================================================

/// Fetch canonical QuoteSummaryResponse from a stock symbol.
pub async fn fetch_quote_response(symbol: &str) -> Result<QuoteSummaryResponse> {
    let gq = global_quote(symbol).await?;
    let price = Price {
        regular_market_price: Some(FormattedValue {
            raw: Some(gq.price),
            fmt: None,
            long_fmt: None,
        }),
        regular_market_change: Some(FormattedValue {
            raw: Some(gq.change),
            fmt: None,
            long_fmt: None,
        }),
        regular_market_change_percent: gq.change_percent.trim_end_matches('%').parse().ok().map(
            |v| FormattedValue {
                raw: Some(v),
                fmt: None,
                long_fmt: None,
            },
        ),
        regular_market_volume: Some(FormattedValue {
            raw: Some(gq.volume as i64),
            fmt: None,
            long_fmt: None,
        }),
        regular_market_previous_close: Some(FormattedValue {
            raw: Some(gq.previous_close),
            fmt: None,
            long_fmt: None,
        }),
        regular_market_open: Some(FormattedValue {
            raw: Some(gq.open),
            fmt: None,
            long_fmt: None,
        }),
        regular_market_day_high: Some(FormattedValue {
            raw: Some(gq.high),
            fmt: None,
            long_fmt: None,
        }),
        regular_market_day_low: Some(FormattedValue {
            raw: Some(gq.low),
            fmt: None,
            long_fmt: None,
        }),
        ..Default::default()
    };
    Ok(QuoteSummaryResponse {
        symbol: symbol.to_string(),
        price: Some(price),
        ..Default::default()
    })
}

/// Build a canonical quote response from one bulk-quote entry.
fn bulk_quote_to_canonical(q: &BulkQuoteDTO) -> QuoteSummaryResponse {
    let raw = |v: Option<f64>| {
        v.map(|v| FormattedValue {
            raw: Some(v),
            fmt: None,
            long_fmt: None,
        })
    };
    let price = Price {
        regular_market_price: raw(q.price),
        regular_market_change: raw(q.change),
        regular_market_change_percent: raw(q
            .change_percent
            .as_deref()
            .and_then(|v| v.trim_end_matches('%').parse().ok())),
        regular_market_volume: q.volume.map(|v| FormattedValue {
            raw: Some(v as i64),
            fmt: None,
            long_fmt: None,
        }),
        regular_market_previous_close: raw(q.previous_close),
        regular_market_open: raw(q.open),
        regular_market_day_high: raw(q.high),
        regular_market_day_low: raw(q.low),
        ..Default::default()
    };
    QuoteSummaryResponse {
        symbol: q.symbol.clone(),
        price: Some(price),
        ..Default::default()
    }
}

/// Fetch canonical quote summaries for many symbols in one request.
///
/// Alpha Vantage's `REALTIME_BULK_QUOTES` takes up to 100 comma-separated
/// symbols, so an N-symbol batch costs one request instead of N — material on
/// the 25 req/day free tier. Note this endpoint is premium-only; on a free key
/// it errors and `ProviderSet` falls back to per-symbol fetches.
pub async fn fetch_quotes_batch_response(
    symbols: &[&str],
) -> Result<Vec<(String, QuoteSummaryResponse)>> {
    let joined = symbols.join(",");
    let quotes = realtime_bulk_quotes(&joined).await?;
    Ok(quotes
        .iter()
        .map(|q| (q.symbol.clone(), bulk_quote_to_canonical(q)))
        .collect())
}

/// Fetch canonical Chart from a symbol with interval/range.
pub async fn fetch_chart_response(
    symbol: &str,
    interval: crate::Interval,
    _range: crate::TimeRange,
) -> Result<Chart> {
    let ts = match interval {
        crate::Interval::OneMinute => {
            time_series_intraday(symbol, AvInterval::OneMin, None).await?
        }
        crate::Interval::FiveMinutes => {
            time_series_intraday(symbol, AvInterval::FiveMin, None).await?
        }
        crate::Interval::FifteenMinutes => {
            time_series_intraday(symbol, AvInterval::FifteenMin, None).await?
        }
        crate::Interval::ThirtyMinutes => {
            time_series_intraday(symbol, AvInterval::ThirtyMin, None).await?
        }
        crate::Interval::OneHour => {
            time_series_intraday(symbol, AvInterval::SixtyMin, None).await?
        }
        _ => time_series_daily(symbol, None).await?,
    };

    Ok(Chart {
        symbol: symbol.to_string(),
        meta: Default::default(),
        candles: entries_to_candles(ts.entries),
        interval: None,
        range: None,
        provider_id: Some(crate::Provider::AlphaVantage),
    })
}

/// Fetch canonical Chart from a symbol with explicit date range.
pub async fn fetch_chart_range_response(
    symbol: &str,
    _interval: crate::Interval,
    start: i64,
    end: i64,
) -> Result<Chart> {
    let from_date = timestamp_to_date_string(start);
    let to_date = timestamp_to_date_string(end);

    let ts = time_series_daily(symbol, Some(super::models::OutputSize::Full)).await?;

    let in_range: Vec<_> = ts
        .entries
        .into_iter()
        .filter(|bar| bar.timestamp >= from_date && bar.timestamp <= to_date)
        .collect();

    Ok(Chart {
        symbol: symbol.to_string(),
        meta: Default::default(),
        candles: entries_to_candles(in_range),
        interval: None,
        range: None,
        provider_id: Some(crate::Provider::AlphaVantage),
    })
}

/// Convert time series entries into canonical candles, ascending by timestamp.
///
/// The DTO layer sorts entries newest-first, but `BacktestEngine::run` rejects
/// candles that are not ascending, so the chart bridge normalises here.
fn entries_to_candles(entries: Vec<TimeSeriesEntryDTO>) -> Vec<Candle> {
    let mut candles: Vec<Candle> = entries
        .into_iter()
        .map(|bar| Candle {
            timestamp: parse_entry_timestamp(&bar.timestamp),
            open: bar.open,
            high: bar.high,
            low: bar.low,
            close: bar.close,
            volume: bar.volume as i64,
            adj_close: None,
            provider_id: Some(crate::Provider::AlphaVantage),
        })
        .collect();
    candles.sort_unstable_by_key(|c| c.timestamp);
    candles
}

/// Parse an Alpha Vantage time series key into a Unix timestamp.
///
/// Daily series are keyed `YYYY-MM-DD`; intraday series carry a time component.
/// An unparseable key yields 0 rather than dropping the bar, matching the
/// previous behaviour.
fn parse_entry_timestamp(key: &str) -> i64 {
    chrono::NaiveDateTime::parse_from_str(key, "%Y-%m-%d %H:%M:%S")
        .ok()
        .or_else(|| {
            chrono::NaiveDate::parse_from_str(key, "%Y-%m-%d")
                .ok()
                .and_then(|d| d.and_hms_opt(0, 0, 0))
        })
        .map(|dt| dt.and_utc().timestamp())
        .unwrap_or(0)
}

/// Convert a Unix timestamp to a YYYY-MM-DD date string.
fn timestamp_to_date_string(ts: i64) -> String {
    chrono::DateTime::from_timestamp(ts, 0)
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "1970-01-01".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(timestamp: &str, close: f64) -> TimeSeriesEntryDTO {
        TimeSeriesEntryDTO {
            timestamp: timestamp.to_string(),
            open: close,
            high: close,
            low: close,
            close,
            volume: 1.0,
        }
    }

    #[test]
    fn entries_to_candles_sorts_newest_first_entries_ascending() {
        // parse_time_series sorts descending; BacktestEngine::run rejects that order.
        let candles = entries_to_candles(vec![
            entry("2024-01-04", 3.0),
            entry("2024-01-03", 2.0),
            entry("2024-01-02", 1.0),
        ]);
        assert!(
            candles.windows(2).all(|w| w[0].timestamp <= w[1].timestamp),
            "candles must be ascending"
        );
        assert_eq!(candles[0].close, 1.0);
        assert_eq!(candles[2].close, 3.0);
    }

    #[test]
    fn entries_to_candles_keeps_intraday_time_of_day() {
        // Intraday keys carry a time component; collapsing them to midnight
        // would flatten every bar of a session onto one timestamp.
        let candles = entries_to_candles(vec![
            entry("2024-01-02 09:35:00", 2.0),
            entry("2024-01-02 09:30:00", 1.0),
        ]);
        assert_eq!(candles[0].timestamp, 1_704_187_800);
        assert_eq!(candles[1].timestamp, 1_704_188_100);
        assert_ne!(
            candles[0].timestamp, candles[1].timestamp,
            "intraday bars must keep distinct timestamps"
        );
    }

    #[test]
    fn parse_entry_timestamp_falls_back_to_zero_on_garbage() {
        assert_eq!(parse_entry_timestamp("not-a-date"), 0);
        assert_eq!(parse_entry_timestamp("2024-01-02"), 1_704_153_600);
    }

    #[test]
    fn test_parse_time_series_daily() {
        let json: serde_json::Value = serde_json::json!({
            "Meta Data": {
                "1. Information": "Daily Prices",
                "2. Symbol": "AAPL",
                "3. Last Refreshed": "2024-01-15"
            },
            "Time Series (Daily)": {
                "2024-01-15": {
                    "1. open": "185.0900",
                    "2. high": "187.0100",
                    "3. low": "184.3500",
                    "4. close": "186.1900",
                    "5. volume": "65076600"
                },
                "2024-01-12": {
                    "1. open": "186.0600",
                    "2. high": "186.7400",
                    "3. low": "185.1900",
                    "4. close": "185.5900",
                    "5. volume": "40434100"
                }
            }
        });

        let entries = parse_time_series(&json, "Daily").unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].timestamp, "2024-01-15");
        assert!((entries[0].open - 185.09).abs() < 0.01);
        assert!((entries[0].close - 186.19).abs() < 0.01);
        assert!((entries[0].volume - 65076600.0).abs() < 1.0);
    }

    #[test]
    fn test_parse_adjusted_time_series() {
        let json: serde_json::Value = serde_json::json!({
            "Meta Data": {
                "2. Symbol": "AAPL",
                "3. Last Refreshed": "2024-01-15"
            },
            "Time Series (Daily)": {
                "2024-01-15": {
                    "1. open": "185.09",
                    "2. high": "187.01",
                    "3. low": "184.35",
                    "4. close": "186.19",
                    "5. adjusted close": "186.19",
                    "6. volume": "65076600",
                    "7. dividend amount": "0.24",
                    "8. split coefficient": "1.0"
                }
            }
        });

        let entries = parse_adjusted_time_series(&json, "Daily").unwrap();
        assert_eq!(entries.len(), 1);
        assert!((entries[0].adjusted_close - 186.19).abs() < 0.01);
        assert!((entries[0].dividend_amount - 0.24).abs() < 0.01);
        assert_eq!(entries[0].split_coefficient, Some(1.0));
    }

    #[test]
    fn test_extract_symbol() {
        let json = serde_json::json!({
            "Meta Data": { "2. Symbol": "MSFT" }
        });
        assert_eq!(extract_symbol(&json), "MSFT");
    }

    #[test]
    fn test_parse_time_series_missing_data() {
        let json = serde_json::json!({"Meta Data": {}});
        assert!(parse_time_series(&json, "Daily").is_err());
    }

    #[tokio::test]
    async fn test_global_quote_mock() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/")
            .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
                "function".into(),
                "GLOBAL_QUOTE".into(),
            )]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "Global Quote": {
                        "01. symbol": "AAPL",
                        "02. open": "185.0900",
                        "03. high": "187.0100",
                        "04. low": "184.3500",
                        "05. price": "186.1900",
                        "06. volume": "65076600",
                        "07. latest trading day": "2024-01-15",
                        "08. previous close": "185.5900",
                        "09. change": "0.6000",
                        "10. change percent": "0.3233%"
                    }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let json = client
            .get("GLOBAL_QUOTE", &[("symbol", "AAPL")])
            .await
            .unwrap();

        let q = json.get("Global Quote").unwrap();
        assert_eq!(q.get("01. symbol").unwrap().as_str().unwrap(), "AAPL");
        assert_eq!(q.get("05. price").unwrap().as_str().unwrap(), "186.1900");
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_symbol_search_mock() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/")
            .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
                "function".into(),
                "SYMBOL_SEARCH".into(),
            )]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "bestMatches": [{
                        "1. symbol": "AAPL",
                        "2. name": "Apple Inc",
                        "3. type": "Equity",
                        "4. region": "United States",
                        "5. marketOpen": "09:30",
                        "6. marketClose": "16:00",
                        "7. timezone": "UTC-04",
                        "8. currency": "USD",
                        "9. matchScore": "1.0000"
                    }]
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let json = client
            .get("SYMBOL_SEARCH", &[("keywords", "apple")])
            .await
            .unwrap();

        let matches = json.get("bestMatches").unwrap().as_array().unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(
            matches[0].get("1. symbol").unwrap().as_str().unwrap(),
            "AAPL"
        );
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_rate_limit_json_error() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "Note": "Thank you for using Alpha Vantage! Our standard API call frequency is 25 calls per day."
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let result = client.get("GLOBAL_QUOTE", &[("symbol", "AAPL")]).await;
        assert!(matches!(
            result,
            Err(crate::error::FinanceError::RateLimited { .. })
        ));
    }

    #[tokio::test]
    async fn test_error_message_response() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "Error Message": "Invalid API call. Please retry or visit the documentation."
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let result = client.get("INVALID_FUNC", &[]).await;
        assert!(matches!(
            result,
            Err(crate::error::FinanceError::InvalidParameter { .. })
        ));
    }

    #[tokio::test]
    async fn test_http_401_error() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(401)
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let result = client.get("GLOBAL_QUOTE", &[("symbol", "AAPL")]).await;
        assert!(matches!(
            result,
            Err(crate::error::FinanceError::AuthenticationFailed { .. })
        ));
    }

    #[tokio::test]
    async fn test_information_field_premium_endpoint_includes_message() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"Information": "Thank you for using Alpha Vantage! This is a premium endpoint."}"#)
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let result = client.get("OVERVIEW", &[("symbol", "AAPL")]).await;

        let err = result
            .expect_err("expected an error from a 200 OK with non-rate-limit Information field");
        let msg = format!("{err:?}");
        assert!(
            msg.contains("premium endpoint"),
            "expected error to include AV info text, got: {msg}"
        );
    }

    #[tokio::test]
    async fn test_daily_time_series_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/")
            .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
                "function".into(),
                "TIME_SERIES_DAILY".into(),
            )]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "Meta Data": {
                        "1. Information": "Daily Prices (open, high, low, close) for AAPL",
                        "2. Symbol": "AAPL",
                        "3. Last Refreshed": "2024-01-15",
                        "4. Output Size": "Compact",
                        "5. Time Zone": "US/Eastern"
                    },
                    "Time Series (Daily)": {
                        "2024-01-15": {
                            "1. open": "185.09",
                            "2. high": "187.01",
                            "3. low": "184.35",
                            "4. close": "186.19",
                            "5. volume": "65076600"
                        },
                        "2024-01-12": {
                            "1. open": "186.06",
                            "2. high": "186.74",
                            "3. low": "185.19",
                            "4. close": "185.59",
                            "5. volume": "40434100"
                        }
                    }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let json = client
            .get("TIME_SERIES_DAILY", &[("symbol", "AAPL")])
            .await
            .unwrap();

        let entries = parse_time_series(&json, "Daily").unwrap();
        assert_eq!(entries.len(), 2);

        let symbol = extract_symbol(&json);
        assert_eq!(symbol, "AAPL");
    }
}
