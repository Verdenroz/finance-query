use crate::cache::{self, Cache};
use finance_query::{Ticker, Tickers};
use tracing::info;

use super::{ServiceError, ServiceResult};

pub async fn get_options(cache: &Cache, symbol: &str, date: Option<i64>) -> ServiceResult {
    let date_str = date.map(|d| d.to_string()).unwrap_or_default();
    let cache_key = Cache::key("options", &[&symbol.to_uppercase(), &date_str]);
    let symbol = symbol.to_string();

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::OPTIONS,
            cache::is_market_open(),
            || async move {
                let ticker = Ticker::new(&symbol).await?;
                let options_response = ticker.options(date).await?;
                serde_json::to_value(&options_response).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

pub async fn get_batch_options(
    cache: &Cache,
    symbols: Vec<&str>,
    date: Option<i64>,
) -> ServiceResult {
    let mut symbols = symbols;
    symbols.sort();
    let symbols_key = symbols.join(",").to_uppercase();
    let date_str = date
        .map(|d| d.to_string())
        .unwrap_or_else(|| "latest".to_string());
    let cache_key = Cache::key("options", &[&symbols_key, &date_str]);

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::OPTIONS,
            cache::is_market_open(),
            || async move {
                let tickers = Tickers::new(symbols).await?;
                let batch_response = tickers.options(date).await?;
                info!(
                    "Options fetch complete: {} success, {} errors",
                    batch_response.success_count(),
                    batch_response.error_count()
                );
                serde_json::to_value(&batch_response).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}
