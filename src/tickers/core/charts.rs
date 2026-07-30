use super::{BatchChartsResponse, BatchSparksResponse, Tickers};
use crate::constants::{Interval, TimeRange};
use crate::error::{FinanceError, Result};
use crate::models::chart::Chart;
use crate::models::chart::events::ChartEvents;
use crate::providers::Capability;
use crate::utils::CacheEntry;
use futures::stream::{self, StreamExt};
use std::sync::Arc;

impl Tickers {
    /// Batch fetch charts for all symbols concurrently
    ///
    /// Chart data cannot be batched in a single request, so this fetches
    /// all charts concurrently using tokio for maximum performance.
    pub async fn charts(
        &self,
        interval: Interval,
        range: TimeRange,
    ) -> Result<BatchChartsResponse> {
        // Fast path: check if all symbols are cached
        {
            let cache = self.chart_cache.read().await;
            if self.all_cached(
                &cache,
                self.symbols.iter().map(|s| (s.clone(), interval, range)),
            ) {
                let mut response = BatchChartsResponse::with_capacity(self.symbols.len());
                for symbol in &self.symbols {
                    if let Some(entry) = cache.get(&(symbol.clone(), interval, range)) {
                        response
                            .charts
                            .insert(symbol.to_string(), entry.value.clone());
                    }
                }
                return Ok(response);
            }
        }

        // Slow path: acquire fetch guard to prevent duplicate concurrent requests
        let fetch_guard = Self::get_fetch_guard(&self.charts_fetch, (interval, range)).await;
        let _guard = fetch_guard.lock().await;

        // Double-check: another task may have fetched while we waited
        {
            let cache = self.chart_cache.read().await;
            if self.all_cached(
                &cache,
                self.symbols.iter().map(|s| (s.clone(), interval, range)),
            ) {
                let mut response = BatchChartsResponse::with_capacity(self.symbols.len());
                for symbol in &self.symbols {
                    if let Some(entry) = cache.get(&(symbol.clone(), interval, range)) {
                        response
                            .charts
                            .insert(symbol.to_string(), entry.value.clone());
                    }
                }
                return Ok(response);
            }
        }

        // Fetch all charts concurrently via provider dispatch (no lock held during I/O)
        let futures: Vec<_> = self
            .symbols
            .iter()
            .map(|symbol| {
                let providers = Arc::clone(&self.providers);
                let symbol = Arc::clone(symbol);
                async move {
                    let sym = symbol.to_string();
                    let result = providers
                        .fetch(Capability::CHART, |p| {
                            let sym = sym.clone();
                            let p = p.clone();
                            async move { p.fetch_chart(&sym, interval, range).await }
                        })
                        .await;
                    (symbol, result)
                }
            })
            .collect();

        let results: Vec<_> = stream::iter(futures)
            .buffer_unordered(self.max_concurrency)
            .collect()
            .await;

        let mut response = BatchChartsResponse::with_capacity(self.symbols.len());
        let mut parsed_charts: Vec<(Arc<str>, Chart)> = Vec::new();

        for (symbol, result) in results {
            match result {
                Ok(data) => {
                    let chart = data;
                    parsed_charts.push((symbol, chart));
                }
                Err(e) => {
                    response.errors.insert(symbol.to_string(), e.to_string());
                }
            }
        }

        // Move into cache, then clone for response — avoids double-clone
        if self.cache_ttl.is_some() {
            let mut cache = self.chart_cache.write().await;
            let cache_keys: Vec<_> = parsed_charts
                .into_iter()
                .map(|(symbol, chart)| {
                    self.cache_insert(&mut cache, (symbol.clone(), interval, range), chart);
                    symbol
                })
                .collect();
            for symbol in cache_keys {
                if let Some(cached) = cache.get(&(symbol.clone(), interval, range)) {
                    response
                        .charts
                        .insert(symbol.to_string(), cached.value.clone());
                }
            }
        } else {
            for (symbol, chart) in parsed_charts {
                response.charts.insert(symbol.to_string(), chart);
            }
        }

        Ok(response)
    }

    /// Get a specific chart by symbol
    pub async fn chart(&self, symbol: &str, interval: Interval, range: TimeRange) -> Result<Chart> {
        {
            let cache = self.chart_cache.read().await;
            let key: Arc<str> = symbol.into();
            if let Some(entry) = cache.get(&(key, interval, range))
                && self.is_cache_fresh(Some(entry))
            {
                return Ok(entry.value.clone());
            }
        }

        let response = self.charts(interval, range).await?;

        response
            .charts
            .get(symbol)
            .cloned()
            .ok_or_else(|| FinanceError::SymbolNotFound {
                symbol: Some(symbol.to_string()),
                context: response
                    .errors
                    .get(symbol)
                    .cloned()
                    .unwrap_or_else(|| "Symbol not found".to_string()),
            })
    }

    /// Batch fetch chart data for a custom date range for all symbols concurrently.
    ///
    /// Unlike [`charts()`](Self::charts) which uses predefined time ranges,
    /// this method accepts absolute start/end timestamps. Results are **not cached**
    /// since custom ranges have unbounded key space.
    ///
    /// # Arguments
    ///
    /// * `interval` - Time interval between data points
    /// * `start` - Start date as Unix timestamp (seconds since epoch)
    /// * `end` - End date as Unix timestamp (seconds since epoch)
    pub async fn charts_range(
        &self,
        interval: Interval,
        start: i64,
        end: i64,
    ) -> Result<BatchChartsResponse> {
        let futures: Vec<_> = self
            .symbols
            .iter()
            .map(|symbol| {
                let providers = Arc::clone(&self.providers);
                let symbol = Arc::clone(symbol);
                async move {
                    let sym = symbol.to_string();
                    let result = providers
                        .fetch(Capability::CHART, |p| {
                            let sym = sym.clone();
                            let p = p.clone();
                            async move { p.fetch_chart_range(&sym, interval, start, end).await }
                        })
                        .await;
                    (symbol, result)
                }
            })
            .collect();

        let results: Vec<_> = stream::iter(futures)
            .buffer_unordered(self.max_concurrency)
            .collect()
            .await;

        let mut response = BatchChartsResponse::with_capacity(self.symbols.len());

        for (symbol, result) in results {
            match result {
                Ok(data) => {
                    let chart = data;
                    response.charts.insert(symbol.to_string(), chart);
                }
                Err(e) => {
                    response.errors.insert(symbol.to_string(), e.to_string());
                }
            }
        }

        Ok(response)
    }

    /// Ensures events are loaded for all symbols using chart requests.
    ///
    /// Fetches events concurrently for symbols that don't have cached events.
    /// Uses `TimeRange::Max` to get full event history (Yahoo returns all
    /// dividends/splits/capital gains regardless of chart range).
    ///
    /// Events are always stored regardless of `cache_ttl` because they are
    /// derived data (not a TTL-bounded cache). When `cache_ttl` is `None`,
    /// events persist for the lifetime of the `Tickers` instance.
    pub(super) async fn ensure_events_loaded(&self) -> Result<()> {
        // Check which symbols need event data (existence check, not TTL-based)
        let symbols_to_fetch: Vec<Arc<str>> = {
            let cache = self.events_cache.read().await;
            self.symbols
                .iter()
                .filter(|sym| !cache.contains_key(*sym))
                .cloned()
                .collect()
        };

        if symbols_to_fetch.is_empty() {
            return Ok(());
        }

        // Fetch events concurrently for all symbols that need it via provider dispatch
        let futures: Vec<_> = symbols_to_fetch
            .iter()
            .map(|symbol| {
                let providers = Arc::clone(&self.providers);
                let symbol = Arc::clone(symbol);
                async move {
                    let sym = symbol.to_string();
                    let result = providers
                        .fetch(Capability::CORPORATE, |p| {
                            let sym = sym.clone();
                            let p = p.clone();
                            async move { p.fetch_events(&sym).await }
                        })
                        .await;
                    (symbol, result)
                }
            })
            .collect();

        let results: Vec<_> = stream::iter(futures)
            .buffer_unordered(self.max_concurrency)
            .collect()
            .await;

        let mut parsed_events: Vec<(Arc<str>, ChartEvents)> = Vec::new();

        for (symbol, result) in results {
            if let Ok(events_data) = result {
                parsed_events.push((symbol, events_data));
            }
        }

        // Always store events — they are derived data, not TTL-bounded cache
        if !parsed_events.is_empty() {
            let mut events_cache = self.events_cache.write().await;
            for (symbol, events) in parsed_events {
                events_cache.insert(symbol, CacheEntry::new(events));
            }
        }

        Ok(())
    }

    /// Batch fetch spark data for all symbols in a single request.
    ///
    /// Spark data is optimized for sparkline rendering, returning only close prices.
    /// Unlike `charts()`, this fetches all symbols in ONE API call, making it
    /// much more efficient for displaying price trends on dashboards or watchlists.
    ///
    /// # Arguments
    ///
    /// * `interval` - Time interval between data points (e.g., `Interval::FiveMinutes`)
    /// * `range` - Time range to fetch (e.g., `TimeRange::OneDay`)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::{Tickers, Interval, TimeRange};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let tickers = Tickers::new(["AAPL", "MSFT", "GOOGL"]).await?;
    /// let sparks = tickers.spark(Interval::FiveMinutes, TimeRange::OneDay).await?;
    ///
    /// for (symbol, spark) in &sparks.sparks {
    ///     if let Some(change) = spark.percent_change() {
    ///         println!("{}: {:.2}%", symbol, change);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn spark(&self, interval: Interval, range: TimeRange) -> Result<BatchSparksResponse> {
        // Fast path: check if all symbols are cached
        {
            let cache = self.spark_cache.read().await;
            if self.all_cached(
                &cache,
                self.symbols.iter().map(|s| (s.clone(), interval, range)),
            ) {
                let mut response = BatchSparksResponse::with_capacity(self.symbols.len());
                for symbol in &self.symbols {
                    if let Some(entry) = cache.get(&(symbol.clone(), interval, range)) {
                        response
                            .sparks
                            .insert(symbol.to_string(), entry.value.clone());
                    }
                }
                return Ok(response);
            }
        }

        // Slow path: acquire fetch guard
        let fetch_guard = Self::get_fetch_guard(&self.spark_fetch, (interval, range)).await;
        let _guard = fetch_guard.lock().await;

        // Double-check after guard
        {
            let cache = self.spark_cache.read().await;
            if self.all_cached(
                &cache,
                self.symbols.iter().map(|s| (s.clone(), interval, range)),
            ) {
                let mut response = BatchSparksResponse::with_capacity(self.symbols.len());
                for symbol in &self.symbols {
                    if let Some(entry) = cache.get(&(symbol.clone(), interval, range)) {
                        response
                            .sparks
                            .insert(symbol.to_string(), entry.value.clone());
                    }
                }
                return Ok(response);
            }
        }

        // Dispatch through the provider set under the CHART capability so spark
        // honors routing like every other chart path (Yahoo is the default).
        let providers = Arc::clone(&self.providers);
        let syms: Vec<String> = self.symbols.iter().map(|s| s.to_string()).collect();
        let spark_result = providers
            .fetch(Capability::CHART, |p| {
                let syms = syms.clone();
                let p = p.clone();
                async move {
                    let syms_ref: Vec<&str> = syms.iter().map(String::as_str).collect();
                    p.fetch_spark(&syms_ref, interval, range).await
                }
            })
            .await;

        let mut response = BatchSparksResponse::with_capacity(self.symbols.len());

        match spark_result {
            Ok(parsed_sparks) => {
                // Cache all parsed sparks
                if self.cache_ttl.is_some() {
                    let mut cache = self.spark_cache.write().await;
                    for (symbol, spark) in &parsed_sparks {
                        let key: Arc<str> = symbol.as_str().into();
                        self.cache_insert(&mut cache, (key, interval, range), spark.clone());
                    }
                }

                // Build response
                for (symbol, spark) in parsed_sparks {
                    response.sparks.insert(symbol, spark);
                }

                // Track missing symbols
                for symbol in &self.symbols {
                    let symbol_str = &**symbol;
                    if !response.sparks.contains_key(symbol_str)
                        && !response.errors.contains_key(symbol_str)
                    {
                        response.errors.insert(
                            symbol.to_string(),
                            "Symbol not found in response".to_string(),
                        );
                    }
                }
            }
            Err(e) => {
                for symbol in &self.symbols {
                    response.errors.insert(symbol.to_string(), e.to_string());
                }
            }
        }

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_tickers_charts() {
        let tickers = Tickers::new(["AAPL", "MSFT"]).await.unwrap();
        let result = tickers
            .charts(Interval::OneDay, TimeRange::FiveDays)
            .await
            .unwrap();

        assert!(result.success_count() > 0);
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_tickers_spark() {
        let tickers = Tickers::new(["AAPL", "MSFT", "GOOGL"]).await.unwrap();
        let result = tickers
            .spark(Interval::FiveMinutes, TimeRange::OneDay)
            .await
            .unwrap();

        assert!(result.success_count() > 0);

        // Verify spark data structure
        if let Some(spark) = result.sparks.get("AAPL") {
            assert!(!spark.closes.is_empty());
            assert_eq!(spark.symbol, "AAPL");
            // Verify helper methods work
            assert!(spark.percent_change().is_some());
        }
    }
}
