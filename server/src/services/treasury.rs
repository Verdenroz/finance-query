//! US Treasury securities auction services, provider-routed through
//! `Capability::ECONOMIC` (FiscalData is the only provider that serves them).

use std::sync::Arc;

use crate::cache::{self, Cache};
use finance_query::{Providers, TreasuryAuctionQuery};

use super::{ServiceError, ServiceResult};

/// Auctions matching `query`, most recent auction date first.
pub async fn get_treasury_auctions(
    cache: &Cache,
    providers: &Arc<Providers>,
    query: TreasuryAuctionQuery,
) -> ServiceResult {
    let cache_key = Cache::key(
        "treasury_auctions",
        &[
            query.security_type.as_deref().unwrap_or(""),
            query.security_term.as_deref().unwrap_or(""),
            query.from.as_deref().unwrap_or(""),
            query.to.as_deref().unwrap_or(""),
            &query.limit.map(|l| l.to_string()).unwrap_or_default(),
        ],
    );
    let providers = Arc::clone(providers);
    cache
        .get_or_fetch(&cache_key, cache::ttl::ANALYSIS, false, || async move {
            let auctions = providers
                .economic_catalog()
                .treasury_auctions(&query)
                .await?;
            serde_json::to_value(&auctions).map_err(|e| Box::new(e) as ServiceError)
        })
        .await
}

/// Auctions Treasury has scheduled but not yet held, soonest first.
pub async fn get_upcoming_auctions(cache: &Cache, providers: &Arc<Providers>) -> ServiceResult {
    let cache_key = Cache::key("upcoming_auctions", &[]);
    let providers = Arc::clone(providers);
    cache
        .get_or_fetch(&cache_key, cache::ttl::ANALYSIS, false, || async move {
            let upcoming = providers.economic_catalog().upcoming_auctions().await?;
            serde_json::to_value(&upcoming).map_err(|e| Box::new(e) as ServiceError)
        })
        .await
}
