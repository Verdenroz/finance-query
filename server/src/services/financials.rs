use crate::cache::{self, Cache};
use finance_query::{Frequency, StatementType, Ticker, Tickers};
use tracing::info;

use super::{ServiceError, ServiceResult};

pub async fn get_financials(
    cache: &Cache,
    symbol: &str,
    statement_type: StatementType,
    statement_str: &str,
    frequency: Frequency,
    frequency_str: &str,
) -> ServiceResult {
    let cache_key = Cache::key(
        "financials",
        &[&symbol.to_uppercase(), statement_str, frequency_str],
    );
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
    let cache_key = Cache::key("financials", &[&symbols_key, statement_str, frequency_str]);

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
