use crate::cache::{self, Cache};
use finance_query::{Interval, Ticker, TimeRange};

use super::{ServiceError, ServiceResult};

pub async fn get_risk(
    cache: &Cache,
    symbol: &str,
    interval: Interval,
    interval_str: &str,
    range: TimeRange,
    range_str: &str,
    benchmark: Option<&str>,
) -> ServiceResult {
    let cache_key = Cache::key(
        "risk",
        &[
            &symbol.to_uppercase(),
            interval_str,
            range_str,
            benchmark.unwrap_or(""),
        ],
    );
    let symbol = symbol.to_string();
    let benchmark = benchmark.map(|s| s.to_string());

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::HISTORICAL,
            cache::is_market_open(),
            || async move {
                let ticker = Ticker::new(&symbol).await?;
                let summary = ticker.risk(interval, range, benchmark.as_deref()).await?;
                serde_json::to_value(&summary).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}
