use crate::cache::{self, Cache};
use finance_query::{Interval, Ticker, Tickers, TimeRange};
use tracing::info;

use super::{ServiceError, ServiceResult};

pub async fn get_indicators(
    cache: &Cache,
    symbol: &str,
    interval: Interval,
    interval_str: &str,
    range: TimeRange,
    range_str: &str,
) -> ServiceResult {
    let cache_key = Cache::key(
        "indicators",
        &[&symbol.to_uppercase(), interval_str, range_str],
    );
    let symbol = symbol.to_string();

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::INDICATORS,
            cache::is_market_open(),
            || async move {
                let ticker = Ticker::new(&symbol).await?;
                let indicators = ticker.indicators(interval, range).await?;
                info!("Successfully calculated indicators for {}", symbol);
                serde_json::to_value(&indicators).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

pub async fn get_batch_indicators(
    cache: &Cache,
    symbols: Vec<&str>,
    interval: Interval,
    interval_str: &str,
    range: TimeRange,
    range_str: &str,
) -> ServiceResult {
    let mut symbols = symbols;
    symbols.sort();
    let symbols_key = symbols.join(",").to_uppercase();
    let cache_key = Cache::key("indicators", &[&symbols_key, interval_str, range_str]);

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::INDICATORS,
            cache::is_market_open(),
            || async move {
                let tickers = Tickers::new(symbols).await?;
                let batch_response = tickers.indicators(interval, range).await?;
                info!(
                    "Indicators fetch complete: {} success, {} errors",
                    batch_response.success_count(),
                    batch_response.error_count()
                );
                serde_json::to_value(&batch_response).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}
