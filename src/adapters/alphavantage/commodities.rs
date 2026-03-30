//! Commodity price endpoints: oil, gas, metals, agriculture, and composite index.

use crate::error::{FinanceError, Result};

use super::build_client;
use super::models::*;

/// Helper to parse a commodity/economic-style response.
fn parse_commodity_series(json: &serde_json::Value) -> Result<CommoditySeries> {
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
            Some(CommodityDataPoint { date, value })
        })
        .collect();

    Ok(CommoditySeries {
        name,
        interval,
        unit,
        data,
    })
}

/// Fetch a commodity series by function name and optional interval.
async fn fetch_commodity(function: &str, interval: Option<&str>) -> Result<CommoditySeries> {
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
pub async fn commodity_wti(interval: Option<&str>) -> Result<CommoditySeries> {
    fetch_commodity("WTI", interval).await
}

/// Fetch Brent crude oil prices.
pub async fn commodity_brent(interval: Option<&str>) -> Result<CommoditySeries> {
    fetch_commodity("BRENT", interval).await
}

/// Fetch natural gas prices.
pub async fn commodity_natural_gas(interval: Option<&str>) -> Result<CommoditySeries> {
    fetch_commodity("NATURAL_GAS", interval).await
}

/// Fetch copper prices.
pub async fn commodity_copper(interval: Option<&str>) -> Result<CommoditySeries> {
    fetch_commodity("COPPER", interval).await
}

/// Fetch aluminum prices.
pub async fn commodity_aluminum(interval: Option<&str>) -> Result<CommoditySeries> {
    fetch_commodity("ALUMINUM", interval).await
}

/// Fetch wheat prices.
pub async fn commodity_wheat(interval: Option<&str>) -> Result<CommoditySeries> {
    fetch_commodity("WHEAT", interval).await
}

/// Fetch corn prices.
pub async fn commodity_corn(interval: Option<&str>) -> Result<CommoditySeries> {
    fetch_commodity("CORN", interval).await
}

/// Fetch cotton prices.
pub async fn commodity_cotton(interval: Option<&str>) -> Result<CommoditySeries> {
    fetch_commodity("COTTON", interval).await
}

/// Fetch sugar prices.
pub async fn commodity_sugar(interval: Option<&str>) -> Result<CommoditySeries> {
    fetch_commodity("SUGAR", interval).await
}

/// Fetch coffee prices.
pub async fn commodity_coffee(interval: Option<&str>) -> Result<CommoditySeries> {
    fetch_commodity("COFFEE", interval).await
}

/// Fetch the global commodities index.
pub async fn commodity_all_commodities(interval: Option<&str>) -> Result<CommoditySeries> {
    fetch_commodity("ALL_COMMODITIES", interval).await
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
        assert_eq!(series.name, "Crude Oil Prices - West Texas Intermediate (WTI)");
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
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("function".into(), "WTI".into()),
            ]))
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
    }
}
