//! GraphQL types for FRED economic series and US Treasury yield curve data.

use async_graphql::SimpleObject;
use serde::Deserialize;

/// A single observation in a FRED data series.
#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(default)]
pub struct GqlMacroObservation {
    pub date: String,
    pub value: Option<f64>,
}

/// A FRED macro-economic time series with all its observations.
#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(default)]
pub struct GqlMacroSeries {
    pub id: String,
    pub observations: Vec<GqlMacroObservation>,
}

/// One day of US Treasury yield curve rates. Maturities with no published
/// rate on a given date are `null`.
#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(default)]
pub struct GqlTreasuryYield {
    pub date: String,
    pub y1m: Option<f64>,
    pub y2m: Option<f64>,
    pub y3m: Option<f64>,
    pub y4m: Option<f64>,
    pub y6m: Option<f64>,
    pub y1: Option<f64>,
    pub y2: Option<f64>,
    pub y3: Option<f64>,
    pub y5: Option<f64>,
    pub y7: Option<f64>,
    pub y10: Option<f64>,
    pub y20: Option<f64>,
    pub y30: Option<f64>,
}
