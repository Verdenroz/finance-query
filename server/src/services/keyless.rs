//! Keyless provider services: GDELT global news search and CFTC Commitments
//! of Traders. Both call the library's keyless shortcut modules directly —
//! neither needs an API key or provider routing.

use crate::cache::{self, Cache};

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
