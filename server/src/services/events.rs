use crate::cache::{self, Cache};
use finance_query::{Ticker, Tickers, TimeRange};
use tracing::info;

use super::{ServiceError, ServiceResult};

pub async fn get_dividends(
    cache: &Cache,
    symbol: &str,
    range: TimeRange,
    range_str: &str,
) -> ServiceResult {
    let cache_key = Cache::key("dividends", &[&symbol.to_uppercase(), range_str]);
    let symbol = symbol.to_string();

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::HISTORICAL,
            cache::is_market_open(),
            || async move {
                let ticker = Ticker::new(&symbol).await?;
                let dividends = ticker.dividends(range).await?;
                let analytics = ticker.dividend_analytics(range).await?;
                let json = serde_json::json!({
                    "dividends": dividends,
                    "analytics": analytics,
                });
                serde_json::to_value(json).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

pub async fn get_splits(
    cache: &Cache,
    symbol: &str,
    range: TimeRange,
    range_str: &str,
) -> ServiceResult {
    let cache_key = Cache::key("splits", &[&symbol.to_uppercase(), range_str]);
    let symbol = symbol.to_string();

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::HISTORICAL,
            cache::is_market_open(),
            || async move {
                let ticker = Ticker::new(&symbol).await?;
                let splits = ticker.splits(range).await?;
                serde_json::to_value(&splits).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

pub async fn get_capital_gains(
    cache: &Cache,
    symbol: &str,
    range: TimeRange,
    range_str: &str,
) -> ServiceResult {
    let cache_key = Cache::key("capital-gains", &[&symbol.to_uppercase(), range_str]);
    let symbol = symbol.to_string();

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::HISTORICAL,
            cache::is_market_open(),
            || async move {
                let ticker = Ticker::new(&symbol).await?;
                let gains = ticker.capital_gains(range).await?;
                serde_json::to_value(&gains).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

pub async fn get_batch_dividends(
    cache: &Cache,
    symbols: Vec<&str>,
    range: TimeRange,
    range_str: &str,
) -> ServiceResult {
    let mut symbols = symbols;
    symbols.sort();
    let symbols_key = symbols.join(",").to_uppercase();
    let cache_key = Cache::key("dividends", &[&symbols_key, range_str]);

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::HISTORICAL,
            cache::is_market_open(),
            || async move {
                let tickers = Tickers::new(symbols).await?;
                let batch_response = tickers.dividends(range).await?;
                info!(
                    "Dividends fetch complete: {} success, {} errors",
                    batch_response.success_count(),
                    batch_response.error_count()
                );
                serde_json::to_value(&batch_response).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

pub async fn get_batch_splits(
    cache: &Cache,
    symbols: Vec<&str>,
    range: TimeRange,
    range_str: &str,
) -> ServiceResult {
    let mut symbols = symbols;
    symbols.sort();
    let symbols_key = symbols.join(",").to_uppercase();
    let cache_key = Cache::key("splits", &[&symbols_key, range_str]);

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::HISTORICAL,
            cache::is_market_open(),
            || async move {
                let tickers = Tickers::new(symbols).await?;
                let batch_response = tickers.splits(range).await?;
                info!(
                    "Splits fetch complete: {} success, {} errors",
                    batch_response.success_count(),
                    batch_response.error_count()
                );
                serde_json::to_value(&batch_response).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

pub async fn get_batch_capital_gains(
    cache: &Cache,
    symbols: Vec<&str>,
    range: TimeRange,
    range_str: &str,
) -> ServiceResult {
    let mut symbols = symbols;
    symbols.sort();
    let symbols_key = symbols.join(",").to_uppercase();
    let cache_key = Cache::key("capital-gains", &[&symbols_key, range_str]);

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::HISTORICAL,
            cache::is_market_open(),
            || async move {
                let tickers = Tickers::new(symbols).await?;
                let batch_response = tickers.capital_gains(range).await?;
                info!(
                    "Capital gains fetch complete: {} success, {} errors",
                    batch_response.success_count(),
                    batch_response.error_count()
                );
                serde_json::to_value(&batch_response).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}
