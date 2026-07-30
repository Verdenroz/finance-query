use super::Tickers;
use std::sync::Arc;

impl Tickers {
    // ========================================================================
    // Dynamic Symbol Management
    // ========================================================================

    /// Add symbols to the watch list
    ///
    /// Adds new symbols to track without affecting existing cached data.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::Tickers;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut tickers = Tickers::new(["AAPL"]).await?;
    /// tickers.add_symbols(["MSFT", "GOOGL"]);
    /// assert_eq!(tickers.len(), 3);
    /// # Ok(())
    /// # }
    /// ```
    pub fn add_symbols<S, I>(&mut self, symbols: I)
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        // Use HashSet for O(n+m) deduplication instead of O(n*m) linear search
        use std::collections::HashSet;

        let existing: HashSet<&str> = self.symbols.iter().map(|s| &**s).collect();
        let to_add: Vec<Arc<str>> = symbols
            .into_iter()
            .map(Into::into)
            .filter(|s| !existing.contains(s.as_str()))
            .map(|s| s.into())
            .collect();

        self.symbols.extend(to_add);
    }

    /// Remove symbols from the watch list
    ///
    /// Removes symbols and clears their cached data to free memory.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::Tickers;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut tickers = Tickers::new(["AAPL", "MSFT", "GOOGL"]).await?;
    /// tickers.remove_symbols(["MSFT"]);
    /// assert_eq!(tickers.len(), 2);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn remove_symbols<S, I>(&mut self, symbols: I)
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        use std::collections::HashSet;
        let owned: Vec<String> = symbols.into_iter().map(Into::into).collect();
        let to_remove: HashSet<&str> = owned.iter().map(|s| s.as_str()).collect();

        // Remove from symbol list — O(1) lookup per element
        self.symbols.retain(|s| !to_remove.contains(&**s));

        // Acquire all independent write locks in parallel
        let (
            mut quote_cache,
            mut chart_cache,
            mut events_cache,
            mut financials_cache,
            mut news_cache,
            mut recommendations_cache,
            mut options_cache,
            mut spark_cache,
        ) = tokio::join!(
            self.quote_cache.write(),
            self.chart_cache.write(),
            self.events_cache.write(),
            self.financials_cache.write(),
            self.news_cache.write(),
            self.recommendations_cache.write(),
            self.options_cache.write(),
            self.spark_cache.write(),
        );

        // Simple key caches — O(1) per removal
        for symbol in &to_remove {
            let key: Arc<str> = (*symbol).into();
            quote_cache.remove(&key);
            events_cache.remove(&key);
            news_cache.remove(&key);
        }

        // Composite key caches — O(n) retain but O(1) contains check
        chart_cache.retain(|(sym, _, _), _| !to_remove.contains(&**sym));
        financials_cache.retain(|(sym, _, _), _| !to_remove.contains(&**sym));
        recommendations_cache.retain(|(sym, _), _| !to_remove.contains(&**sym));
        options_cache.retain(|(sym, _), _| !to_remove.contains(&**sym));
        spark_cache.retain(|(sym, _, _), _| !to_remove.contains(&**sym));

        // Drop all guards before cfg-gated lock
        drop((
            quote_cache,
            chart_cache,
            events_cache,
            financials_cache,
            news_cache,
            recommendations_cache,
            options_cache,
            spark_cache,
        ));

        #[cfg(feature = "indicators")]
        self.indicators_cache
            .write()
            .await
            .retain(|(sym, _, _), _| !to_remove.contains(&**sym));
    }

    /// Clear all cached data and fetch guards, forcing fresh fetches on next access.
    ///
    /// Use this when you need up-to-date data from a long-lived `Tickers` instance.
    /// Also clears fetch guard maps to prevent unbounded growth.
    pub async fn clear_cache(&self) {
        tokio::join!(
            // Data caches
            async { self.quote_cache.write().await.clear() },
            async { self.chart_cache.write().await.clear() },
            async { self.events_cache.write().await.clear() },
            async { self.financials_cache.write().await.clear() },
            async { self.news_cache.write().await.clear() },
            async { self.recommendations_cache.write().await.clear() },
            async { self.options_cache.write().await.clear() },
            async { self.spark_cache.write().await.clear() },
            async {
                #[cfg(feature = "indicators")]
                self.indicators_cache.write().await.clear();
            },
            // Fetch guard maps (prevent unbounded growth)
            async { self.charts_fetch.write().await.clear() },
            async { self.financials_fetch.write().await.clear() },
            async { self.recommendations_fetch.write().await.clear() },
            async { self.options_fetch.write().await.clear() },
            async { self.spark_fetch.write().await.clear() },
            async {
                #[cfg(feature = "indicators")]
                self.indicators_fetch.write().await.clear();
            },
        );
    }

    /// Clear only the cached quote data.
    ///
    /// The next call to `quotes()` or `quote()` will re-fetch from the API.
    pub async fn clear_quote_cache(&self) {
        self.quote_cache.write().await.clear();
    }

    /// Clear only the cached chart, spark, and events data.
    ///
    /// The next call to `charts()`, `spark()`, `dividends()`, `splits()`,
    /// or `capital_gains()` will re-fetch from the API.
    pub async fn clear_chart_cache(&self) {
        tokio::join!(
            async { self.chart_cache.write().await.clear() },
            async { self.events_cache.write().await.clear() },
            async { self.spark_cache.write().await.clear() },
            async {
                #[cfg(feature = "indicators")]
                self.indicators_cache.write().await.clear();
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tickers_add_symbols() {
        let mut tickers = Tickers::new(["AAPL"]).await.unwrap();
        assert_eq!(tickers.len(), 1);
        assert_eq!(tickers.symbols(), &["AAPL"]);

        tickers.add_symbols(["MSFT", "GOOGL"]);
        assert_eq!(tickers.len(), 3);
        assert!(tickers.symbols().contains(&"AAPL"));
        assert!(tickers.symbols().contains(&"MSFT"));
        assert!(tickers.symbols().contains(&"GOOGL"));

        // Adding duplicate shouldn't increase count
        tickers.add_symbols(["AAPL"]);
        assert_eq!(tickers.len(), 3);
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_tickers_remove_symbols() {
        let mut tickers = Tickers::new(["AAPL", "MSFT", "GOOGL"]).await.unwrap();
        assert_eq!(tickers.len(), 3);

        // Fetch some data to populate caches
        let _ = tickers.quotes().await;

        // Remove one symbol
        tickers.remove_symbols(["MSFT"]).await;
        assert_eq!(tickers.len(), 2);
        assert!(tickers.symbols().contains(&"AAPL"));
        assert!(!tickers.symbols().contains(&"MSFT"));
        assert!(tickers.symbols().contains(&"GOOGL"));

        // Verify cache was cleared
        let quotes = tickers.quotes().await.unwrap();
        assert!(!quotes.quotes.contains_key("MSFT"));
        assert_eq!(quotes.quotes.len(), 2);
    }
}
