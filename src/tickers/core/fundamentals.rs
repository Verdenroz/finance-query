use super::{
    BatchFinancialsResponse, BatchNewsResponse, BatchOptionsResponse, BatchRecommendationsResponse,
    Tickers,
};
use crate::constants::{Frequency, StatementType};
use crate::error::Result;
use crate::models::corporate::news::News;
use crate::providers::Capability;
use crate::providers::types::recommendation_from_similar;
use crate::tickers::macros::batch_fetch_cached;
use futures::stream::StreamExt;

impl Tickers {
    /// Batch fetch financial statements for all symbols
    ///
    /// Fetches the specified financial statement type for all symbols concurrently.
    /// Financial statements are cached per (symbol, statement_type, frequency) tuple.
    ///
    /// # Arguments
    ///
    /// * `statement_type` - Type of statement (Income, Balance, CashFlow)
    /// * `frequency` - Annual or Quarterly
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::{Tickers, StatementType, Frequency};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let tickers = Tickers::new(["AAPL", "MSFT", "GOOGL"]).await?;
    /// let financials = tickers.financials(StatementType::Income, Frequency::Annual).await?;
    ///
    /// for (symbol, stmt) in &financials.financials {
    ///     if let Some(revenue) = stmt.statement.get("TotalRevenue") {
    ///         println!("{}: {:?}", symbol, revenue);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn financials(
        &self,
        statement_type: StatementType,
        frequency: Frequency,
    ) -> Result<BatchFinancialsResponse> {
        batch_fetch_cached!(self;
            cache: financials_cache,
            guard: map(financials_fetch, (statement_type, frequency)),
            key: |s| (s.clone(), statement_type, frequency),
            response: BatchFinancialsResponse.financials,
            fetch: |providers, symbol| {
                let sym = symbol.to_string();
                providers.fetch(Capability::FUNDAMENTALS, move |p| {
                    let sym = sym.clone();
                    let p = p.clone();
                    async move {
                        p.fetch_financials(&sym, statement_type, frequency)
                            .await
                    }
                }).await
            },
        )
    }

    /// Batch fetch news articles for all symbols
    ///
    /// Fetches recent news articles for all symbols concurrently using scrapers.
    /// News articles are cached per symbol.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::Tickers;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let tickers = Tickers::new(["AAPL", "MSFT"]).await?;
    /// let news = tickers.news().await?;
    ///
    /// for (symbol, articles) in &news.news {
    ///     println!("{}: {} articles", symbol, articles.len());
    ///     for article in articles.iter().take(3) {
    ///         println!("  - {}", article.title);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn news(&self) -> Result<BatchNewsResponse> {
        batch_fetch_cached!(self;
            cache: news_cache,
            guard: simple(news_fetch),
            key: |s| s.clone(),
            response: BatchNewsResponse.news,
            fetch: |providers, symbol| {
                let sym = symbol.to_string();
                providers.fetch(Capability::CORPORATE, move |p| {
                    let sym = sym.clone();
                    let p = p.clone();
                    async move {
                        p.fetch_news(&sym)
                            .await
                            .map(|data| data.into_iter().collect::<Vec<News>>())
                    }
                }).await
            },
        )
    }

    /// Batch fetch recommendations for all symbols
    ///
    /// Fetches analyst recommendations and similar stocks for all symbols concurrently.
    /// Recommendations are cached per (symbol, limit) tuple — different limits
    /// produce different API responses and are cached independently.
    ///
    /// # Arguments
    ///
    /// * `limit` - Maximum number of similar stocks to return per symbol
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::Tickers;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let tickers = Tickers::new(["AAPL", "MSFT"]).await?;
    /// let recommendations = tickers.recommendations(10).await?;
    ///
    /// for (symbol, rec) in &recommendations.recommendations {
    ///     println!("{}: {} recommendations", symbol, rec.count());
    ///     for similar in &rec.recommendations {
    ///         println!("  - {}: score {}", similar.symbol, similar.score);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn recommendations(&self, limit: u32) -> Result<BatchRecommendationsResponse> {
        batch_fetch_cached!(self;
            cache: recommendations_cache,
            guard: map(recommendations_fetch, limit),
            key: |s| (s.clone(), limit),
            response: BatchRecommendationsResponse.recommendations,
            fetch: |providers, symbol| {
                let sym = symbol.to_string();
                providers.fetch(Capability::CORPORATE, move |p| {
                    let sym = sym.clone();
                    let p = p.clone();
                    async move {
                        let items = p.fetch_similar_symbols(&sym, limit).await?;
                        Ok(recommendation_from_similar(
                            sym,
                            Some(p.id()),
                            items,
                            Some(limit),
                        ))
                    }
                }).await
            },
        )
    }

    /// Batch fetch options chains for all symbols
    ///
    /// Fetches options chains for the specified expiration date for all symbols concurrently.
    /// Options are cached per (symbol, date) tuple.
    ///
    /// # Arguments
    ///
    /// * `date` - Optional expiration date (Unix timestamp). If None, fetches nearest expiration.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::Tickers;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let tickers = Tickers::new(["AAPL", "MSFT"]).await?;
    /// let options = tickers.options(None).await?;
    ///
    /// for (symbol, opts) in &options.options {
    ///     println!("{}: {} expirations", symbol, opts.expiration_dates().len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn options(&self, date: Option<i64>) -> Result<BatchOptionsResponse> {
        batch_fetch_cached!(self;
            cache: options_cache,
            guard: map(options_fetch, date),
            key: |s| (s.clone(), date),
            response: BatchOptionsResponse.options,
            fetch: |providers, symbol| {
                let sym = symbol.to_string();
                providers.fetch(Capability::OPTIONS, move |p| {
                    let sym = sym.clone();
                    let p = p.clone();
                    async move {
                        p.fetch_options(&sym, date).await
                    }
                }).await
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_tickers_financials() {
        let tickers = Tickers::new(["AAPL", "MSFT"]).await.unwrap();
        let result = tickers
            .financials(StatementType::Income, Frequency::Annual)
            .await
            .unwrap();

        assert!(result.success_count() > 0);

        // Verify financial statement structure
        for (symbol, stmt) in &result.financials {
            assert_eq!(stmt.symbol, *symbol);
            assert_eq!(stmt.statement_type, "income");
            assert_eq!(stmt.frequency, "annual");
            assert!(!stmt.statement.is_empty());

            // Common income statement fields
            if let Some(revenue) = stmt.statement.get("TotalRevenue") {
                assert!(!revenue.is_empty());
            }
        }
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_tickers_news() {
        let tickers = Tickers::new(["AAPL", "TSLA"]).await.unwrap();
        let result = tickers.news().await.unwrap();

        assert!(result.success_count() > 0);

        // Verify news structure
        for articles in result.news.values() {
            if !articles.is_empty() {
                let article = &articles[0];
                assert!(!article.title.is_empty());
                assert!(!article.link.is_empty());
                assert!(!article.source.is_empty());
            }
        }
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_tickers_recommendations() {
        let tickers = Tickers::new(["AAPL", "MSFT"]).await.unwrap();
        let result = tickers.recommendations(5).await.unwrap();

        assert!(result.success_count() > 0);

        // Verify recommendations structure
        for (symbol, rec) in &result.recommendations {
            assert_eq!(rec.symbol, *symbol);
            assert!(rec.count() > 0);
            for similar in &rec.recommendations {
                assert!(!similar.symbol.is_empty());
            }
        }
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_tickers_options() {
        let tickers = Tickers::new(["AAPL", "MSFT"]).await.unwrap();
        let result = tickers.options(None).await.unwrap();

        assert!(result.success_count() > 0);

        // Verify options structure
        for opts in result.options.values() {
            assert!(!opts.expiration_dates().is_empty());
        }
    }
}
