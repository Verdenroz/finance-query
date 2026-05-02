//! TMX/Wall Street Horizon corporate events: earnings, dividends, conferences, splits.

use serde::{Deserialize, Serialize};

use crate::error::Result;

use super::build_client;
use super::models::PaginatedResponse;

/// Corporate event from Wall Street Horizon.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CorporateEvent {
    /// Ticker.
    pub ticker: Option<String>,
    /// Event type (e.g., `"earnings"`, `"dividend"`, `"conference"`, `"split"`).
    pub event_type: Option<String>,
    /// Event name.
    pub name: Option<String>,
    /// Event date.
    pub date: Option<String>,
    /// Event status.
    pub status: Option<String>,
    /// Additional details.
    #[serde(flatten)]
    pub details: std::collections::HashMap<String, serde_json::Value>,
}

/// Fetch corporate events.
pub async fn corporate_events(
    params: &[(&str, &str)],
) -> Result<PaginatedResponse<CorporateEvent>> {
    let client = build_client()?;
    client.get("/v1/reference/corporate-events", params).await
}
