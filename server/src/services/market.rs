use crate::cache::{self, Cache};
use finance_query::{IndicesRegion, Region, Screener, Tickers, finance};
use tracing::info;

use super::{ServiceError, ServiceResult, lang_key};

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
                // Yahoo's trending endpoint only returns bare symbols — enrich
                // each with a batch quote lookup so name/price/change aren't null.
                let symbols: Vec<&str> = trending.iter().map(|t| t.symbol.as_str()).collect();
                let quotes = if symbols.is_empty() {
                    std::collections::HashMap::new()
                } else {
                    Tickers::builder(symbols).build().await?.quotes().await?.quotes
                };
                let enriched: Vec<serde_json::Value> = trending
                    .into_iter()
                    .map(|t| {
                        let quote_json = quotes
                            .get(&t.symbol)
                            .and_then(|q| serde_json::to_value(q).ok());
                        serde_json::json!({
                            "symbol": t.symbol,
                            "shortName": quote_json.as_ref().and_then(|v| v.get("shortName")).cloned(),
                            "regularMarketPrice": quote_json.as_ref().and_then(|v| v.get("regularMarketPrice")).cloned(),
                            "regularMarketChangePercent": quote_json.as_ref().and_then(|v| v.get("regularMarketChangePercent")).cloned(),
                        })
                    })
                    .collect();
                serde_json::to_value(enriched).map_err(|e| Box::new(e) as ServiceError)
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

pub async fn get_market_summary(
    cache: &Cache,
    region: Option<Region>,
    lang: Option<&str>,
) -> ServiceResult {
    let region_str = region
        .map(|r| format!("{:?}", r))
        .unwrap_or_else(|| "US".to_string());
    let cache_key = Cache::key("market_summary", &[&region_str, lang_key(lang)]);
    let lang = lang.map(str::to_string);

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::INDICES,
            cache::is_market_open(),
            || async move {
                let mut summary = finance::market_summary(region).await?;
                super::translate(&mut summary, lang.as_deref()).await?;
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
    lang: Option<&str>,
) -> ServiceResult {
    let cache_key = Cache::key("sector", &[sector_str, lang_key(lang)]);
    let lang = lang.map(str::to_string);

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::SECTORS,
            cache::is_market_open(),
            || async move {
                let mut data = finance::sector(sector).await?;
                super::translate(&mut data, lang.as_deref()).await?;
                serde_json::to_value(data).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

pub async fn get_industry(cache: &Cache, industry: &str, lang: Option<&str>) -> ServiceResult {
    let cache_key = Cache::key("industry", &[industry, lang_key(lang)]);
    let industry = industry.to_string();
    let lang = lang.map(str::to_string);

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::SECTORS,
            cache::is_market_open(),
            || async move {
                let mut data = finance::industry(&industry).await?;
                super::translate(&mut data, lang.as_deref()).await?;
                serde_json::to_value(data).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}
