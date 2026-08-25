use std::sync::Arc;

use crate::cache::{self, Cache};
use finance_query::Providers;

use super::{ServiceError, ServiceResult};

/// A currency pair's current exchange rate, via `Capability::FOREX`.
pub async fn get_quote(
    cache: &Cache,
    providers: &Arc<Providers>,
    from: &str,
    to: &str,
) -> ServiceResult {
    let cache_key = Cache::key("forex-quote", &[&from.to_uppercase(), &to.to_uppercase()]);
    let providers = Arc::clone(providers);
    let from = from.to_string();
    let to = to.to_string();

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::MOVERS,
            cache::is_market_open(),
            || async move {
                let quote = providers.forex(&from, &to).quote().await?;
                serde_json::to_value(quote).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

/// Market-wide forex news via `Capability::FOREX` (currently FMP only).
pub async fn get_news(cache: &Cache, providers: &Arc<Providers>, limit: u32) -> ServiceResult {
    let cache_key = Cache::key("forex_news", &[&limit.to_string()]);
    let providers = Arc::clone(providers);

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::NEWS,
            cache::is_market_open(),
            || async move {
                // The pair is unused: this call routes on Capability::FOREX alone.
                let news = providers.forex("", "").news(limit).await?;
                serde_json::to_value(&news).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}
