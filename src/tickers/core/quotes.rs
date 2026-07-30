use super::{BatchQuotesResponse, Tickers};
use crate::error::{FinanceError, Result};
use crate::format::Both;
use crate::models::format::Format;
use crate::models::quote::{Quote, QuoteSummaryResponse};
use crate::providers::Capability;
use futures::stream::{self, StreamExt};
use std::collections::HashMap;
use std::sync::Arc;

impl Tickers {
    /// Batch fetch quotes for all symbols.
    ///
    /// Dispatches through the configured provider set. When logos are enabled,
    /// fetches logo URLs in parallel via the Yahoo client.
    ///
    /// Use [`TickersBuilder::logo()`](TickersBuilder::logo) to enable logo fetching
    /// for this tickers instance.
    pub async fn quotes(&self) -> Result<BatchQuotesResponse> {
        // Fast path: check if all symbols are cached
        {
            let cache = self.quote_cache.read().await;
            if self.all_cached(&cache, self.symbols.iter().cloned()) {
                let mut response = BatchQuotesResponse::with_capacity(self.symbols.len());
                for symbol in &self.symbols {
                    if let Some(entry) = cache.get(symbol) {
                        response
                            .quotes
                            .insert(symbol.to_string(), entry.value.clone());
                    }
                }
                return Ok(response);
            }
        }

        let _fetch_guard = self.quotes_fetch.lock().await;

        // Double-check: another task may have fetched while we waited
        {
            let cache = self.quote_cache.read().await;
            if self.all_cached(&cache, self.symbols.iter().cloned()) {
                let mut response = BatchQuotesResponse::with_capacity(self.symbols.len());
                for symbol in &self.symbols {
                    if let Some(entry) = cache.get(symbol) {
                        response
                            .quotes
                            .insert(symbol.to_string(), entry.value.clone());
                    }
                }
                return Ok(response);
            }
        }

        let symbol_strings: Vec<String> = self.symbols.iter().map(|s| s.to_string()).collect();
        let mut response = BatchQuotesResponse::with_capacity(self.symbols.len());

        let (quote_data, logos) = if self.include_logo {
            // Fire logo fetch in parallel with quote fetch; logos are Yahoo-only
            let providers_logo = Arc::clone(&self.providers);
            let syms_logo = symbol_strings.clone();
            let logo_future = async move {
                if let Ok(client) = providers_logo.first_yahoo() {
                    let syms_ref: Vec<&str> = syms_logo.iter().map(String::as_str).collect();
                    crate::adapters::yahoo::quote::quotes::fetch_with_fields(
                        &client,
                        &syms_ref,
                        Some(&["logoUrl", "companyLogoUrl"]),
                        true,
                        true,
                    )
                    .await
                    .ok()
                } else {
                    None
                }
            };

            let providers_quote = Arc::clone(&self.providers);
            let syms_quote = symbol_strings.clone();
            let quote_future = async move {
                providers_quote
                    .fetch(Capability::QUOTE, |p| {
                        let syms = syms_quote.clone();
                        let p = p.clone();
                        async move {
                            let syms_ref: Vec<&str> = syms.iter().map(String::as_str).collect();
                            p.fetch_quotes_batch(&syms_ref).await
                        }
                    })
                    .await
            };

            let (batch_result, logo_result) = tokio::join!(quote_future, logo_future);
            let quote_data = match batch_result {
                Ok(data) => data,
                Err(_) => {
                    self.fetch_quotes_per_symbol(&symbol_strings, &mut response)
                        .await
                }
            };
            (quote_data, logo_result)
        } else {
            let providers = Arc::clone(&self.providers);
            let syms = symbol_strings.clone();
            let batch_result = providers
                .fetch(Capability::QUOTE, |p| {
                    let syms = syms.clone();
                    let p = p.clone();
                    async move {
                        let syms_ref: Vec<&str> = syms.iter().map(String::as_str).collect();
                        p.fetch_quotes_batch(&syms_ref).await
                    }
                })
                .await;
            let data = match batch_result {
                Ok(data) => data,
                Err(_) => {
                    self.fetch_quotes_per_symbol(&symbol_strings, &mut response)
                        .await
                }
            };
            (data, None)
        };

        let logo_map: HashMap<String, (Option<String>, Option<String>)> = logos
            .and_then(|l| l.get("quoteResponse")?.get("result")?.as_array().cloned())
            .map(|results| {
                results
                    .iter()
                    .filter_map(|r| {
                        let symbol = r.get("symbol")?.as_str()?.to_string();
                        let logo_url = r.get("logoUrl").and_then(|v| v.as_str()).map(String::from);
                        let company_logo_url = r
                            .get("companyLogoUrl")
                            .and_then(|v| v.as_str())
                            .map(String::from);
                        Some((symbol, (logo_url, company_logo_url)))
                    })
                    .collect()
            })
            .unwrap_or_default();

        let mut parsed_quotes: Vec<(String, Quote)> = Vec::new();

        for (symbol, summary) in quote_data {
            let logo_url = logo_map.get(&symbol).and_then(|(l, _)| l.clone());
            let company_logo_url = logo_map.get(&symbol).and_then(|(_, c)| c.clone());
            let quote = Quote::from_response(&summary, logo_url, company_logo_url);
            parsed_quotes.push((symbol, quote));
        }

        for (symbol, quote) in parsed_quotes {
            response.quotes.insert(symbol, quote);
        }

        // Translate before caching so cached quotes are already localized
        // and repeat reads don't re-run the translation backend.
        #[cfg(feature = "translation")]
        self.translate_response(&mut response).await?;

        if self.cache_ttl.is_some() {
            let mut cache = self.quote_cache.write().await;
            for (symbol, quote) in &response.quotes {
                self.cache_insert(&mut cache, symbol.as_str().into(), quote.clone());
            }
        }

        // Track missing symbols
        for symbol in &self.symbols {
            let s = &**symbol;
            if !response.quotes.contains_key(s) && !response.errors.contains_key(s) {
                response.errors.insert(
                    symbol.to_string(),
                    "Symbol not found in response".to_string(),
                );
            }
        }

        Ok(response)
    }

    /// Fallback for when no provider supports `fetch_quotes_batch`.
    /// Fetches each symbol individually; failures go into `response.errors`.
    async fn fetch_quotes_per_symbol(
        &self,
        symbols: &[String],
        response: &mut BatchQuotesResponse,
    ) -> Vec<(String, QuoteSummaryResponse)> {
        let futures: Vec<_> = symbols
            .iter()
            .map(|sym| {
                let providers = Arc::clone(&self.providers);
                let sym = sym.clone();
                async move {
                    let result = providers
                        .fetch(Capability::QUOTE, |p| {
                            let sym = sym.clone();
                            let p = p.clone();
                            async move { p.fetch_quote(&sym).await }
                        })
                        .await;
                    (sym, result)
                }
            })
            .collect();

        let results: Vec<_> = stream::iter(futures)
            .buffer_unordered(self.max_concurrency)
            .collect()
            .await;

        let mut successes = Vec::new();
        for (sym, result) in results {
            match result {
                Ok(resp) => successes.push((sym, resp)),
                Err(e) => {
                    response.errors.insert(sym, e.to_string());
                }
            }
        }
        successes
    }

    /// Get a specific quote by symbol (from cache or fetch all)
    pub async fn quote<F>(&self, symbol: &str) -> Result<Quote<F>>
    where
        F: Format,
        Quote<Both>: Into<Quote<F>>,
    {
        {
            let cache = self.quote_cache.read().await;
            if let Some(entry) = cache.get(symbol)
                && self.is_cache_fresh(Some(entry))
            {
                return Ok(entry.value.clone().into());
            }
        }

        let response = self.quotes().await?;

        response
            .quotes
            .get(symbol)
            .cloned()
            .map(Into::into)
            .ok_or_else(|| FinanceError::SymbolNotFound {
                symbol: Some(symbol.to_string()),
                context: response
                    .errors
                    .get(symbol)
                    .cloned()
                    .unwrap_or_else(|| "Symbol not found".to_string()),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_tickers_quotes() {
        let tickers = Tickers::new(["AAPL", "MSFT", "GOOGL"]).await.unwrap();
        let result = tickers.quotes().await.unwrap();

        assert!(result.success_count() > 0);
    }
}
