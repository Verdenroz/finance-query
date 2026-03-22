//! Alternative.me Fear & Greed Index endpoint.
//!
//! No authentication required. Returns the current market sentiment index.

use std::time::Duration;

use crate::error::{FinanceError, Result};
use crate::models::sentiment::response::{FearAndGreed, FearAndGreedApiResponse};

const API_URL: &str = "https://api.alternative.me/fng/?limit=1&format=json";

/// Fetch the current Fear & Greed Index from Alternative.me.
pub(crate) async fn fetch() -> Result<FearAndGreed> {
    let response = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?
        .get(API_URL)
        .send()
        .await?;

    let status = response.status().as_u16();
    if !response.status().is_success() {
        return Err(FinanceError::ExternalApiError {
            api: "alternative.me".to_string(),
            status,
        });
    }

    let raw: FearAndGreedApiResponse = response.json().await?;

    FearAndGreed::from_response(raw)
}
