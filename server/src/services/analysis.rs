use std::sync::Arc;

use crate::cache::{self, Cache};
use finance_query::{Providers, Ticker};

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

pub async fn get_company_profile(
    cache: &Cache,
    providers: &Arc<Providers>,
    symbol: &str,
) -> ServiceResult {
    let cache_key = Cache::key("analysis", &[&symbol.to_uppercase(), "company-profile"]);
    let providers = Arc::clone(providers);
    let symbol = symbol.to_string();
    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::ANALYSIS,
            cache::is_market_open(),
            || async move {
                let ticker = providers.ticker(&symbol).build().await?;
                let data = ticker.company_profile().await?;
                serde_json::to_value(data).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

pub async fn get_earnings_surprises(
    cache: &Cache,
    providers: &Arc<Providers>,
    symbol: &str,
) -> ServiceResult {
    let cache_key = Cache::key("analysis", &[&symbol.to_uppercase(), "earnings-surprises"]);
    let providers = Arc::clone(providers);
    let symbol = symbol.to_string();
    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::ANALYSIS,
            cache::is_market_open(),
            || async move {
                let ticker = providers.ticker(&symbol).build().await?;
                let surprises = ticker.earnings_surprises().await?;
                serde_json::to_value(serde_json::json!({ "surprises": surprises }))
                    .map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

pub async fn get_rating_consensus(
    cache: &Cache,
    providers: &Arc<Providers>,
    symbol: &str,
) -> ServiceResult {
    let cache_key = Cache::key("analysis", &[&symbol.to_uppercase(), "rating-consensus"]);
    let providers = Arc::clone(providers);
    let symbol = symbol.to_string();
    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::ANALYSIS,
            cache::is_market_open(),
            || async move {
                let ticker = providers.ticker(&symbol).build().await?;
                let data = ticker.rating_consensus().await?;
                serde_json::to_value(data).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

pub async fn get_price_target_consensus(
    cache: &Cache,
    providers: &Arc<Providers>,
    symbol: &str,
) -> ServiceResult {
    let cache_key = Cache::key(
        "analysis",
        &[&symbol.to_uppercase(), "price-target-consensus"],
    );
    let providers = Arc::clone(providers);
    let symbol = symbol.to_string();
    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::ANALYSIS,
            cache::is_market_open(),
            || async move {
                let ticker = providers.ticker(&symbol).build().await?;
                let data = ticker.price_target_consensus().await?;
                serde_json::to_value(data).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

pub async fn get_etf_profile(
    cache: &Cache,
    providers: &Arc<Providers>,
    symbol: &str,
) -> ServiceResult {
    let cache_key = Cache::key("analysis", &[&symbol.to_uppercase(), "etf-profile"]);
    let providers = Arc::clone(providers);
    let symbol = symbol.to_string();
    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::ANALYSIS,
            cache::is_market_open(),
            || async move {
                let ticker = providers.ticker(&symbol).build().await?;
                let data = ticker.etf_profile().await?;
                serde_json::to_value(data).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

pub async fn get_earnings_transcript(
    cache: &Cache,
    providers: &Arc<Providers>,
    symbol: &str,
    quarter: Option<&str>,
    year: Option<i32>,
) -> ServiceResult {
    let quarter_key = quarter.unwrap_or("latest").to_string();
    let year_key = year
        .map(|y| y.to_string())
        .unwrap_or_else(|| "latest".to_string());
    let cache_key = Cache::key(
        "analysis",
        &[
            &symbol.to_uppercase(),
            "earnings-transcript",
            &quarter_key,
            &year_key,
        ],
    );
    let providers = Arc::clone(providers);
    let symbol = symbol.to_string();
    let quarter = quarter.map(str::to_string);
    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::ANALYSIS,
            cache::is_market_open(),
            || async move {
                let ticker = providers.ticker(&symbol).build().await?;
                let data = ticker.earnings_transcript(quarter.as_deref(), year).await?;
                serde_json::to_value(data).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

pub async fn get_grading_actions(
    cache: &Cache,
    providers: &Arc<Providers>,
    symbol: &str,
) -> ServiceResult {
    let cache_key = Cache::key("analysis", &[&symbol.to_uppercase(), "grading-actions"]);
    let providers = Arc::clone(providers);
    let symbol = symbol.to_string();
    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::ANALYSIS,
            cache::is_market_open(),
            || async move {
                let ticker = providers.ticker(&symbol).build().await?;
                let data = ticker.grading_actions().await?;
                serde_json::to_value(data).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

pub async fn get_price_target_summary(
    cache: &Cache,
    providers: &Arc<Providers>,
    symbol: &str,
) -> ServiceResult {
    let cache_key = Cache::key(
        "analysis",
        &[&symbol.to_uppercase(), "price-target-summary"],
    );
    let providers = Arc::clone(providers);
    let symbol = symbol.to_string();
    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::ANALYSIS,
            cache::is_market_open(),
            || async move {
                let ticker = providers.ticker(&symbol).build().await?;
                let data = ticker.price_target_summary().await?;
                serde_json::to_value(data).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}
