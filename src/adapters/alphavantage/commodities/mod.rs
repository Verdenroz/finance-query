//! Commodity price endpoints: oil, gas, metals, agriculture, and composite index.

#![allow(dead_code)]
use crate::error::{FinanceError, Result};

use super::build_client;
use super::models::*;

/// Helper to parse a commodity/economic-style response.
fn parse_commodity_series(json: &serde_json::Value) -> Result<CommoditySeriesDTO> {
    let name = json
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let interval = json
        .get("interval")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let unit = json
        .get("unit")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let data = json
        .get("data")
        .and_then(|v| v.as_array())
        .ok_or_else(|| FinanceError::ResponseStructureError {
            field: "data".to_string(),
            context: "Missing data array in commodity response".to_string(),
        })?
        .iter()
        .filter_map(|d| {
            let date = d.get("date")?.as_str()?.to_string();
            let raw = d.get("value")?.as_str()?;
            let value = if raw == "." || raw == "None" || raw.is_empty() {
                None
            } else {
                raw.parse::<f64>().ok()
            };
            Some(CommodityDataPointDTO { date, value })
        })
        .collect();

    Ok(CommoditySeriesDTO {
        name,
        interval,
        unit,
        data,
    })
}

/// Fetch a commodity series by function name and optional interval.
pub(crate) async fn fetch_commodity(
    function: &str,
    interval: Option<&str>,
) -> Result<CommoditySeriesDTO> {
    let client = build_client()?;
    let params: Vec<(&str, &str)> = match interval {
        Some(i) => vec![("interval", i)],
        None => vec![],
    };
    let json = client.get(function, &params).await?;
    parse_commodity_series(&json)
}

/// Fetch WTI crude oil prices.
///
/// * `interval` - `None` for monthly (default), or `"daily"`, `"weekly"`, `"monthly"`
pub async fn commodity_wti(interval: Option<&str>) -> Result<CommoditySeriesDTO> {
    fetch_commodity("WTI", interval).await
}

/// Fetch Brent crude oil prices.
pub async fn commodity_brent(interval: Option<&str>) -> Result<CommoditySeriesDTO> {
    fetch_commodity("BRENT", interval).await
}

/// Fetch natural gas prices.
pub async fn commodity_natural_gas(interval: Option<&str>) -> Result<CommoditySeriesDTO> {
    fetch_commodity("NATURAL_GAS", interval).await
}

/// Fetch copper prices.
pub async fn commodity_copper(interval: Option<&str>) -> Result<CommoditySeriesDTO> {
    fetch_commodity("COPPER", interval).await
}

/// Fetch aluminum prices.
pub async fn commodity_aluminum(interval: Option<&str>) -> Result<CommoditySeriesDTO> {
    fetch_commodity("ALUMINUM", interval).await
}

/// Fetch wheat prices.
pub async fn commodity_wheat(interval: Option<&str>) -> Result<CommoditySeriesDTO> {
    fetch_commodity("WHEAT", interval).await
}

/// Fetch corn prices.
pub async fn commodity_corn(interval: Option<&str>) -> Result<CommoditySeriesDTO> {
    fetch_commodity("CORN", interval).await
}

/// Fetch cotton prices.
pub async fn commodity_cotton(interval: Option<&str>) -> Result<CommoditySeriesDTO> {
    fetch_commodity("COTTON", interval).await
}

/// Fetch sugar prices.
pub async fn commodity_sugar(interval: Option<&str>) -> Result<CommoditySeriesDTO> {
    fetch_commodity("SUGAR", interval).await
}

/// Fetch coffee prices.
pub async fn commodity_coffee(interval: Option<&str>) -> Result<CommoditySeriesDTO> {
    fetch_commodity("COFFEE", interval).await
}

/// Fetch the global commodities index.
pub async fn commodity_all_commodities(interval: Option<&str>) -> Result<CommoditySeriesDTO> {
    fetch_commodity("ALL_COMMODITIES", interval).await
}

// ============================================================================
// Canonical model conversion functions
// ============================================================================

/// Fetch canonical CommodityQuote for a commodity symbol.
pub async fn fetch_commodities_quote_response(
    symbol: &str,
) -> Result<crate::models::commodities::CommodityQuote> {
    let series = fetch_commodity(symbol, Some("daily")).await?;
    Ok(series_to_quote(symbol, series))
}

/// Map a commodity series to the canonical
/// [`CommodityQuote`](crate::models::commodities::CommodityQuote), deriving
/// change from the two most recent data points.
fn series_to_quote(
    symbol: &str,
    series: CommoditySeriesDTO,
) -> crate::models::commodities::CommodityQuote {
    let (price, change, change_percent) = match series.data.len() {
        0 => (None, None, None),
        1 => (series.data[0].value, None, None),
        _ => {
            let latest = series.data[0].value;
            let prev = series.data[1].value;
            match (latest, prev) {
                (Some(l), Some(p)) if p != 0.0 => {
                    let chg = l - p;
                    let pct = (chg / p) * 100.0;
                    (Some(l), Some(chg), Some(pct))
                }
                (Some(l), _) => (Some(l), None, None),
                _ => (None, None, None),
            }
        }
    };
    crate::models::commodities::CommodityQuote {
        symbol: symbol.to_string(),
        name: Some(series.name),
        unit: Some(series.unit),
        price,
        change,
        change_percent,
        timestamp: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_commodity_series() {
        let json = serde_json::json!({
            "name": "Crude Oil Prices - West Texas Intermediate (WTI)",
            "interval": "monthly",
            "unit": "dollars per barrel",
            "data": [
                { "date": "2024-01-01", "value": "72.68" },
                { "date": "2023-12-01", "value": "71.65" },
                { "date": "2023-11-01", "value": "." }
            ]
        });

        let series = parse_commodity_series(&json).unwrap();
        assert_eq!(
            series.name,
            "Crude Oil Prices - West Texas Intermediate (WTI)"
        );
        assert_eq!(series.interval, "monthly");
        assert_eq!(series.unit, "dollars per barrel");
        assert_eq!(series.data.len(), 3);
        assert!((series.data[0].value.unwrap() - 72.68).abs() < 0.01);
        assert!(series.data[2].value.is_none()); // "." → None
    }

    #[test]
    fn test_parse_commodity_missing_data() {
        let json = serde_json::json!({"name": "WTI"});
        assert!(parse_commodity_series(&json).is_err());
    }

    #[tokio::test]
    async fn test_commodity_wti_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/")
            .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
                "function".into(),
                "WTI".into(),
            )]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "name": "WTI",
                    "interval": "monthly",
                    "unit": "dollars per barrel",
                    "data": [
                        { "date": "2024-01-01", "value": "72.68" }
                    ]
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let json = client.get("WTI", &[]).await.unwrap();
        let series = parse_commodity_series(&json).unwrap();
        assert_eq!(series.data.len(), 1);
        assert!((series.data[0].value.unwrap() - 72.68).abs() < 0.01);

        // Mocked HTTP → parse → canonical CommodityQuote, covering the full
        // fetch_commodities_quote_response pipeline without a network call.
        let quote = series_to_quote("WTI", series);
        assert_eq!(quote.symbol, "WTI");
        assert_eq!(quote.unit.as_deref(), Some("dollars per barrel"));
        assert_eq!(quote.price, Some(72.68));
        assert!(quote.change.is_none(), "single point has no change");
    }

    fn wti_series(data: serde_json::Value) -> CommoditySeriesDTO {
        serde_json::from_value(serde_json::json!({
            "name": "WTI",
            "interval": "daily",
            "unit": "dollars per barrel",
            "data": data
        }))
        .unwrap()
    }

    #[test]
    fn series_to_quote_derives_change_from_two_latest_points() {
        let quote = series_to_quote(
            "WTI",
            wti_series(serde_json::json!([
                {"date": "2024-01-02", "value": 74.0},
                {"date": "2024-01-01", "value": 72.0}
            ])),
        );
        assert_eq!(quote.symbol, "WTI");
        assert_eq!(quote.name.as_deref(), Some("WTI"));
        assert_eq!(quote.unit.as_deref(), Some("dollars per barrel"));
        assert_eq!(quote.price, Some(74.0));
        assert_eq!(quote.change, Some(2.0));
        let pct = quote.change_percent.unwrap();
        assert!((pct - (2.0 / 72.0 * 100.0)).abs() < 1e-9);
    }

    #[test]
    fn series_to_quote_single_point_has_price_but_no_change() {
        let quote = series_to_quote(
            "WTI",
            wti_series(serde_json::json!([{"date": "2024-01-02", "value": 74.0}])),
        );
        assert_eq!(quote.price, Some(74.0));
        assert!(quote.change.is_none());
    }

    #[test]
    fn series_to_quote_missing_prev_value_yields_price_only() {
        let quote = series_to_quote(
            "WTI",
            wti_series(serde_json::json!([
                {"date": "2024-01-02", "value": 74.0},
                {"date": "2024-01-01", "value": null}
            ])),
        );
        assert_eq!(quote.price, Some(74.0));
        assert!(quote.change.is_none());
        assert!(quote.change_percent.is_none());
    }

    #[test]
    fn series_to_quote_zero_prev_value_yields_price_only() {
        let quote = series_to_quote(
            "WTI",
            wti_series(serde_json::json!([
                {"date": "2024-01-02", "value": 74.0},
                {"date": "2024-01-01", "value": 0.0}
            ])),
        );
        assert_eq!(quote.price, Some(74.0));
        assert!(quote.change.is_none(), "division by zero prev is avoided");
    }

    #[test]
    fn series_to_quote_empty_series_yields_no_values() {
        let quote = series_to_quote("WTI", wti_series(serde_json::json!([])));
        assert!(quote.price.is_none());
        assert!(quote.change.is_none());
    }
}
