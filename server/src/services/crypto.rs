use std::sync::Arc;

use crate::cache::{self, Cache};
use finance_query::Providers;

use super::{ServiceError, ServiceResult};

pub async fn get_coins(cache: &Cache, vs_currency: &str, count: usize) -> ServiceResult {
    let cache_key = Cache::key("crypto_coins", &[vs_currency, &count.to_string()]);
    let vs = vs_currency.to_string();

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::QUOTES,
            cache::is_market_open(),
            || async move {
                let coins = finance_query::crypto::coins(&vs, count).await?;
                serde_json::to_value(&coins).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

pub async fn get_trending(cache: &Cache) -> ServiceResult {
    cache
        .get_or_fetch(
            &Cache::key("crypto_trending", &[]),
            cache::ttl::QUOTES,
            cache::is_market_open(),
            || async move {
                let coins = finance_query::crypto::trending().await?;
                serde_json::to_value(&coins).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

pub async fn get_global(cache: &Cache) -> ServiceResult {
    cache
        .get_or_fetch(
            &Cache::key("crypto_global", &[]),
            cache::ttl::QUOTES,
            cache::is_market_open(),
            || async move {
                let stats = finance_query::crypto::global().await?;
                serde_json::to_value(&stats).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

pub async fn search(cache: &Cache, query: &str, limit: u32) -> ServiceResult {
    let cache_key = Cache::key("crypto_search", &[query, &limit.to_string()]);
    let q = query.to_string();

    cache
        .get_or_fetch(&cache_key, cache::ttl::QUOTES, false, || async move {
            let matches = finance_query::crypto::search(&q, limit).await?;
            serde_json::to_value(&matches).map_err(|e| Box::new(e) as ServiceError)
        })
        .await
}

/// Market-wide crypto news via `Capability::CRYPTO` (currently FMP only).
pub async fn get_news(cache: &Cache, providers: &Arc<Providers>, limit: u32) -> ServiceResult {
    let cache_key = Cache::key("crypto_news", &[&limit.to_string()]);
    let providers = Arc::clone(providers);

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::NEWS,
            cache::is_market_open(),
            || async move {
                // The id is unused: this call routes on Capability::CRYPTO alone.
                let news = providers.crypto("").news(limit).await?;
                serde_json::to_value(&news).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

pub async fn get_coin(cache: &Cache, coin_id: &str, vs_currency: &str) -> ServiceResult {
    let cache_key = Cache::key("crypto_coin", &[coin_id, vs_currency]);
    let id = coin_id.to_string();
    let vs = vs_currency.to_string();

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::QUOTES,
            cache::is_market_open(),
            || async move {
                let coin = finance_query::crypto::coin(&id, &vs).await?;
                serde_json::to_value(&coin).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

/// A protocol's current TVL, provider-routed (DefiLlama, keyless).
pub async fn get_protocol_tvl(
    cache: &Cache,
    providers: &Arc<Providers>,
    id: &str,
) -> ServiceResult {
    let cache_key = Cache::key("crypto_tvl", &[id]);
    let providers = Arc::clone(providers);
    let id = id.to_string();
    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::QUOTES,
            cache::is_market_open(),
            || async move {
                let tvl = providers.crypto(&id).tvl().await?;
                serde_json::to_value(&tvl).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

/// A protocol's TVL history, oldest first.
pub async fn get_protocol_tvl_history(
    cache: &Cache,
    providers: &Arc<Providers>,
    id: &str,
) -> ServiceResult {
    let cache_key = Cache::key("crypto_tvl_history", &[id]);
    let providers = Arc::clone(providers);
    let id = id.to_string();
    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::HISTORICAL,
            cache::is_market_open(),
            || async move {
                let points = providers.crypto(&id).tvl_history().await?;
                serde_json::to_value(&points).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}
