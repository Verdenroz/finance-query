use crate::cache::{self, Cache};
use finance_query::{Region, finance};

use super::{ServiceError, ServiceResult};

/// Boolean feature flags for a search request.
#[derive(Clone, Copy, Default)]
pub struct SearchFlags {
    pub fuzzy: bool,
    pub logo: bool,
    pub research: bool,
    pub cultural: bool,
}

pub async fn search(
    cache: &Cache,
    query: &str,
    quotes: u32,
    news: u32,
    flags: SearchFlags,
    region: Option<Region>,
) -> ServiceResult {
    let cache_key = Cache::key(
        "search",
        &[
            &query.to_lowercase(),
            &quotes.to_string(),
            &news.to_string(),
            if flags.logo { "1" } else { "0" },
        ],
    );
    let query = query.to_string();

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::SEARCH,
            cache::is_market_open(),
            || async move {
                let mut options = finance::SearchOptions::new()
                    .quotes_count(quotes)
                    .news_count(news)
                    .enable_fuzzy_query(flags.fuzzy)
                    .enable_logo_url(flags.logo)
                    .enable_research_reports(flags.research)
                    .enable_cultural_assets(flags.cultural);

                if let Some(r) = region {
                    options = options.region(r);
                }

                let result = finance::search(&query, &options).await?;
                serde_json::to_value(&result).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

pub async fn lookup(
    cache: &Cache,
    query: &str,
    lookup_type: finance::LookupType,
    count: u32,
    logo: bool,
    region: Option<Region>,
) -> ServiceResult {
    let type_str = format!("{:?}", lookup_type).to_lowercase();
    let cache_key = Cache::key(
        "lookup",
        &[
            &query.to_lowercase(),
            &type_str,
            &count.to_string(),
            if logo { "1" } else { "0" },
        ],
    );
    let query = query.to_string();

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::SEARCH,
            cache::is_market_open(),
            || async move {
                let mut options = finance::LookupOptions::new()
                    .lookup_type(lookup_type)
                    .count(count)
                    .include_logo(logo);

                if let Some(r) = region {
                    options = options.region(r);
                }

                let result = finance::lookup(&query, &options).await?;
                serde_json::to_value(&result).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}
