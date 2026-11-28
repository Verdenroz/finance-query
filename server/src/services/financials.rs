use crate::cache::{self, Cache};
use finance_query::{Frequency, Providers, StatementType, Ticker, Tickers};
use std::sync::Arc;
use tracing::info;

use super::{ServiceError, ServiceResult};

/// Cached statements written before canonical metric names carry the old keys; bump on any
/// change to the statement key vocabulary so they are not served.
const STATEMENTS_ENDPOINT: &str = "financials-v2";

fn statements_key(subject: &str, statement_str: &str, frequency_str: &str) -> String {
    Cache::key(
        STATEMENTS_ENDPOINT,
        &[subject, statement_str, frequency_str],
    )
}

pub async fn get_financials(
    cache: &Cache,
    symbol: &str,
    statement_type: StatementType,
    statement_str: &str,
    frequency: Frequency,
    frequency_str: &str,
) -> ServiceResult {
    let cache_key = statements_key(&symbol.to_uppercase(), statement_str, frequency_str);
    let symbol = symbol.to_string();

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::FINANCIALS,
            cache::is_market_open(),
            || async move {
                let ticker = Ticker::new(&symbol).await?;
                let result = ticker.financials(statement_type, frequency).await?;
                serde_json::to_value(&result).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

pub async fn get_batch_financials(
    cache: &Cache,
    symbols: Vec<&str>,
    statement_type: StatementType,
    statement_str: &str,
    frequency: Frequency,
    frequency_str: &str,
) -> ServiceResult {
    let mut symbols = symbols;
    symbols.sort();
    let symbols_key = symbols.join(",").to_uppercase();
    let cache_key = statements_key(&symbols_key, statement_str, frequency_str);

    cache
        .get_or_fetch(&cache_key, cache::ttl::FINANCIALS, false, || async move {
            let tickers = Tickers::new(symbols).await?;
            let batch_response = tickers.financials(statement_type, frequency).await?;
            info!(
                "Financials fetch complete: {} success, {} errors",
                batch_response.success_count(),
                batch_response.error_count()
            );
            serde_json::to_value(&batch_response).map_err(|e| Box::new(e) as ServiceError)
        })
        .await
}

pub async fn get_key_metrics_ttm(
    cache: &Cache,
    providers: &Arc<Providers>,
    symbol: &str,
) -> ServiceResult {
    let cache_key = Cache::key("financials", &[&symbol.to_uppercase(), "key-metrics-ttm"]);
    let providers = Arc::clone(providers);
    let symbol = symbol.to_string();
    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::ANALYSIS,
            cache::is_market_open(),
            || async move {
                let ticker = providers.ticker(&symbol).build().await?;
                let data = ticker.key_metrics_ttm().await?;
                serde_json::to_value(data).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

pub async fn get_ratios_ttm(
    cache: &Cache,
    providers: &Arc<Providers>,
    symbol: &str,
) -> ServiceResult {
    let cache_key = Cache::key("financials", &[&symbol.to_uppercase(), "ratios-ttm"]);
    let providers = Arc::clone(providers);
    let symbol = symbol.to_string();
    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::ANALYSIS,
            cache::is_market_open(),
            || async move {
                let ticker = providers.ticker(&symbol).build().await?;
                let data = ticker.ratios_ttm().await?;
                serde_json::to_value(data).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn statement_keys_differ_from_those_cached_before_canonical_names() {
        let old = Cache::key("financials", &["AAPL", "income", "annual"]);
        let new = statements_key("AAPL", "income", "annual");
        assert_ne!(old, new);
        assert!(new.contains("financials-v2"));
    }
}
