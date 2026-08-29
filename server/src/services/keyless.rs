//! Keyless provider services: GDELT global news search and CFTC Commitments
//! of Traders. Both call the library's keyless shortcut modules directly —
//! neither needs an API key or provider routing.

use std::sync::Arc;

use crate::cache::{self, Cache};
use finance_query::Providers;

use super::{ServiceError, ServiceResult};

/// Worldwide news mentioning `symbol`, from GDELT DOC 2.0.
///
/// GDELT self-paces to ~1 request per 5 seconds process-wide, so this is
/// cached aggressively — an uncached burst would serialise behind that
/// limiter.
pub async fn get_gdelt_news(cache: &Cache, symbol: &str) -> ServiceResult {
    let cache_key = Cache::key("gdelt_news", &[symbol]);
    let sym = symbol.to_string();

    cache
        .get_or_fetch(&cache_key, cache::ttl::GENERAL_NEWS, false, || async move {
            let news = finance_query::gdelt::news(&sym).await?;
            serde_json::to_value(&news).map_err(|e| Box::new(e) as ServiceError)
        })
        .await
}

/// Weekly CFTC Commitments of Traders positioning for a futures contract.
///
/// Reports publish weekly, so the response is cached for a long interval.
pub async fn get_commitments_of_traders(cache: &Cache, symbol: &str) -> ServiceResult {
    let cache_key = Cache::key("cftc_cot", &[symbol]);
    let sym = symbol.to_string();

    cache
        .get_or_fetch(&cache_key, cache::ttl::FINANCIALS, false, || async move {
            let cot = finance_query::cftc::commitments_of_traders(&sym).await?;
            serde_json::to_value(&cot).map_err(|e| Box::new(e) as ServiceError)
        })
        .await
}

/// Reference detail for one symbol, provider-routed (Capability::DISCOVERY).
pub async fn get_symbol_details(
    cache: &Cache,
    providers: &Arc<Providers>,
    symbol: &str,
) -> ServiceResult {
    let cache_key = Cache::key("symbol_details", &[&symbol.to_uppercase()]);
    let providers = Arc::clone(providers);
    let symbol = symbol.to_string();
    cache
        .get_or_fetch(&cache_key, cache::ttl::METADATA, false, || async move {
            let details = providers.discovery().details(&symbol).await?;
            serde_json::to_value(&details).map_err(|e| Box::new(e) as ServiceError)
        })
        .await
}

/// Listed or delisted symbols, provider-routed (Capability::DISCOVERY).
pub async fn get_listing_status(
    cache: &Cache,
    providers: &Arc<Providers>,
    active: bool,
) -> ServiceResult {
    let cache_key = Cache::key("listing_status", &[&active.to_string()]);
    let providers = Arc::clone(providers);
    cache
        .get_or_fetch(&cache_key, cache::ttl::METADATA, false, || async move {
            let symbols = providers.discovery().listing_status(active).await?;
            serde_json::to_value(&symbols).map_err(|e| Box::new(e) as ServiceError)
        })
        .await
}
