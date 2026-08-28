use std::sync::Arc;

use crate::cache::{self, Cache};
use finance_query::{FilingSectionForm, Providers};

use super::{ServiceError, ServiceResult};

/// Congressional trading disclosures for a symbol, via `Capability::FILINGS`
/// (FMP, falling back to keyless House PTR disclosures).
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

/// Sectioned text of one filing, via `Capability::FILINGS` (EDGAR,
/// best-effort HTML extraction; Polygon when configured). Not cached — keyed
/// by accession number, which is already effectively unique per call.
pub async fn get_filing_sections(
    providers: &Arc<Providers>,
    symbol: &str,
    accession_number: &str,
    form: FilingSectionForm,
) -> ServiceResult {
    let sections = providers
        .filings(symbol)
        .sections(accession_number, form)
        .await?;
    serde_json::to_value(&sections).map_err(|e| Box::new(e) as ServiceError)
}

/// Risk factors extracted from a symbol's SEC filings, via
/// `Capability::FILINGS` (EDGAR, best-effort HTML extraction; Polygon when
/// configured).
pub async fn get_risk_factors(
    cache: &Cache,
    providers: &Arc<Providers>,
    symbol: &str,
) -> ServiceResult {
    let cache_key = Cache::key("filings", &[&symbol.to_uppercase(), "risk-factors"]);
    let providers = Arc::clone(providers);
    let symbol = symbol.to_string();

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::ANALYSIS,
            cache::is_market_open(),
            || async move {
                let factors = providers.filings(&symbol).risk_factors().await?;
                serde_json::to_value(&factors).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}
