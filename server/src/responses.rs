//! REST and WebSocket response envelope types that are not GraphQL wire
//! types.
//!
//! Most REST endpoints serialize `graphql::types::*` shapes directly; the
//! types here cover the rest — system endpoints, the shared error body, and
//! open-schema envelopes for endpoints whose payload varies by path param.
//! They live in the lib so `cargo soothfast spec gen` can resolve them from
//! the lib's rustdoc JSON.

use finance_query::streaming::{AlertEvent, PriceUpdate};
use serde::Serialize;

/// Body of `GET /v2/health`.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthResponse {
    /// Always "healthy" when the server can respond at all.
    pub status: String,
    /// Server crate version.
    pub version: String,
    /// RFC 3339 timestamp of the response.
    pub timestamp: String,
    /// Data-provider attribution notices.
    pub notices: &'static [&'static str],
}

/// Body of `GET /v2/ping`.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PingResponse {
    /// Always "pong".
    pub message: String,
}

/// Body of `GET /v2/metrics`: Prometheus text exposition format
/// (served as `text/plain; version=0.0.4`, not JSON).
#[derive(Serialize)]
#[serde(transparent)]
pub struct MetricsText(pub String);

/// Generic REST error body, emitted on 4xx/5xx by the GraphQL bridge and by
/// handlers rejecting invalid path/query input.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiError {
    /// Human-readable error message.
    pub error: String,
    /// HTTP status code, duplicated in the body.
    pub status: u16,
}

/// Body of `GET /v2/analysis/{symbol}/{type}` — the shape depends on the
/// `{type}` path param (recommendations, upgrades-downgrades,
/// earnings-estimate, earnings-history), so the schema is deliberately open.
#[derive(Serialize)]
#[serde(transparent)]
pub struct AnalysisResponse(pub serde_json::Value);

/// Body of `GET /v2/holders/{symbol}/{type}` — the shape depends on the
/// `{type}` path param (major, institutional, mutualfund,
/// insider-transactions, insider-purchases, insider-roster), so the schema
/// is deliberately open.
#[derive(Serialize)]
#[serde(transparent)]
pub struct HoldersResponse(pub serde_json::Value);

/// Outbound message on `/v2/stream`: a live price tick, or — while a
/// client's alert rules are active — a fired alert event instead of the raw
/// tick. Schema-only: `deliver()` in `handlers::stream` sends each variant's
/// JSON directly rather than constructing this type.
#[derive(Serialize)]
#[serde(untagged)]
pub enum PriceStreamMessage {
    Tick(PriceUpdate),
    Alert(AlertEvent),
}
