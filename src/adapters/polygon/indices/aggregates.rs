//! Index aggregate bar endpoints: previous close, daily open/close.

use crate::adapters::common::encode_path_segment;
use crate::error::Result;

use super::super::build_client;
use super::super::models::*;

/// Fetch the previous day's OHLCV bar for an index ticker.
///
/// * `ticker` - Index ticker symbol with `I:` prefix (e.g., `"I:SPX"`)
#[allow(dead_code)] // unrouted: no capability route for a standalone previous-close lookup yet
pub async fn index_previous_close(ticker: &str) -> Result<AggregateResponseDTO> {
    let client = build_client()?;
    let path = format!("/v2/aggs/ticker/{}/prev", encode_path_segment(ticker));

    client
        .get_as(
            &path,
            &[],
            "index_previous_close",
            "index previous close response",
        )
        .await
}

/// Fetch daily open/close for an index ticker on a specific date.
///
/// * `ticker` - Index ticker symbol with `I:` prefix (e.g., `"I:SPX"`)
/// * `date` - Date as `"YYYY-MM-DD"`
#[allow(dead_code)] // unrouted: no capability route for a standalone daily-open-close lookup yet
pub async fn index_daily_open_close(ticker: &str, date: &str) -> Result<DailyOpenCloseDTO> {
    let client = build_client()?;
    let path = format!(
        "/v1/open-close/{}/{}",
        encode_path_segment(ticker),
        encode_path_segment(date)
    );

    client
        .get_as(
            &path,
            &[],
            "index_daily_open_close",
            "index daily open/close response",
        )
        .await
}
