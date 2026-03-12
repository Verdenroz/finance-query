use crate::cache::{self, Cache};
use finance_query::finance;

use super::{ServiceError, ServiceResult};

pub async fn get_transcript(
    cache: &Cache,
    symbol: &str,
    quarter: Option<&str>,
    year: Option<i32>,
) -> ServiceResult {
    let quarter_str = quarter.unwrap_or("latest");
    let year_str = year
        .map(|y| y.to_string())
        .unwrap_or_else(|| "latest".to_string());
    let cache_key = Cache::key(
        "transcript",
        &[&symbol.to_uppercase(), quarter_str, &year_str],
    );
    let symbol = symbol.to_string();
    let quarter = quarter.map(|s| s.to_string());

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::TRANSCRIPT,
            cache::is_market_open(),
            || async move {
                let response =
                    finance::earnings_transcript(&symbol, quarter.as_deref(), year).await?;
                serde_json::to_value(&response).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

pub async fn get_transcripts(cache: &Cache, symbol: &str, limit: Option<usize>) -> ServiceResult {
    let limit_str = limit
        .map(|l| l.to_string())
        .unwrap_or_else(|| "all".to_string());
    let cache_key = Cache::key("transcripts", &[&symbol.to_uppercase(), &limit_str]);
    let symbol = symbol.to_string();

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::EARNINGS_LIST,
            cache::is_market_open(),
            || async move {
                let transcripts = finance::earnings_transcripts(&symbol, limit).await?;
                serde_json::to_value(&transcripts).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}
