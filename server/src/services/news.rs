use std::sync::Arc;

use crate::cache::{self, Cache};
use finance_query::{Providers, Ticker, finance};
use tracing::info;

use super::{ServiceError, ServiceResult, lang_key};

pub async fn get_news(
    cache: &Cache,
    symbol: &str,
    count: usize,
    lang: Option<&str>,
) -> ServiceResult {
    let cache_key = Cache::key("news", &[&symbol.to_uppercase(), lang_key(lang)]);
    let symbol = symbol.to_string();
    let lang = lang.map(str::to_string);

    let json = cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::NEWS,
            cache::is_market_open(),
            || async move {
                let builder = Ticker::builder(&symbol);
                let builder = match &lang {
                    Some(lang) => builder.lang(lang),
                    None => builder,
                };
                let ticker = builder.build().await?;
                let news = ticker.news().await?;
                serde_json::to_value(&news).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await?;

    Ok(truncate_array(json, count))
}

pub async fn get_general_news(cache: &Cache, count: usize, lang: Option<&str>) -> ServiceResult {
    let cache_key = Cache::key("news", &["general", lang_key(lang)]);
    let lang = lang.map(str::to_string);

    let json = cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::GENERAL_NEWS,
            cache::is_market_open(),
            || async move {
                let mut news = finance::news().await?;
                super::translate(&mut news, lang.as_deref()).await?;
                info!("Fetched general market news");
                serde_json::to_value(&news).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await?;

    Ok(truncate_array(json, count))
}

/// A company's own press releases, via `Capability::CORPORATE` (EDGAR 8-K
/// exhibits, falling back to FMP/Alpha Vantage when configured). Distinct
/// from `get_news`, which returns press coverage.
pub async fn get_press_releases(
    cache: &Cache,
    providers: &Arc<Providers>,
    symbol: &str,
    limit: u32,
) -> ServiceResult {
    let cache_key = Cache::key(
        "press-releases",
        &[&symbol.to_uppercase(), &limit.to_string()],
    );
    let providers = Arc::clone(providers);
    let symbol = symbol.to_string();

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::NEWS,
            cache::is_market_open(),
            || async move {
                let ticker = providers.ticker(&symbol).build().await?;
                let releases = ticker.press_releases(limit).await?;
                serde_json::to_value(&releases).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

/// Truncate a JSON array value to at most `count` elements.
/// Non-array values are returned unchanged.
fn truncate_array(mut value: serde_json::Value, count: usize) -> serde_json::Value {
    if let serde_json::Value::Array(ref mut arr) = value {
        arr.truncate(count);
    }
    value
}
