//! Cryptocurrency endpoints: exchange rates and crypto time series.

#![allow(dead_code)]
use crate::error::{FinanceError, Result};

use super::build_client;
use super::models::*;

/// Helper to parse crypto time series responses.
fn parse_crypto_series(
    json: &serde_json::Value,
    symbol: &str,
    market: &str,
) -> Result<CryptoTimeSeriesDTO> {
    let meta = json.get("Meta Data");
    let last_refreshed = meta
        .and_then(|m| {
            m.get("6. Last Refreshed")
                .or_else(|| m.get("5. Last Refreshed"))
        })
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let series = json
        .as_object()
        .and_then(|obj| {
            obj.iter()
                .find(|(k, _)| k.starts_with("Time Series"))
                .map(|(_, v)| v)
        })
        .and_then(|v| v.as_object())
        .ok_or_else(|| FinanceError::ResponseStructureError {
            field: "Time Series Crypto".to_string(),
            context: "Missing crypto time series data".to_string(),
        })?;

    let market_lower = market.to_lowercase();
    let open_key = format!("1a. open ({market_lower})");
    let high_key = format!("2a. high ({market_lower})");
    let low_key = format!("3a. low ({market_lower})");
    let close_key = format!("4a. close ({market_lower})");

    let mut entries: Vec<CryptoEntryDTO> = series
        .iter()
        .filter_map(|(timestamp, values)| {
            // Try market-specific keys first, fall back to generic numbered keys
            let open = values
                .get(&open_key)
                .or_else(|| values.get("1. open"))
                .and_then(|v| v.as_str()?.parse().ok())?;
            let high = values
                .get(&high_key)
                .or_else(|| values.get("2. high"))
                .and_then(|v| v.as_str()?.parse().ok())?;
            let low = values
                .get(&low_key)
                .or_else(|| values.get("3. low"))
                .and_then(|v| v.as_str()?.parse().ok())?;
            let close = values
                .get(&close_key)
                .or_else(|| values.get("4. close"))
                .and_then(|v| v.as_str()?.parse().ok())?;
            let volume = values
                .get("5. volume")
                .and_then(|v| v.as_str()?.parse().ok())
                .unwrap_or(0.0);

            Some(CryptoEntryDTO {
                timestamp: timestamp.clone(),
                open,
                high,
                low,
                close,
                volume,
            })
        })
        .collect();

    entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    Ok(CryptoTimeSeriesDTO {
        symbol: symbol.to_string(),
        market: market.to_string(),
        last_refreshed,
        entries,
    })
}

/// Fetch intraday cryptocurrency time series.
pub async fn crypto_intraday(
    symbol: &str,
    market: &str,
    interval: AvInterval,
    output_size: Option<OutputSize>,
) -> Result<CryptoTimeSeriesDTO> {
    let client = build_client()?;
    let mut params = vec![
        ("symbol", symbol),
        ("market", market),
        ("interval", interval.as_str()),
    ];
    let os_str;
    if let Some(os) = output_size {
        os_str = os.as_str().to_string();
        params.push(("outputsize", &os_str));
    }
    let json = client.get("CRYPTO_INTRADAY", &params).await?;
    parse_crypto_series(&json, symbol, market)
}

/// Fetch daily cryptocurrency time series.
pub async fn crypto_daily(symbol: &str, market: &str) -> Result<CryptoTimeSeriesDTO> {
    let client = build_client()?;
    let json = client
        .get(
            "DIGITAL_CURRENCY_DAILY",
            &[("symbol", symbol), ("market", market)],
        )
        .await?;
    parse_crypto_series(&json, symbol, market)
}

/// Fetch weekly cryptocurrency time series.
pub async fn crypto_weekly(symbol: &str, market: &str) -> Result<CryptoTimeSeriesDTO> {
    let client = build_client()?;
    let json = client
        .get(
            "DIGITAL_CURRENCY_WEEKLY",
            &[("symbol", symbol), ("market", market)],
        )
        .await?;
    parse_crypto_series(&json, symbol, market)
}

/// Fetch monthly cryptocurrency time series.
pub async fn crypto_monthly(symbol: &str, market: &str) -> Result<CryptoTimeSeriesDTO> {
    let client = build_client()?;
    let json = client
        .get(
            "DIGITAL_CURRENCY_MONTHLY",
            &[("symbol", symbol), ("market", market)],
        )
        .await?;
    parse_crypto_series(&json, symbol, market)
}

// ============================================================================
// Canonical model conversion functions
// ============================================================================

/// Fetch canonical CryptoQuote for a symbol/market pair.
pub async fn fetch_crypto_quote_response(
    symbol: &str,
    market: &str,
) -> Result<crate::models::crypto::CryptoQuote> {
    let series = crypto_daily(symbol, market).await?;
    Ok(series_to_quote(series))
}

/// Map a daily time series to the canonical [`CryptoQuote`](crate::models::crypto::CryptoQuote),
/// deriving 24h change from the two most recent entries.
fn series_to_quote(series: CryptoTimeSeriesDTO) -> crate::models::crypto::CryptoQuote {
    let (price, change_24h, change_percent_24h, high_24h, low_24h, volume_24h) =
        if series.entries.len() >= 2 {
            let latest = &series.entries[0];
            let prev = &series.entries[1];
            let chg = latest.close - prev.close;
            let pct = if prev.close != 0.0 {
                (chg / prev.close) * 100.0
            } else {
                0.0
            };
            (
                Some(latest.close),
                Some(chg),
                Some(pct),
                Some(latest.high),
                Some(latest.low),
                Some(latest.volume),
            )
        } else if series.entries.len() == 1 {
            let latest = &series.entries[0];
            (
                Some(latest.close),
                None,
                None,
                Some(latest.high),
                Some(latest.low),
                Some(latest.volume),
            )
        } else {
            (None, None, None, None, None, None)
        };

    crate::models::crypto::CryptoQuote {
        id: series.symbol.clone(),
        symbol: series.symbol,
        name: String::new(),
        price,
        market_cap: None,
        volume_24h,
        change_24h,
        change_percent_24h,
        high_24h,
        low_24h,
        circulating_supply: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn series(entries: serde_json::Value) -> CryptoTimeSeriesDTO {
        serde_json::from_value(serde_json::json!({
            "symbol": "BTC",
            "market": "USD",
            "last_refreshed": "2024-01-15 00:00:00",
            "entries": entries
        }))
        .unwrap()
    }

    #[test]
    fn series_to_quote_derives_change_from_two_latest_entries() {
        let quote = series_to_quote(series(serde_json::json!([
            {"timestamp": "2024-01-15", "open": 42500.0, "high": 43500.0, "low": 41800.0, "close": 43200.0, "volume": 12345.67},
            {"timestamp": "2024-01-14", "open": 41800.0, "high": 42600.0, "low": 41500.0, "close": 42000.0, "volume": 11000.0}
        ])));

        assert_eq!(quote.id, "BTC");
        assert_eq!(quote.symbol, "BTC");
        assert_eq!(quote.price, Some(43200.0));
        assert_eq!(quote.change_24h, Some(1200.0));
        let pct = quote.change_percent_24h.unwrap();
        assert!((pct - (1200.0 / 42000.0 * 100.0)).abs() < 1e-9);
        assert_eq!(quote.high_24h, Some(43500.0));
        assert_eq!(quote.low_24h, Some(41800.0));
        assert_eq!(quote.volume_24h, Some(12345.67));
    }

    #[test]
    fn series_to_quote_zero_prev_close_yields_zero_percent() {
        let quote = series_to_quote(series(serde_json::json!([
            {"timestamp": "2024-01-15", "open": 1.0, "high": 1.0, "low": 1.0, "close": 5.0, "volume": 1.0},
            {"timestamp": "2024-01-14", "open": 0.0, "high": 0.0, "low": 0.0, "close": 0.0, "volume": 1.0}
        ])));
        assert_eq!(quote.change_24h, Some(5.0));
        assert_eq!(quote.change_percent_24h, Some(0.0));
    }

    #[test]
    fn series_to_quote_single_entry_has_price_but_no_change() {
        let quote = series_to_quote(series(serde_json::json!([
            {"timestamp": "2024-01-15", "open": 42500.0, "high": 43500.0, "low": 41800.0, "close": 43200.0, "volume": 12345.67}
        ])));
        assert_eq!(quote.price, Some(43200.0));
        assert!(quote.change_24h.is_none());
        assert!(quote.change_percent_24h.is_none());
        assert_eq!(quote.volume_24h, Some(12345.67));
    }

    #[test]
    fn series_to_quote_empty_series_yields_no_values() {
        let quote = series_to_quote(series(serde_json::json!([])));
        assert_eq!(quote.id, "BTC");
        assert!(quote.price.is_none());
        assert!(quote.change_24h.is_none());
        assert!(quote.volume_24h.is_none());
    }

    /// Mocked HTTP → `parse_crypto_series` → `series_to_quote`, covering the
    /// full `fetch_crypto_quote_response` pipeline without a network call.
    #[tokio::test]
    async fn test_crypto_daily_to_canonical_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("function".into(), "DIGITAL_CURRENCY_DAILY".into()),
                mockito::Matcher::UrlEncoded("symbol".into(), "BTC".into()),
                mockito::Matcher::UrlEncoded("market".into(), "USD".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "Meta Data": {
                        "1. Information": "Daily Prices and Volumes for Digital Currency",
                        "2. Digital Currency Code": "BTC",
                        "6. Last Refreshed": "2024-01-15 00:00:00"
                    },
                    "Time Series (Digital Currency Daily)": {
                        "2024-01-15": {
                            "1. open": "42500.00", "2. high": "43500.00",
                            "3. low": "41800.00", "4. close": "43200.00",
                            "5. volume": "12345.67"
                        },
                        "2024-01-14": {
                            "1. open": "41800.00", "2. high": "42600.00",
                            "3. low": "41500.00", "4. close": "42000.00",
                            "5. volume": "11000.00"
                        }
                    }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let json = client
            .get(
                "DIGITAL_CURRENCY_DAILY",
                &[("symbol", "BTC"), ("market", "USD")],
            )
            .await
            .unwrap();

        let series = parse_crypto_series(&json, "BTC", "USD").unwrap();
        assert_eq!(series.entries.len(), 2);
        assert_eq!(
            series.entries[0].timestamp, "2024-01-15",
            "sorted newest first"
        );

        let quote = series_to_quote(series);
        assert_eq!(quote.symbol, "BTC");
        assert_eq!(quote.price, Some(43200.0));
        assert_eq!(quote.change_24h, Some(1200.0));
        assert_eq!(quote.volume_24h, Some(12345.67));
    }
}
