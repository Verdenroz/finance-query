use crate::cache::{self, Cache};
use finance_query::{Ticker, finance};
use tracing::info;

use super::{ServiceError, ServiceResult};

pub async fn get_news(cache: &Cache, symbol: &str, count: usize) -> ServiceResult {
    let cache_key = Cache::key("news", &[&symbol.to_uppercase()]);
    let symbol = symbol.to_string();

    let json = cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::NEWS,
            cache::is_market_open(),
            || async move {
                let ticker = Ticker::new(&symbol).await?;
                let news = ticker.news().await?;
                serde_json::to_value(&news).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await?;

    Ok(truncate_array(json, count))
}

pub async fn get_general_news(cache: &Cache, count: usize) -> ServiceResult {
    let cache_key = Cache::key("news", &["general"]);

    let json = cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::GENERAL_NEWS,
            cache::is_market_open(),
            || async move {
                let news = finance::news().await?;
                info!("Fetched general market news");
                serde_json::to_value(&news).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await?;

    Ok(truncate_array(json, count))
}

/// Truncate a JSON array value to at most `count` elements.
/// Non-array values are returned unchanged.
fn truncate_array(mut value: serde_json::Value, count: usize) -> serde_json::Value {
    if let serde_json::Value::Array(ref mut arr) = value {
        arr.truncate(count);
    }
    value
}
