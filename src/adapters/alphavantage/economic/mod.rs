//! Economic indicator endpoints: GDP, CPI, Treasury yield, unemployment, etc.

#![allow(dead_code)]
use crate::error::{FinanceError, Result};

use super::build_client;
use super::models::*;

/// Helper to parse an economic indicator response.
fn parse_economic_series(json: &serde_json::Value) -> Result<EconomicSeriesDTO> {
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
            context: "Missing data array in economic indicator response".to_string(),
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
            Some(EconomicDataPointDTO { date, value })
        })
        .collect();

    Ok(EconomicSeriesDTO {
        name,
        interval,
        unit,
        data,
    })
}

/// Fetch a generic economic indicator by function name.
async fn fetch_indicator(function: &str, interval: Option<&str>) -> Result<EconomicSeriesDTO> {
    let client = build_client()?;
    let params: Vec<(&str, &str)> = match interval {
        Some(i) => vec![("interval", i)],
        None => vec![],
    };
    let json = client.get(function, &params).await?;
    parse_economic_series(&json)
}

/// Fetch Real GDP data.
///
/// * `interval` - `None` for annual (default), or `"quarterly"`, `"annual"`
pub async fn real_gdp(interval: Option<&str>) -> Result<EconomicSeriesDTO> {
    fetch_indicator("REAL_GDP", interval).await
}

/// Fetch Real GDP per capita.
pub async fn real_gdp_per_capita() -> Result<EconomicSeriesDTO> {
    fetch_indicator("REAL_GDP_PER_CAPITA", None).await
}

/// Fetch US Treasury yield for a given maturity.
///
/// * `interval` - `None` for monthly (default), or `"daily"`, `"weekly"`, `"monthly"`
/// * `maturity` - `None` for 10-year (default), or `"3month"`, `"2year"`, `"5year"`, `"10year"`, `"30year"`
pub async fn treasury_yield(
    interval: Option<&str>,
    maturity: Option<&str>,
) -> Result<EconomicSeriesDTO> {
    let client = build_client()?;
    let mut params: Vec<(&str, &str)> = Vec::new();
    if let Some(i) = interval {
        params.push(("interval", i));
    }
    if let Some(m) = maturity {
        params.push(("maturity", m));
    }
    let json = client.get("TREASURY_YIELD", &params).await?;
    parse_economic_series(&json)
}

/// Fetch Federal Funds (interest) Rate.
///
/// * `interval` - `None` for monthly (default), or `"daily"`, `"weekly"`, `"monthly"`
pub async fn federal_funds_rate(interval: Option<&str>) -> Result<EconomicSeriesDTO> {
    fetch_indicator("FEDERAL_FUNDS_RATE", interval).await
}

/// Fetch Consumer Price Index (CPI).
///
/// * `interval` - `None` for monthly (default), or `"semiannual"`
pub async fn cpi(interval: Option<&str>) -> Result<EconomicSeriesDTO> {
    fetch_indicator("CPI", interval).await
}

/// Fetch annual inflation rates.
pub async fn inflation() -> Result<EconomicSeriesDTO> {
    fetch_indicator("INFLATION", None).await
}

/// Fetch retail sales data.
pub async fn retail_sales() -> Result<EconomicSeriesDTO> {
    fetch_indicator("RETAIL_SALES", None).await
}

/// Fetch durable goods orders.
pub async fn durables() -> Result<EconomicSeriesDTO> {
    fetch_indicator("DURABLES", None).await
}

/// Fetch US unemployment rate.
pub async fn unemployment() -> Result<EconomicSeriesDTO> {
    fetch_indicator("UNEMPLOYMENT", None).await
}

/// Fetch nonfarm payroll data.
pub async fn nonfarm_payroll() -> Result<EconomicSeriesDTO> {
    fetch_indicator("NONFARM_PAYROLL", None).await
}

// ============================================================================
// Canonical model conversion functions
// ============================================================================

/// Fetch canonical EconomicSeries by series_id.
pub async fn fetch_economic_series_response(
    series_id: &str,
) -> Result<crate::models::economic::EconomicSeries> {
    let (dto, func_name) = match series_id.to_uppercase().as_str() {
        "REAL_GDP" => (real_gdp(None).await?, "REAL_GDP"),
        "REAL_GDP_PER_CAPITA" => (real_gdp_per_capita().await?, "REAL_GDP_PER_CAPITA"),
        "TREASURY_YIELD" => (treasury_yield(None, None).await?, "TREASURY_YIELD"),
        "FEDERAL_FUNDS_RATE" => (federal_funds_rate(None).await?, "FEDERAL_FUNDS_RATE"),
        "CPI" => (cpi(None).await?, "CPI"),
        "INFLATION" => (inflation().await?, "INFLATION"),
        "RETAIL_SALES" => (retail_sales().await?, "RETAIL_SALES"),
        "DURABLES" => (durables().await?, "DURABLES"),
        "UNEMPLOYMENT" => (unemployment().await?, "UNEMPLOYMENT"),
        "NONFARM_PAYROLL" => (nonfarm_payroll().await?, "NONFARM_PAYROLL"),
        _ => {
            return Err(crate::error::FinanceError::InvalidParameter {
                param: "series_id".to_string(),
                reason: format!("Unknown Alpha Vantage economic series: {series_id}"),
            });
        }
    };

    Ok(dto_to_series(func_name, dto))
}

/// Map an economic series DTO to the canonical
/// [`EconomicSeries`](crate::models::economic::EconomicSeries); empty
/// unit/interval strings become `None`.
fn dto_to_series(
    func_name: &str,
    dto: EconomicSeriesDTO,
) -> crate::models::economic::EconomicSeries {
    crate::models::economic::EconomicSeries {
        series_id: func_name.to_string(),
        title: Some(dto.name),
        units: if dto.unit.is_empty() {
            None
        } else {
            Some(dto.unit)
        },
        frequency: if dto.interval.is_empty() {
            None
        } else {
            Some(dto.interval)
        },
        observations: dto
            .data
            .into_iter()
            .map(|dp| crate::models::economic::MacroObservation {
                date: dp.date,
                value: dp.value,
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dto_to_series_maps_fields_and_observations() {
        let dto: EconomicSeriesDTO = serde_json::from_value(serde_json::json!({
            "name": "Real Gross Domestic Product",
            "interval": "annual",
            "unit": "billions of dollars",
            "data": [
                {"date": "2023-01-01", "value": 22403.435},
                {"date": "2022-01-01", "value": null}
            ]
        }))
        .unwrap();

        let series = dto_to_series("REAL_GDP", dto);
        assert_eq!(series.series_id, "REAL_GDP");
        assert_eq!(series.title.as_deref(), Some("Real Gross Domestic Product"));
        assert_eq!(series.units.as_deref(), Some("billions of dollars"));
        assert_eq!(series.frequency.as_deref(), Some("annual"));
        assert_eq!(series.observations.len(), 2);
        assert_eq!(series.observations[0].date, "2023-01-01");
        assert_eq!(series.observations[0].value, Some(22403.435));
        assert_eq!(series.observations[1].value, None);
    }

    #[test]
    fn dto_to_series_empty_unit_and_interval_become_none() {
        let dto: EconomicSeriesDTO = serde_json::from_value(serde_json::json!({
            "name": "unknown",
            "interval": "",
            "unit": "",
            "data": []
        }))
        .unwrap();
        let series = dto_to_series("CPI", dto);
        assert!(series.units.is_none());
        assert!(series.frequency.is_none());
        assert!(series.observations.is_empty());
    }

    #[tokio::test]
    async fn fetch_economic_series_rejects_unknown_series_id() {
        let err = fetch_economic_series_response("NOT_A_SERIES")
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            crate::error::FinanceError::InvalidParameter { .. }
        ));
    }

    /// Mocked HTTP → `parse_economic_series` → `dto_to_series`, covering the
    /// full `fetch_economic_series_response` pipeline without a network call.
    #[tokio::test]
    async fn test_real_gdp_to_canonical_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/")
            .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
                "function".into(),
                "REAL_GDP".into(),
            )]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "name": "Real Gross Domestic Product",
                    "interval": "annual",
                    "unit": "billions of dollars",
                    "data": [
                        { "date": "2023-01-01", "value": "22403.435" },
                        { "date": "2022-01-01", "value": "." }
                    ]
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let json = client.get("REAL_GDP", &[]).await.unwrap();
        let dto = parse_economic_series(&json).unwrap();
        assert_eq!(dto.data.len(), 2);
        assert!(dto.data[1].value.is_none(), "\".\" parses to None");

        let series = dto_to_series("REAL_GDP", dto);
        assert_eq!(series.series_id, "REAL_GDP");
        assert_eq!(series.frequency.as_deref(), Some("annual"));
        assert_eq!(series.observations[0].value, Some(22403.435));
        assert_eq!(series.observations[1].value, None);
    }
}
