use crate::cache::{self, Cache};
use finance_query::{Ticker, finance};

use super::{ServiceError, ServiceResult};

pub async fn get_currencies(cache: &Cache) -> ServiceResult {
    let cache_key = Cache::key("currencies", &[]);

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::METADATA,
            cache::is_market_open(),
            || async {
                let currencies = finance::currencies().await?;
                serde_json::to_value(currencies).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

pub async fn get_exchanges(cache: &Cache) -> ServiceResult {
    let cache_key = Cache::key("exchanges", &[]);

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::METADATA,
            cache::is_market_open(),
            || async {
                let exchanges = finance::exchanges().await?;
                serde_json::to_value(&exchanges).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

pub async fn get_hours(cache: &Cache, region: Option<&str>) -> ServiceResult {
    let region_display = region.unwrap_or("US");
    let cache_key = Cache::key("hours", &[region_display]);
    let region = region.map(|s| s.to_string());

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::MARKET_HOURS,
            cache::is_market_open(),
            || async move {
                let response = finance::hours(region.as_deref()).await?;
                serde_json::to_value(&response).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

pub async fn get_quote_type(cache: &Cache, symbol: &str) -> ServiceResult {
    let cache_key = Cache::key("quote-type", &[&symbol.to_uppercase()]);
    let symbol = symbol.to_string();

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::METADATA,
            cache::is_market_open(),
            || async move {
                let ticker = Ticker::new(&symbol).await?;
                let response = ticker.quote_type().await?;
                serde_json::to_value(&response).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}
