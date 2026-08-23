use std::sync::Arc;

use crate::cache::{self, Cache};
use finance_query::Providers;

use super::{ServiceError, ServiceResult};

/// Congressional trading disclosures for a symbol, via `Capability::FILINGS`
/// (FMP, falling back to keyless House disclosures — Senate is unrouted).
pub async fn get_congressional_trades(
    cache: &Cache,
    providers: &Arc<Providers>,
    symbol: &str,
) -> ServiceResult {
    let cache_key = Cache::key("filings", &[&symbol.to_uppercase(), "congressional-trades"]);
    let providers = Arc::clone(providers);
    let symbol = symbol.to_string();

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::ANALYSIS,
            cache::is_market_open(),
            || async move {
                let trades = providers.filings(&symbol).congressional_trades().await?;
                serde_json::to_value(&trades).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

/// Fails-to-deliver records for a symbol, via `Capability::FILINGS`
/// (currently FMP only).
pub async fn get_fails_to_deliver(
    cache: &Cache,
    providers: &Arc<Providers>,
    symbol: &str,
) -> ServiceResult {
    let cache_key = Cache::key("filings", &[&symbol.to_uppercase(), "fails-to-deliver"]);
    let providers = Arc::clone(providers);
    let symbol = symbol.to_string();

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::ANALYSIS,
            cache::is_market_open(),
            || async move {
                let records = providers.filings(&symbol).fails_to_deliver().await?;
                serde_json::to_value(&records).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}
