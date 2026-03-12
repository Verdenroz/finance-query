use crate::cache::{self, Cache};
use finance_query::feeds::FeedSource;

use super::{ServiceError, ServiceResult};

pub async fn get_feeds(
    cache: &Cache,
    sources: &[FeedSource],
    source_key: &str,
    form_type_key: &str,
) -> ServiceResult {
    let cache_key = Cache::key("feeds", &[source_key, form_type_key]);
    let sources = sources.to_vec();

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::GENERAL_NEWS,
            cache::is_market_open(),
            || async move {
                let entries = finance_query::feeds::fetch_all(&sources).await?;
                serde_json::to_value(&entries).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}
