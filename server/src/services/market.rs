use crate::cache::{self, Cache};
use finance_query::{IndicesRegion, Region, Screener, finance};
use tracing::info;

use super::{ServiceError, ServiceResult};

pub async fn get_trending(cache: &Cache, region: Option<Region>) -> ServiceResult {
    let region_str = region
        .map(|r| format!("{:?}", r))
        .unwrap_or_else(|| "US".to_string());
    let cache_key = Cache::key("trending", &[&region_str]);

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::MOVERS,
            cache::is_market_open(),
            || async move {
                let trending = finance::trending(region).await?;
                serde_json::to_value(trending).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

pub async fn get_indices(cache: &Cache, region: Option<IndicesRegion>) -> ServiceResult {
    let region_str = region.map(|r| r.as_str()).unwrap_or("all");
    let cache_key = Cache::key("indices", &[region_str]);

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::INDICES,
            cache::is_market_open(),
            || async move {
                let batch_response = finance::indices(region).await?;
                info!(
                    "Indices fetch complete: {} success, {} errors",
                    batch_response.success_count(),
                    batch_response.error_count()
                );
                serde_json::to_value(&batch_response).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

pub async fn get_fear_and_greed(cache: &Cache) -> ServiceResult {
    let cache_key = Cache::key("fear_and_greed", &[]);

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::GENERAL_NEWS,
            cache::is_market_open(),
            || async {
                let fng = finance::fear_and_greed().await?;
                serde_json::to_value(&fng).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

pub async fn get_market_summary(cache: &Cache, region: Option<Region>) -> ServiceResult {
    let region_str = region
        .map(|r| format!("{:?}", r))
        .unwrap_or_else(|| "US".to_string());
    let cache_key = Cache::key("market_summary", &[&region_str]);

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::INDICES,
            cache::is_market_open(),
            || async move {
                let summary = finance::market_summary(region).await?;
                serde_json::to_value(summary).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

pub async fn get_screener(
    cache: &Cache,
    screener: Screener,
    screener_str: &str,
    count: u32,
) -> ServiceResult {
    let cache_key = Cache::key("screener", &[screener_str, &count.to_string()]);

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::MOVERS,
            cache::is_market_open(),
            || async move {
                let data = finance::screener(screener, count).await?;
                serde_json::to_value(data).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

pub async fn get_sector(
    cache: &Cache,
    sector: finance_query::Sector,
    sector_str: &str,
) -> ServiceResult {
    let cache_key = Cache::key("sector", &[sector_str]);

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::SECTORS,
            cache::is_market_open(),
            || async move {
                let data = finance::sector(sector).await?;
                serde_json::to_value(data).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

pub async fn get_industry(cache: &Cache, industry: &str) -> ServiceResult {
    let cache_key = Cache::key("industry", &[industry]);
    let industry = industry.to_string();

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::SECTORS,
            cache::is_market_open(),
            || async move {
                let data = finance::industry(&industry).await?;
                serde_json::to_value(data).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}
