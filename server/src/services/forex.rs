use std::sync::Arc;

use crate::cache::{self, Cache};
use finance_query::Providers;

use super::{ServiceError, ServiceResult};

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
