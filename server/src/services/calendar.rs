use crate::cache::{self, Cache};
use finance_query::{Tickers, TimeRange};
use tracing::info;

use super::{ServiceError, ServiceResult};

/// Aggregate upcoming financial events (earnings, dividends, options
/// expirations, and — when `FRED_API_KEY` is set — economic releases) across
/// the given symbols into a single time-sorted list.
pub async fn get_calendar(
    cache: &Cache,
    symbols: Vec<&str>,
    range: TimeRange,
    range_str: &str,
) -> ServiceResult {
    let mut symbols = symbols;
    symbols.sort();
    let symbols_key = symbols.join(",").to_uppercase();
    let cache_key = Cache::key("calendar", &[&symbols_key, range_str]);

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::HISTORICAL,
            cache::is_market_open(),
            || async move {
                let tickers = Tickers::new(symbols).await?;
                let events = tickers.calendar(range).await?;
                info!("Calendar fetch complete: {} events", events.len());
                serde_json::to_value(&events).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}
