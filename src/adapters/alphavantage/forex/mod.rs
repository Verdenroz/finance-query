//! Forex (foreign exchange) endpoints: exchange rates and FX time series.

#![allow(dead_code)]
use crate::error::{FinanceError, Result};

use super::build_client;
use super::models::*;

/// Fetch the real-time exchange rate between two currencies.
pub async fn exchange_rate(from_currency: &str, to_currency: &str) -> Result<ExchangeRateDTO> {
    let client = build_client()?;
    let json = client
        .get(
            "CURRENCY_EXCHANGE_RATE",
            &[
                ("from_currency", from_currency),
                ("to_currency", to_currency),
            ],
        )
        .await?;
    parse_exchange_rate(&json, from_currency, to_currency)
}

/// Parse a `CURRENCY_EXCHANGE_RATE` response into an [`ExchangeRateDTO`].
fn parse_exchange_rate(
    json: &serde_json::Value,
    from_currency: &str,
    to_currency: &str,
) -> Result<ExchangeRateDTO> {
    let rate = json.get("Realtime Currency Exchange Rate").ok_or_else(|| {
        FinanceError::ResponseStructureError {
            field: "Realtime Currency Exchange Rate".to_string(),
            context: "Missing exchange rate data in response".to_string(),
        }
    })?;

    Ok(ExchangeRateDTO {
        from_currency_code: rate
            .get("1. From_Currency Code")
            .and_then(|v| v.as_str())
            .unwrap_or(from_currency)
            .to_string(),
        from_currency_name: rate
            .get("2. From_Currency Name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        to_currency_code: rate
            .get("3. To_Currency Code")
            .and_then(|v| v.as_str())
            .unwrap_or(to_currency)
            .to_string(),
        to_currency_name: rate
            .get("4. To_Currency Name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        exchange_rate: rate
            .get("5. Exchange Rate")
            .and_then(|v| v.as_str()?.parse().ok())
            .unwrap_or(0.0),
        last_refreshed: rate
            .get("6. Last Refreshed")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        bid_price: rate
            .get("8. Bid Price")
            .and_then(|v| v.as_str()?.parse().ok())
            .unwrap_or(0.0),
        ask_price: rate
            .get("9. Ask Price")
            .and_then(|v| v.as_str()?.parse().ok())
            .unwrap_or(0.0),
    })
}

/// Helper to parse FX time series responses.
fn parse_fx_series(
    json: &serde_json::Value,
    from_symbol: &str,
    to_symbol: &str,
) -> Result<ForexTimeSeriesDTO> {
    let meta = json.get("Meta Data");
    let last_refreshed = meta
        .and_then(|m| {
            m.get("5. Last Refreshed")
                .or_else(|| m.get("4. Last Refreshed"))
        })
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    // Find the time series key (varies by endpoint)
    let series = json
        .as_object()
        .and_then(|obj| {
            obj.iter()
                .find(|(k, _)| k.starts_with("Time Series"))
                .map(|(_, v)| v)
        })
        .and_then(|v| v.as_object())
        .ok_or_else(|| FinanceError::ResponseStructureError {
            field: "Time Series FX".to_string(),
            context: "Missing FX time series data".to_string(),
        })?;

    let mut entries: Vec<ForexEntryDTO> = series
        .iter()
        .filter_map(|(timestamp, values)| {
            Some(ForexEntryDTO {
                timestamp: timestamp.clone(),
                open: values.get("1. open")?.as_str()?.parse().ok()?,
                high: values.get("2. high")?.as_str()?.parse().ok()?,
                low: values.get("3. low")?.as_str()?.parse().ok()?,
                close: values.get("4. close")?.as_str()?.parse().ok()?,
            })
        })
        .collect();

    entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    Ok(ForexTimeSeriesDTO {
        from_symbol: from_symbol.to_string(),
        to_symbol: to_symbol.to_string(),
        last_refreshed,
        entries,
    })
}

/// Fetch intraday FX time series.
pub async fn fx_intraday(
    from_symbol: &str,
    to_symbol: &str,
    interval: AvInterval,
    output_size: Option<OutputSize>,
) -> Result<ForexTimeSeriesDTO> {
    let client = build_client()?;
    let mut params = vec![
        ("from_symbol", from_symbol),
        ("to_symbol", to_symbol),
        ("interval", interval.as_str()),
    ];
    let os_str;
    if let Some(os) = output_size {
        os_str = os.as_str().to_string();
        params.push(("outputsize", &os_str));
    }
    let json = client.get("FX_INTRADAY", &params).await?;
    parse_fx_series(&json, from_symbol, to_symbol)
}

/// Fetch daily FX time series.
pub async fn fx_daily(
    from_symbol: &str,
    to_symbol: &str,
    output_size: Option<OutputSize>,
) -> Result<ForexTimeSeriesDTO> {
    let client = build_client()?;
    let mut params = vec![("from_symbol", from_symbol), ("to_symbol", to_symbol)];
    let os_str;
    if let Some(os) = output_size {
        os_str = os.as_str().to_string();
        params.push(("outputsize", &os_str));
    }
    let json = client.get("FX_DAILY", &params).await?;
    parse_fx_series(&json, from_symbol, to_symbol)
}

/// Fetch weekly FX time series.
pub async fn fx_weekly(from_symbol: &str, to_symbol: &str) -> Result<ForexTimeSeriesDTO> {
    let client = build_client()?;
    let json = client
        .get(
            "FX_WEEKLY",
            &[("from_symbol", from_symbol), ("to_symbol", to_symbol)],
        )
        .await?;
    parse_fx_series(&json, from_symbol, to_symbol)
}

/// Fetch monthly FX time series.
pub async fn fx_monthly(from_symbol: &str, to_symbol: &str) -> Result<ForexTimeSeriesDTO> {
    let client = build_client()?;
    let json = client
        .get(
            "FX_MONTHLY",
            &[("from_symbol", from_symbol), ("to_symbol", to_symbol)],
        )
        .await?;
    parse_fx_series(&json, from_symbol, to_symbol)
}

// ============================================================================
// Canonical model conversion functions
// ============================================================================

/// Fetch canonical ForexQuote for a currency pair.
pub async fn fetch_forex_quote_response(
    from: &str,
    to: &str,
) -> Result<crate::models::forex::ForexQuote> {
    let rate = exchange_rate(from, to).await?;
    Ok(rate_to_canonical(rate))
}

/// Map an exchange-rate DTO to the canonical [`ForexQuote`](crate::models::forex::ForexQuote);
/// `price` uses the bid.
fn rate_to_canonical(rate: ExchangeRateDTO) -> crate::models::forex::ForexQuote {
    let timestamp = parse_av_datetime(&rate.last_refreshed);
    let symbol = format!(
        "{}{}",
        rate.from_currency_code.to_uppercase(),
        rate.to_currency_code.to_uppercase()
    );
    crate::models::forex::ForexQuote {
        symbol,
        base_currency: Some(rate.from_currency_code),
        quote_currency: Some(rate.to_currency_code),
        bid: Some(rate.bid_price),
        ask: Some(rate.ask_price),
        price: Some(rate.bid_price),
        change: None,
        change_percent: None,
        timestamp,
    }
}

/// Parse an Alpha Vantage datetime string (e.g. "2024-01-15 14:30:00") to a Unix timestamp.
fn parse_av_datetime(dt_str: &str) -> Option<i64> {
    if dt_str.is_empty() {
        return None;
    }
    chrono::NaiveDateTime::parse_from_str(dt_str, "%Y-%m-%d %H:%M:%S")
        .ok()
        .map(|dt| dt.and_utc().timestamp())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_to_canonical_maps_bid_ask_and_uppercases_symbol() {
        let rate: ExchangeRateDTO = serde_json::from_value(serde_json::json!({
            "from_currency_code": "eur",
            "from_currency_name": "Euro",
            "to_currency_code": "usd",
            "to_currency_name": "United States Dollar",
            "exchange_rate": 1.1051,
            "last_refreshed": "2024-01-15 14:30:00",
            "bid_price": 1.1050,
            "ask_price": 1.1052
        }))
        .unwrap();

        let quote = rate_to_canonical(rate);
        assert_eq!(quote.symbol, "EURUSD");
        assert_eq!(quote.base_currency.as_deref(), Some("eur"));
        assert_eq!(quote.quote_currency.as_deref(), Some("usd"));
        assert_eq!(quote.bid, Some(1.1050));
        assert_eq!(quote.ask, Some(1.1052));
        assert_eq!(quote.price, Some(1.1050), "price uses the bid");
        // 2024-01-15T14:30:00Z.
        assert_eq!(quote.timestamp, Some(1_705_329_000));
    }

    #[test]
    fn rate_to_canonical_unparseable_datetime_yields_no_timestamp() {
        let rate: ExchangeRateDTO = serde_json::from_value(serde_json::json!({
            "from_currency_code": "EUR",
            "from_currency_name": "Euro",
            "to_currency_code": "USD",
            "to_currency_name": "United States Dollar",
            "exchange_rate": 1.1051,
            "last_refreshed": "not-a-datetime",
            "bid_price": 1.1050,
            "ask_price": 1.1052
        }))
        .unwrap();
        assert_eq!(rate_to_canonical(rate).timestamp, None);
    }

    #[test]
    fn parse_av_datetime_handles_valid_empty_and_garbage() {
        assert_eq!(
            parse_av_datetime("2024-01-15 14:30:00"),
            Some(1_705_329_000)
        );
        assert_eq!(parse_av_datetime(""), None);
        assert_eq!(parse_av_datetime("2024-01-15"), None);
    }

    #[test]
    fn parse_exchange_rate_missing_block_errors() {
        let err = parse_exchange_rate(&serde_json::json!({}), "EUR", "USD").unwrap_err();
        assert!(matches!(
            err,
            crate::error::FinanceError::ResponseStructureError { .. }
        ));
    }

    /// Mocked HTTP → `parse_exchange_rate` → `rate_to_canonical`, covering the
    /// full `fetch_forex_quote_response` pipeline without a network call.
    #[tokio::test]
    async fn test_exchange_rate_to_canonical_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("function".into(), "CURRENCY_EXCHANGE_RATE".into()),
                mockito::Matcher::UrlEncoded("from_currency".into(), "EUR".into()),
                mockito::Matcher::UrlEncoded("to_currency".into(), "USD".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "Realtime Currency Exchange Rate": {
                        "1. From_Currency Code": "EUR",
                        "2. From_Currency Name": "Euro",
                        "3. To_Currency Code": "USD",
                        "4. To_Currency Name": "United States Dollar",
                        "5. Exchange Rate": "1.10510000",
                        "6. Last Refreshed": "2024-01-15 14:30:00",
                        "7. Time Zone": "UTC",
                        "8. Bid Price": "1.10500000",
                        "9. Ask Price": "1.10520000"
                    }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let json = client
            .get(
                "CURRENCY_EXCHANGE_RATE",
                &[("from_currency", "EUR"), ("to_currency", "USD")],
            )
            .await
            .unwrap();

        let rate = parse_exchange_rate(&json, "EUR", "USD").unwrap();
        assert_eq!(rate.from_currency_code, "EUR");
        assert!((rate.exchange_rate - 1.1051).abs() < 1e-9);

        let quote = rate_to_canonical(rate);
        assert_eq!(quote.symbol, "EURUSD");
        assert_eq!(quote.bid, Some(1.1050));
        assert_eq!(quote.ask, Some(1.1052));
        assert_eq!(quote.price, Some(1.1050));
        assert_eq!(quote.timestamp, Some(1_705_329_000));
    }
}
