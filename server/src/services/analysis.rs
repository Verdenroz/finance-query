use crate::cache::{self, Cache};
use finance_query::Ticker;

use super::{ServiceError, ServiceResult};

/// Each analysis type gets its own straight-line async fn (rather than one fn
/// with a match over all 4 awaiting different `Ticker` accessors) — a single
/// multi-branch async closure must size its generator state for the union of
/// every branch's locals/await-points at once, which under this workspace's
/// `opt-level=0` (un-inlined) dev profile risks overflowing the worker stack.
/// See `services::holders` (fixed for the same reason) and
/// `services::quote::get_quote` for the one-path shape.
pub async fn get_recommendation_trend(cache: &Cache, symbol: &str) -> ServiceResult {
    let cache_key = Cache::key("analysis", &[&symbol.to_uppercase(), "recommendations"]);
    let symbol = symbol.to_string();
    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::ANALYSIS,
            cache::is_market_open(),
            || async move {
                let ticker = Ticker::new(&symbol).await?;
                let data = ticker.recommendation_trend().await?;
                serde_json::to_value(data).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

pub async fn get_grading_history(cache: &Cache, symbol: &str) -> ServiceResult {
    let cache_key = Cache::key("analysis", &[&symbol.to_uppercase(), "upgrades-downgrades"]);
    let symbol = symbol.to_string();
    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::ANALYSIS,
            cache::is_market_open(),
            || async move {
                let ticker = Ticker::new(&symbol).await?;
                let data = ticker.grading_history().await?;
                serde_json::to_value(data).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

pub async fn get_earnings_trend(cache: &Cache, symbol: &str) -> ServiceResult {
    let cache_key = Cache::key("analysis", &[&symbol.to_uppercase(), "earnings-estimate"]);
    let symbol = symbol.to_string();
    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::ANALYSIS,
            cache::is_market_open(),
            || async move {
                let ticker = Ticker::new(&symbol).await?;
                let data = ticker.earnings_trend().await?;
                serde_json::to_value(data).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

pub async fn get_earnings_history(cache: &Cache, symbol: &str) -> ServiceResult {
    let cache_key = Cache::key("analysis", &[&symbol.to_uppercase(), "earnings-history"]);
    let symbol = symbol.to_string();
    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::ANALYSIS,
            cache::is_market_open(),
            || async move {
                let ticker = Ticker::new(&symbol).await?;
                let data = ticker.earnings_history().await?;
                serde_json::to_value(data).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

pub async fn get_recommendations(cache: &Cache, symbol: &str, limit: u32) -> ServiceResult {
    let cache_key = Cache::key(
        "recommendations",
        &[&symbol.to_uppercase(), &limit.to_string()],
    );
    let symbol = symbol.to_string();

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::ANALYSIS,
            cache::is_market_open(),
            || async move {
                let ticker = Ticker::new(&symbol).await?;
                let recommendation = ticker.recommendations(limit).await?;
                serde_json::to_value(&recommendation).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

pub async fn get_batch_recommendations(
    cache: &Cache,
    symbols: Vec<&str>,
    limit: u32,
) -> ServiceResult {
    let mut symbols = symbols;
    symbols.sort();
    let symbols_key = symbols.join(",").to_uppercase();
    let cache_key = Cache::key("recommendations", &[&symbols_key, &limit.to_string()]);

    cache
        .get_or_fetch(&cache_key, cache::ttl::ANALYSIS, false, || async move {
            let tickers = finance_query::Tickers::new(symbols).await?;
            let batch_response = tickers.recommendations(limit).await?;
            tracing::info!(
                "Recommendations fetch complete: {} success, {} errors",
                batch_response.success_count(),
                batch_response.error_count()
            );
            serde_json::to_value(&batch_response).map_err(|e| Box::new(e) as ServiceError)
        })
        .await
}
