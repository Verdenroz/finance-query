use std::sync::Arc;

use crate::cache::{self, Cache};
use finance_query::Providers;

use super::{ServiceError, ServiceResult};

pub async fn get_series(cache: &Cache, series_id: &str) -> ServiceResult {
    let cache_key = Cache::key("fred_series", &[&series_id.to_uppercase()]);
    let id = series_id.to_string();

    cache
        .get_or_fetch(&cache_key, cache::ttl::ANALYSIS, false, || async move {
            let series = finance_query::fred::series(&id).await?;
            serde_json::to_value(&series).map_err(|e| Box::new(e) as ServiceError)
        })
        .await
}

pub async fn get_treasury_yields(cache: &Cache, year: u32) -> ServiceResult {
    let cache_key = Cache::key("treasury_yields", &[&year.to_string()]);

    cache
        .get_or_fetch(&cache_key, cache::ttl::METADATA, false, || async move {
            let yields = finance_query::fred::treasury_yields(year).await?;
            serde_json::to_value(&yields).map_err(|e| Box::new(e) as ServiceError)
        })
        .await
}

/// A series as it stood on `date` — FRED/ALFRED's point-in-time vintage view.
pub async fn get_series_as_of(
    cache: &Cache,
    providers: &Arc<Providers>,
    series_id: &str,
    date: &str,
) -> ServiceResult {
    let cache_key = Cache::key("fred_series_as_of", &[&series_id.to_uppercase(), date]);
    let providers = Arc::clone(providers);
    let id = series_id.to_string();
    let date = date.to_string();
    cache
        .get_or_fetch(&cache_key, cache::ttl::ANALYSIS, false, || async move {
            let series = providers.economic(&id).as_of(&date).await?;
            serde_json::to_value(&series).map_err(|e| Box::new(e) as ServiceError)
        })
        .await
}
