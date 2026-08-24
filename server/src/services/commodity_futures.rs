use std::sync::Arc;

use crate::cache::{self, Cache};
use finance_query::Providers;

use super::{ServiceError, ServiceResult};

/// A commodity's current quote, via `Capability::COMMODITIES` (Yahoo, keyless).
pub async fn get_commodity_quote(
    cache: &Cache,
    providers: &Arc<Providers>,
    symbol: &str,
) -> ServiceResult {
    let cache_key = Cache::key("commodity-quote", &[&symbol.to_uppercase()]);
    let providers = Arc::clone(providers);
    let symbol = symbol.to_string();

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::MOVERS,
            cache::is_market_open(),
            || async move {
                let quote = providers.commodity(&symbol).quote().await?;
                serde_json::to_value(quote).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

/// A futures contract's current quote, via `Capability::FUTURES` (Yahoo, keyless).
pub async fn get_futures_quote(
    cache: &Cache,
    providers: &Arc<Providers>,
    symbol: &str,
) -> ServiceResult {
    let cache_key = Cache::key("futures-quote", &[&symbol.to_uppercase()]);
    let providers = Arc::clone(providers);
    let symbol = symbol.to_string();

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::MOVERS,
            cache::is_market_open(),
            || async move {
                let quote = providers.futures(&symbol).quote().await?;
                serde_json::to_value(quote).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

/// An index's current constituent list, via `Capability::INDICES`
/// (Wikipedia, S&P 500 only).
pub async fn get_index_constituents(
    cache: &Cache,
    providers: &Arc<Providers>,
    symbol: &str,
) -> ServiceResult {
    let cache_key = Cache::key("index-constituents", &[&symbol.to_uppercase()]);
    let providers = Arc::clone(providers);
    let symbol = symbol.to_string();

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::SECTORS,
            cache::is_market_open(),
            || async move {
                let constituents = providers.index(&symbol).constituents().await?;
                serde_json::to_value(constituents).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}
