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
pub struct GqlRiskSummary {
    pub var_95: f64,
    pub var_99: f64,
    pub parametric_var_95: f64,
    pub sharpe: Option<f64>,
    pub sortino: Option<f64>,
    pub calmar: Option<f64>,
    pub beta: Option<f64>,
    pub max_drawdown: f64,
    pub max_drawdown_recovery_periods: Option<f64>,
}
