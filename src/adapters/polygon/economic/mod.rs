//! Economic indicator endpoints: inflation, labor market, treasury yields.
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::models::economic::{EconomicSeries, MacroObservation};

use super::build_client;
use super::models::PaginatedResponseDTO;

/// Economic data point.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EconomicDataPointDTO {
    /// Date.
    pub date: Option<String>,
    /// Value.
    pub value: Option<f64>,
    /// Period.
    pub period: Option<String>,
}

/// Fetch inflation data.
pub async fn inflation(
    params: &[(&str, &str)],
) -> Result<PaginatedResponseDTO<EconomicDataPointDTO>> {
    let client = build_client()?;
    client.get("/v1/indicators/economy/inflation", params).await
}

/// Fetch inflation expectations.
pub async fn inflation_expectations(
    params: &[(&str, &str)],
) -> Result<PaginatedResponseDTO<EconomicDataPointDTO>> {
    let client = build_client()?;
    client
        .get("/v1/indicators/economy/inflation-expectations", params)
        .await
}

/// Fetch labor market data (unemployment, participation, earnings, job openings).
pub async fn labor_market(
    params: &[(&str, &str)],
) -> Result<PaginatedResponseDTO<EconomicDataPointDTO>> {
    let client = build_client()?;
    client
        .get("/v1/indicators/economy/labor-market", params)
        .await
}

/// Fetch US Treasury yield data.
pub async fn treasury_yields(
    params: &[(&str, &str)],
) -> Result<PaginatedResponseDTO<EconomicDataPointDTO>> {
    let client = build_client()?;
    client
        .get("/v1/indicators/economy/treasury-yields", params)
        .await
}

/// Fetch economic series (canonical) by series ID.
pub async fn fetch_economic_series_response(series_id: &str) -> Result<EconomicSeries> {
    use crate::error::FinanceError;
    let params: &[(&str, &str)] = &[];
    let paginated = match series_id {
        "inflation" => inflation(params).await?,
        "inflation_expectations" => inflation_expectations(params).await?,
        "labor_market" => labor_market(params).await?,
        "treasury_yields" => treasury_yields(params).await?,
        other => {
            return Err(FinanceError::InvalidParameter {
                param: "series_id".to_string(),
                reason: format!("Unknown economic series: {other}"),
            });
        }
    };
    Ok(points_to_series(series_id, paginated.results))
}

/// Map economic data points to the canonical [`EconomicSeries`].
fn points_to_series(series_id: &str, results: Option<Vec<EconomicDataPointDTO>>) -> EconomicSeries {
    EconomicSeries {
        series_id: series_id.to_string(),
        title: None,
        units: None,
        frequency: None,
        observations: results
            .unwrap_or_default()
            .into_iter()
            .map(|d| MacroObservation {
                date: d.date.unwrap_or_default(),
                value: d.value,
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn points_to_series_maps_observations() {
        let points: Vec<EconomicDataPointDTO> = serde_json::from_value(serde_json::json!([
            {"date": "2024-01-01", "value": 3.4, "period": "monthly"},
            {"date": "2024-02-01", "value": 3.2}
        ]))
        .unwrap();

        let series = points_to_series("inflation", Some(points));
        assert_eq!(series.series_id, "inflation");
        assert_eq!(series.observations.len(), 2);
        assert_eq!(series.observations[0].date, "2024-01-01");
        assert_eq!(series.observations[0].value, Some(3.4));
    }

    #[test]
    fn points_to_series_defaults_missing_date_and_value() {
        let points: Vec<EconomicDataPointDTO> =
            serde_json::from_value(serde_json::json!([{"period": "monthly"}])).unwrap();
        let series = points_to_series("labor_market", Some(points));
        assert_eq!(series.observations.len(), 1);
        assert_eq!(series.observations[0].date, "");
        assert_eq!(series.observations[0].value, None);
    }

    #[test]
    fn points_to_series_none_results_yields_empty() {
        let series = points_to_series("treasury_yields", None);
        assert!(series.observations.is_empty());
    }

    #[tokio::test]
    async fn fetch_economic_series_rejects_unknown_series_id() {
        let err = fetch_economic_series_response("not-a-series")
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            crate::error::FinanceError::InvalidParameter { .. }
        ));
    }
}
