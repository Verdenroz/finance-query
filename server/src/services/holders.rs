use crate::cache::{self, Cache};
use finance_query::Ticker;

use super::{ServiceError, ServiceResult};

/// Each holder type gets its own straight-line async fn (rather than one fn
/// with a match over all 6 awaiting different `Ticker` accessors) — a single
/// multi-branch async closure must size its generator state for the union of
/// every branch's locals/await-points at once, which under this workspace's
/// `opt-level=0` (un-inlined) dev profile was deep enough to overflow the
/// worker stack. See `services::quote::get_quote` for the same one-path shape.
pub async fn get_major_holders(cache: &Cache, symbol: &str) -> ServiceResult {
    let cache_key = Cache::key("holders", &[&symbol.to_uppercase(), "major"]);
    let symbol = symbol.to_string();
    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::HOLDERS,
            cache::is_market_open(),
            || async move {
                let ticker = Ticker::new(&symbol).await?;
                let data = ticker.major_holders().await?;
                serde_json::to_value(data).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

pub async fn get_institutional_holders(cache: &Cache, symbol: &str) -> ServiceResult {
    let cache_key = Cache::key("holders", &[&symbol.to_uppercase(), "institutional"]);
    let symbol = symbol.to_string();
    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::HOLDERS,
            cache::is_market_open(),
            || async move {
                let ticker = Ticker::new(&symbol).await?;
                let data = ticker.institution_ownership().await?;
                serde_json::to_value(data).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

pub async fn get_mutual_fund_holders(cache: &Cache, symbol: &str) -> ServiceResult {
    let cache_key = Cache::key("holders", &[&symbol.to_uppercase(), "mutualfund"]);
    let symbol = symbol.to_string();
    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::HOLDERS,
            cache::is_market_open(),
            || async move {
                let ticker = Ticker::new(&symbol).await?;
                let data = ticker.fund_ownership().await?;
                serde_json::to_value(data).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

pub async fn get_insider_transactions(cache: &Cache, symbol: &str) -> ServiceResult {
    let cache_key = Cache::key("holders", &[&symbol.to_uppercase(), "insider-transactions"]);
    let symbol = symbol.to_string();
    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::HOLDERS,
            cache::is_market_open(),
            || async move {
                let ticker = Ticker::new(&symbol).await?;
                let data = ticker.insider_transactions().await?;
                serde_json::to_value(data).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

pub async fn get_insider_purchases(cache: &Cache, symbol: &str) -> ServiceResult {
    let cache_key = Cache::key("holders", &[&symbol.to_uppercase(), "insider-purchases"]);
    let symbol = symbol.to_string();
    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::HOLDERS,
            cache::is_market_open(),
            || async move {
                let ticker = Ticker::new(&symbol).await?;
                let data = ticker.share_purchase_activity().await?;
                serde_json::to_value(data).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

pub async fn get_insider_roster(cache: &Cache, symbol: &str) -> ServiceResult {
    let cache_key = Cache::key("holders", &[&symbol.to_uppercase(), "insider-roster"]);
    let symbol = symbol.to_string();
    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::HOLDERS,
            cache::is_market_open(),
            || async move {
                let ticker = Ticker::new(&symbol).await?;
                let data = ticker.insider_holders().await?;
                serde_json::to_value(data).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}
