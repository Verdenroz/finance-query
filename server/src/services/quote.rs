use crate::cache::{self, Cache};
use finance_query::{Ticker, Tickers};
use tracing::info;

use super::{ServiceError, ServiceResult};

pub async fn get_quote(cache: &Cache, symbol: &str, logo: bool) -> ServiceResult {
    let logo_str = if logo { "1" } else { "0" };
    let cache_key = Cache::key("quote", &[&symbol.to_uppercase(), logo_str]);
    let symbol = symbol.to_string();

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::QUOTES,
            cache::is_market_open(),
            || async move {
                let builder = Ticker::builder(&symbol);
                let builder = if logo { builder.logo() } else { builder };
                let ticker = builder.build().await?;
                let quote = ticker.quote().await?;
                info!("Successfully fetched quote for {}", symbol);
                serde_json::to_value(&quote).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

pub async fn get_quotes(cache: &Cache, symbols: Vec<&str>, logo: bool) -> ServiceResult {
    let mut symbols = symbols;
    symbols.sort();
    let logo_str = if logo { "1" } else { "0" };
    let symbols_key = symbols.join(",").to_uppercase();
    let cache_key = Cache::key("quotes", &[&symbols_key, logo_str]);

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::QUOTES,
            cache::is_market_open(),
            || async move {
                let builder = Tickers::builder(symbols);
                let builder = if logo { builder.logo() } else { builder };
                let tickers = builder.build().await?;
                let batch_response = tickers.quotes().await?;
                info!(
                    "Batch fetch complete: {} success, {} errors",
                    batch_response.success_count(),
                    batch_response.error_count()
                );
                serde_json::to_value(&batch_response).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}
