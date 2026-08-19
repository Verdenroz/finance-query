use super::{BatchCapitalGainsResponse, BatchDividendsResponse, BatchSplitsResponse, Tickers};
use crate::constants::TimeRange;
use crate::error::Result;
use crate::models::chart::events::ChartEvents;
use crate::providers::Capability;
use crate::utils::{CacheEntry, filter_by_range};
use futures::stream::{self, StreamExt};
use std::sync::Arc;

impl Tickers {
    /// Batch fetch dividends for all symbols
    ///
    /// Returns dividend history for all symbols, filtered by the specified time range.
    /// Dividends are cached per symbol after the first chart fetch.
    ///
    /// # Arguments
    ///
    /// * `range` - Time range to filter dividends
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::{Tickers, TimeRange};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let tickers = Tickers::new(["AAPL", "MSFT"]).await?;
    /// let dividends = tickers.dividends(TimeRange::OneYear).await?;
    ///
    /// for (symbol, divs) in &dividends.dividends {
    ///     println!("{}: {} dividends", symbol, divs.len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn dividends(&self, range: TimeRange) -> Result<BatchDividendsResponse> {
        let mut response = BatchDividendsResponse::with_capacity(self.symbols.len());

        // Fetch events efficiently (1-day chart request per symbol)
        self.ensure_events_loaded().await?;

        let events_cache = self.events_cache.read().await;

        for symbol in &self.symbols {
            if let Some(entry) = events_cache.get(symbol) {
                let all_dividends = entry.value.to_dividends();
                let filtered = filter_by_range(all_dividends, range);
                response.dividends.insert(symbol.to_string(), filtered);
            } else {
                response
                    .errors
                    .insert(symbol.to_string(), "No events data available".to_string());
            }
        }

        Ok(response)
    }

    /// Batch fetch stock splits for all symbols
    ///
    /// Returns stock split history for all symbols, filtered by the specified time range.
    /// Splits are cached per symbol after the first chart fetch.
    ///
    /// # Arguments
    ///
    /// * `range` - Time range to filter splits
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::{Tickers, TimeRange};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let tickers = Tickers::new(["NVDA", "TSLA"]).await?;
    /// let splits = tickers.splits(TimeRange::FiveYears).await?;
    ///
    /// for (symbol, sp) in &splits.splits {
    ///     for split in sp {
    ///         println!("{}: {}", symbol, split.ratio);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn splits(&self, range: TimeRange) -> Result<BatchSplitsResponse> {
        let mut response = BatchSplitsResponse::with_capacity(self.symbols.len());

        // Fetch events efficiently (1-day chart request per symbol)
        self.ensure_events_loaded().await?;

        let events_cache = self.events_cache.read().await;

        for symbol in &self.symbols {
            if let Some(entry) = events_cache.get(symbol) {
                let all_splits = entry.value.to_splits();
                let filtered = filter_by_range(all_splits, range);
                response.splits.insert(symbol.to_string(), filtered);
            } else {
                response
                    .errors
                    .insert(symbol.to_string(), "No events data available".to_string());
            }
        }

        Ok(response)
    }

    /// Batch fetch capital gains for all symbols
    ///
    /// Returns capital gain distribution history for all symbols, filtered by the
    /// specified time range. This is primarily relevant for mutual funds and ETFs.
    /// Capital gains are cached per symbol after the first chart fetch.
    ///
    /// # Arguments
    ///
    /// * `range` - Time range to filter capital gains
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::{Tickers, TimeRange};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let tickers = Tickers::new(["VFIAX", "VTI"]).await?;
    /// let gains = tickers.capital_gains(TimeRange::TwoYears).await?;
    ///
    /// for (symbol, cg) in &gains.capital_gains {
    ///     println!("{}: {} distributions", symbol, cg.len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn capital_gains(&self, range: TimeRange) -> Result<BatchCapitalGainsResponse> {
        let mut response = BatchCapitalGainsResponse::with_capacity(self.symbols.len());

        // Fetch events efficiently (1-day chart request per symbol)
        self.ensure_events_loaded().await?;

        let events_cache = self.events_cache.read().await;

        for symbol in &self.symbols {
            if let Some(entry) = events_cache.get(symbol) {
                let all_gains = entry.value.to_capital_gains();
                let filtered = filter_by_range(all_gains, range);
                response.capital_gains.insert(symbol.to_string(), filtered);
            } else {
                response
                    .errors
                    .insert(symbol.to_string(), "No events data available".to_string());
            }
        }

        Ok(response)
    }

    /// Aggregate upcoming financial events across all symbols into a single
    /// time-sorted list.
    ///
    /// Merges earnings, dividend, and standard-monthly options-expiration events
    /// for every symbol — plus, with the `fred` feature, major economic releases
    /// (CPI, NFP, GDP, …) — within the forward window `[now, now + range]`,
    /// sorted ascending by timestamp.
    ///
    /// Best-effort per symbol: a symbol whose quote or options fetch fails
    /// simply contributes no events rather than failing the whole call.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::{Tickers, TimeRange};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let tickers = Tickers::new(["AAPL", "MSFT", "TSLA"]).await?;
    /// let events = tickers.calendar(TimeRange::OneMonth).await?;
    /// for e in &events {
    ///     println!("{} {:?} {:?}", e.date, e.symbol, e.event);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn calendar(
        &self,
        range: TimeRange,
    ) -> Result<Vec<crate::models::calendar::CalendarEvent>> {
        let now = chrono::Utc::now().timestamp();
        let window = (now, now + range.approx_duration_secs());

        let symbol_strings: Vec<String> = self.symbols.iter().map(|s| s.to_string()).collect();
        let providers = Arc::clone(&self.providers);

        let per_symbol = symbol_strings.into_iter().map(|sym| {
            let providers = Arc::clone(&providers);
            async move {
                let quote = {
                    let sym = sym.clone();
                    providers
                        .fetch(Capability::QUOTE, move |p| {
                            let sym = sym.clone();
                            let p = p.clone();
                            async move {
                                p.as_quote()
                                    .ok_or_else(|| {
                                        p.not_supported(crate::providers::Operation::Quote)
                                    })?
                                    .fetch_quote(&sym)
                                    .await
                            }
                        })
                        .await
                };
                (sym, quote.ok().and_then(|q| q.calendar_events))
            }
        });

        let per_symbol_fut = stream::iter(per_symbol)
            .buffer_unordered(self.max_concurrency)
            .collect::<Vec<_>>();

        // Options go through `self.options`, so a chain already in the options
        // cache is reused instead of refetched. The FRED economic-release fetch
        // is independent of both, so run all of them concurrently.
        #[cfg(feature = "fred")]
        let (per_symbol_quotes, options_resp, releases) = tokio::join!(
            per_symbol_fut,
            self.options(None),
            crate::adapters::fred::release_dates()
        );
        #[cfg(not(feature = "fred"))]
        let (per_symbol_quotes, options_resp) = tokio::join!(per_symbol_fut, self.options(None));

        let options_map = options_resp.map(|r| r.options).unwrap_or_default();

        let mut events: Vec<crate::models::calendar::CalendarEvent> = per_symbol_quotes
            .into_iter()
            .flat_map(|(sym, calendar_events)| {
                crate::models::calendar::build_symbol_events(
                    &sym,
                    calendar_events.as_ref(),
                    options_map.get(&sym),
                    window,
                )
            })
            .collect();

        #[cfg(feature = "fred")]
        if let Ok(releases) = releases {
            events.extend(crate::models::calendar::build_economic_events(
                releases, window,
            ));
        }

        crate::models::calendar::sort_events(&mut events);
        Ok(events)
    }

    /// Ensures events are loaded for all symbols using chart requests.
    ///
    /// Fetches events concurrently for symbols that don't have cached events.
    /// Uses `TimeRange::Max` to get full event history (Yahoo returns all
    /// dividends/splits/capital gains regardless of chart range).
    ///
    /// Events are always stored regardless of the cache mode because they are
    /// derived data (not a TTL-bounded cache), so they persist for the lifetime
    /// of the `Tickers` instance even under [`CacheMode::Off`](crate::utils::CacheMode::Off).
    pub(super) async fn ensure_events_loaded(&self) -> Result<()> {
        if self.events_missing().await.is_empty() {
            return Ok(());
        }

        let _fetch_guard = self.events_fetch.lock().await;

        // Double-check: another task may have fetched while we waited
        let symbols_to_fetch = self.events_missing().await;
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
                            async move {
                                p.as_corporate()
                                    .ok_or_else(|| {
                                        p.not_supported(crate::providers::Operation::Events)
                                    })?
                                    .fetch_events(&sym)
                                    .await
                            }
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

    /// Symbols with no entry in the events cache (existence check, not TTL-based).
    async fn events_missing(&self) -> Vec<Arc<str>> {
        let cache = self.events_cache.read().await;
        self.symbols
            .iter()
            .filter(|sym| !cache.contains_key(*sym))
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_tickers_dividends() {
        let tickers = Tickers::new(["AAPL", "MSFT"]).await.unwrap();
        let result = tickers.dividends(TimeRange::OneYear).await.unwrap();

        assert!(result.success_count() > 0);

        // Verify dividend data structure
        if let Some(dividends) = result.dividends.get("AAPL")
            && !dividends.is_empty()
        {
            let div = &dividends[0];
            assert!(div.timestamp > 0);
            assert!(div.amount > 0.0);
        }
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_tickers_splits() {
        let tickers = Tickers::new(["NVDA", "TSLA"]).await.unwrap();
        let result = tickers.splits(TimeRange::FiveYears).await.unwrap();

        // Note: Not all symbols have splits, so we just check for successful response
        assert!(result.success_count() > 0);

        // If there are splits, verify structure
        for splits in result.splits.values() {
            for split in splits {
                assert!(split.timestamp > 0);
                assert!(split.numerator > 0.0);
                assert!(split.denominator > 0.0);
                assert!(!split.ratio.is_empty());
            }
        }
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_tickers_capital_gains() {
        let tickers = Tickers::new(["VFIAX", "VTI"]).await.unwrap();
        let result = tickers.capital_gains(TimeRange::TwoYears).await.unwrap();

        // Note: Not all symbols have capital gains distributions
        assert!(result.success_count() > 0);

        // If there are capital gains, verify structure
        for gains in result.capital_gains.values() {
            for gain in gains {
                assert!(gain.timestamp > 0);
                assert!(gain.amount >= 0.0);
            }
        }
    }
}
