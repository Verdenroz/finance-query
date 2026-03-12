use crate::cache::{self, Cache};

use super::{ServiceError, ServiceResult};

pub async fn get_coins(cache: &Cache, vs_currency: &str, count: usize) -> ServiceResult {
    let cache_key = Cache::key("crypto_coins", &[vs_currency, &count.to_string()]);
    let vs = vs_currency.to_string();

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::QUOTES,
            cache::is_market_open(),
            || async move {
                let coins = finance_query::crypto::coins(&vs, count).await?;
                serde_json::to_value(&coins).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

pub async fn get_coin(cache: &Cache, coin_id: &str, vs_currency: &str) -> ServiceResult {
    let cache_key = Cache::key("crypto_coin", &[coin_id, vs_currency]);
    let id = coin_id.to_string();
    let vs = vs_currency.to_string();

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::QUOTES,
            cache::is_market_open(),
            || async move {
                let coin = finance_query::crypto::coin(&id, &vs).await?;
                serde_json::to_value(&coin).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}
