//! GraphQL type for risk analytics.

use async_graphql::SimpleObject;
use serde::Deserialize;

/// Risk/performance summary for a symbol.
///
/// Mirrors `finance_query::risk::RiskSummary`, which has no serde rename of
/// its own (its JSON keys are plain snake_case matching its Rust field
/// names) — so this type must NOT rename for deserialization either, even
/// though the GraphQL-facing field names are camelCase.
#[derive(SimpleObject, Deserialize, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct GqlRiskSummary {
    #[serde(alias = "var_95")]
    pub var_95: f64,
    #[serde(alias = "var_99")]
    pub var_99: f64,
    #[serde(alias = "parametric_var_95")]
    pub parametric_var_95: f64,
    #[serde(alias = "cvar_95")]
    pub cvar_95: f64,
    #[serde(alias = "cvar_99")]
    pub cvar_99: f64,
    #[serde(alias = "parametric_cvar_95")]
    pub parametric_cvar_95: f64,
    pub omega: f64,
    pub kelly: f64,
    pub sharpe: Option<f64>,
    pub sortino: Option<f64>,
    pub calmar: Option<f64>,
    pub beta: Option<f64>,
    #[serde(alias = "max_drawdown")]
    pub max_drawdown: f64,
    #[serde(alias = "max_drawdown_recovery_periods")]
    pub max_drawdown_recovery_periods: Option<f64>,
    #[serde(alias = "ulcer_index")]
    pub ulcer_index: f64,
    #[serde(alias = "information_ratio")]
    pub information_ratio: Option<f64>,
    #[serde(alias = "tracking_error")]
    pub tracking_error: Option<f64>,
}
