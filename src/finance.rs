//! Non-symbol-specific Yahoo Finance operations
//!
//! This module provides functions for operations that don't require a specific stock symbol,
//! such as searching for symbols and fetching screener data.

use crate::client::{ClientConfig, YahooClient};
use crate::constants::Region;
use crate::constants::screener_types::ScreenerType;
use crate::constants::sector_types::SectorType;
use crate::error::Result;
use crate::models::industries::Industry;
use crate::models::screeners::ScreenerResults;
use crate::models::search::SearchResults;
use crate::models::sectors::Sector;
use crate::models::transcript::{Transcript, TranscriptWithMeta};

// Re-export options for convenience
pub use crate::endpoints::lookup::{LookupOptions, LookupType};
pub use crate::endpoints::search::SearchOptions;

/// Search for stock symbols and companies
///
/// # Arguments
///
/// * `query` - Search term (company name, symbol, etc.)
/// * `options` - Search configuration options
///
/// # Examples
///
/// ```no_run
/// use finance_query::{finance, SearchOptions, Region};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Simple search with defaults
/// let results = finance::search("Apple", &SearchOptions::default()).await?;
/// println!("Found {} results", results.result_count());
///
/// // Search with custom options
/// let options = SearchOptions::new()
///     .quotes_count(10)
///     .news_count(5)
///     .enable_research_reports(true)
///     .region(Region::Canada);
/// let results = finance::search("NVDA", &options).await?;
/// println!("Found {} quotes", results.quotes.len());
/// # Ok(())
/// # }
/// ```
pub async fn search(query: &str, options: &SearchOptions) -> Result<SearchResults> {
    let client = YahooClient::new(ClientConfig::default()).await?;
    client.search(query, options).await
}

/// Look up symbols by type (equity, ETF, mutual fund, index, future, currency, cryptocurrency)
///
/// Unlike search, lookup specializes in discovering tickers filtered by asset type.
/// Optionally fetches logo URLs via an additional API call.
///
/// # Arguments
///
/// * `query` - Search term (company name, symbol, etc.)
/// * `options` - Lookup configuration options
///
/// # Examples
///
/// ```no_run
/// use finance_query::{finance, LookupOptions, LookupType, Region};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Simple lookup with defaults
/// let results = finance::lookup("Apple", &LookupOptions::default()).await?;
/// println!("Found {} results", results.result_count());
///
/// // Lookup equities with logos
/// let options = LookupOptions::new()
///     .lookup_type(LookupType::Equity)
///     .count(10)
///     .include_logo(true);
/// let results = finance::lookup("NVDA", &options).await?;
/// for quote in &results.quotes {
///     println!("{}: {:?}", quote.symbol, quote.logo_url);
/// }
/// # Ok(())
/// # }
/// ```
pub async fn lookup(
    query: &str,
    options: &LookupOptions,
) -> Result<crate::models::lookup::LookupResults> {
    let client = YahooClient::new(ClientConfig::default()).await?;
    client.lookup(query, options).await
}

/// Fetch data from a predefined Yahoo Finance screener
///
/// Returns stocks/funds matching the criteria of the specified screener type.
///
/// # Arguments
///
/// * `screener_type` - The predefined screener to use
/// * `count` - Number of results to return (max 250)
///
/// # Examples
///
/// ```no_run
/// use finance_query::{finance, ScreenerType};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Get top gainers
/// let gainers = finance::screener(ScreenerType::DayGainers, 25).await?;
/// println!("Top gainers: {:#?}", gainers);
///
/// // Get most shorted stocks
/// let shorted = finance::screener(ScreenerType::MostShortedStocks, 25).await?;
///
/// // Get growth technology stocks
/// let tech = finance::screener(ScreenerType::GrowthTechnologyStocks, 25).await?;
/// # Ok(())
/// # }
/// ```
pub async fn screener(screener_type: ScreenerType, count: u32) -> Result<ScreenerResults> {
    let client = YahooClient::new(ClientConfig::default()).await?;
    client.get_screener(screener_type, count).await
}

/// Execute a custom screener query
///
/// Allows flexible filtering of stocks/funds/ETFs based on various criteria.
///
/// # Arguments
///
/// * `query` - The custom screener query to execute
///
/// # Examples
///
/// ```no_run
/// use finance_query::{finance, ScreenerQuery, QueryCondition, screener_query::Operator};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Find US stocks with high volume sorted by market cap
/// let query = ScreenerQuery::new()
///     .size(25)
///     .sort_by("intradaymarketcap", false)
///     .add_condition(QueryCondition::new("region", Operator::Eq).value_str("us"))
///     .add_condition(QueryCondition::new("avgdailyvol3m", Operator::Gt).value(200000));
///
/// let result = finance::custom_screener(query).await?;
/// println!("Found {} stocks", result.quotes.len());
/// # Ok(())
/// # }
/// ```
pub async fn custom_screener(
    query: crate::models::screeners::ScreenerQuery,
) -> Result<ScreenerResults> {
    let client = YahooClient::new(ClientConfig::default()).await?;
    client.custom_screener(query).await
}

/// Get general market news
///
/// # Examples
///
/// ```no_run
/// use finance_query::finance;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let news = finance::news().await?;
/// for article in news {
///     println!("{}: {}", article.source, article.title);
/// }
/// # Ok(())
/// # }
/// ```
pub async fn news() -> Result<Vec<crate::models::news::News>> {
    crate::scrapers::stockanalysis::scrape_general_news().await
}

/// Get earnings transcript for a symbol
///
/// Fetches the earnings call transcript, handling all the complexity internally:
/// 1. Gets the company ID (quartrId) from the quote_type endpoint
/// 2. Scrapes available earnings calls
/// 3. Fetches the requested transcript
///
/// # Arguments
///
/// * `symbol` - Stock symbol (e.g., "AAPL", "MSFT")
/// * `quarter` - Optional fiscal quarter (Q1, Q2, Q3, Q4). If None, gets latest.
/// * `year` - Optional fiscal year. If None, gets latest.
///
/// # Examples
///
/// ```no_run
/// use finance_query::finance;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Get the latest transcript
/// let latest = finance::earnings_transcript("AAPL", None, None).await?;
/// println!("Quarter: {} {}", latest.quarter(), latest.year());
///
/// // Get a specific quarter
/// let q4_2024 = finance::earnings_transcript("AAPL", Some("Q4"), Some(2024)).await?;
/// # Ok(())
/// # }
/// ```
pub async fn earnings_transcript(
    symbol: &str,
    quarter: Option<&str>,
    year: Option<i32>,
) -> Result<Transcript> {
    let client = YahooClient::new(ClientConfig::default()).await?;
    crate::endpoints::transcripts::fetch_for_symbol(&client, symbol, quarter, year).await
}

/// Get all earnings transcripts for a symbol
///
/// Fetches transcripts for all available earnings calls.
///
/// # Arguments
///
/// * `symbol` - Stock symbol (e.g., "AAPL", "MSFT")
/// * `limit` - Optional maximum number of transcripts. If None, fetches all.
///
/// # Examples
///
/// ```no_run
/// use finance_query::finance;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Get all transcripts
/// let all = finance::earnings_transcripts("AAPL", None).await?;
///
/// // Get only the 5 most recent
/// let recent = finance::earnings_transcripts("AAPL", Some(5)).await?;
/// for t in &recent {
///     println!("{}: {} {}", t.title, t.transcript.quarter(), t.transcript.year());
/// }
/// # Ok(())
/// # }
/// ```
pub async fn earnings_transcripts(
    symbol: &str,
    limit: Option<usize>,
) -> Result<Vec<TranscriptWithMeta>> {
    let client = YahooClient::new(ClientConfig::default()).await?;
    crate::endpoints::transcripts::fetch_all_for_symbol(&client, symbol, limit).await
}

/// Get market hours/status
///
/// Returns the current status for various markets.
///
/// # Arguments
///
/// * `region` - Optional region override (e.g., "US", "JP", "GB"). If None, uses default (US).
///
/// # Examples
///
/// ```no_run
/// use finance_query::finance;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Get US market hours (default)
/// let hours = finance::hours(None).await?;
///
/// // Get Japan market hours
/// let jp_hours = finance::hours(Some("JP")).await?;
/// # Ok(())
/// # }
/// ```
pub async fn hours(region: Option<&str>) -> Result<crate::models::hours::MarketHours> {
    let client = YahooClient::new(ClientConfig::default()).await?;
    client.get_hours(region).await
}

/// Get world market indices quotes
///
/// Returns quotes for major world indices, optionally filtered by region.
///
/// # Arguments
///
/// * `region` - Optional region filter. If None, returns all world indices.
///
/// # Examples
///
/// ```no_run
/// use finance_query::{finance, IndicesRegion};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Get all world indices
/// let all = finance::indices(None).await?;
/// println!("Fetched {} indices", all.success_count());
///
/// // Get only Americas indices
/// let americas = finance::indices(Some(IndicesRegion::Americas)).await?;
/// # Ok(())
/// # }
/// ```
pub async fn indices(
    region: Option<crate::constants::indices::Region>,
) -> Result<crate::tickers::BatchQuotesResponse> {
    use crate::Tickers;
    use crate::constants::indices::all_symbols;

    let symbols: Vec<&str> = match region {
        Some(r) => r.symbols().to_vec(),
        None => all_symbols(),
    };

    let tickers = Tickers::new(symbols).await?;
    tickers.quotes(false).await
}

/// Fetch detailed sector data from Yahoo Finance
///
/// Returns comprehensive sector information including overview, performance,
/// top companies, ETFs, mutual funds, industries, and research reports.
///
/// # Arguments
///
/// * `sector_type` - The sector to fetch data for
///
/// # Examples
///
/// ```no_run
/// use finance_query::{finance, SectorType};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let sector = finance::sector(SectorType::Technology).await?;
/// println!("Sector: {} ({} companies)", sector.name,
///     sector.overview.as_ref().map(|o| o.companies_count.unwrap_or(0)).unwrap_or(0));
///
/// for company in sector.top_companies.iter().take(5) {
///     println!("  {} - {:?}", company.symbol, company.name);
/// }
/// # Ok(())
/// # }
/// ```
pub async fn sector(sector_type: SectorType) -> Result<Sector> {
    let client = YahooClient::new(ClientConfig::default()).await?;
    client.get_sector(sector_type).await
}

/// Fetch detailed industry data from Yahoo Finance
///
/// Returns comprehensive industry information including overview, performance,
/// top companies, top performing companies, top growth companies, and research reports.
///
/// # Arguments
///
/// * `industry_key` - The industry key/slug (e.g., "semiconductors", "software-infrastructure")
///
/// # Examples
///
/// ```no_run
/// use finance_query::finance;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let industry = finance::industry("semiconductors").await?;
/// println!("Industry: {} ({} companies)", industry.name,
///     industry.overview.as_ref().map(|o| o.companies_count.unwrap_or(0)).unwrap_or(0));
///
/// for company in industry.top_companies.iter().take(5) {
///     println!("  {} - {:?}", company.symbol, company.name);
/// }
/// # Ok(())
/// # }
/// ```
pub async fn industry(industry_key: &str) -> Result<Industry> {
    let client = YahooClient::new(ClientConfig::default()).await?;
    client.get_industry(industry_key).await
}

/// Get list of available currencies
///
/// Returns currency information from Yahoo Finance.
///
/// # Examples
///
/// ```no_run
/// use finance_query::finance;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let currencies = finance::currencies().await?;
/// # Ok(())
/// # }
/// ```
pub async fn currencies() -> Result<Vec<crate::models::currencies::Currency>> {
    let client = YahooClient::new(ClientConfig::default()).await?;
    client.get_currencies().await
}

/// Get list of supported exchanges
///
/// Scrapes the Yahoo Finance help page for a list of supported exchanges
/// with their symbol suffixes and data delay information.
///
/// # Examples
///
/// ```no_run
/// use finance_query::finance;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let exchanges = finance::exchanges().await?;
/// for exchange in &exchanges {
///     println!("{} - {} ({})", exchange.country, exchange.market, exchange.suffix);
/// }
/// # Ok(())
/// # }
/// ```
pub async fn exchanges() -> Result<Vec<crate::models::exchanges::Exchange>> {
    crate::scrapers::yahoo_exchanges::scrape_exchanges().await
}

/// Get market summary
///
/// Returns market summary with major indices, currencies, and commodities.
///
/// # Arguments
///
/// * `region` - Optional region for localization. If None, uses default (US).
///
/// # Examples
///
/// ```no_run
/// use finance_query::{finance, Region};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Use default (US)
/// let summary = finance::market_summary(None).await?;
/// // Or specify a region
/// let summary = finance::market_summary(Some(Region::Canada)).await?;
/// # Ok(())
/// # }
/// ```
pub async fn market_summary(
    region: Option<Region>,
) -> Result<Vec<crate::models::market_summary::MarketSummaryQuote>> {
    let client = YahooClient::new(ClientConfig::default()).await?;
    client.get_market_summary(region).await
}

/// Get trending tickers for a region
///
/// Returns trending stocks for a specific region.
///
/// # Arguments
///
/// * `region` - Optional region for localization. If None, uses default (US).
///
/// # Examples
///
/// ```no_run
/// use finance_query::{finance, Region};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Use default (US)
/// let trending = finance::trending(None).await?;
/// // Or specify a region
/// let trending = finance::trending(Some(Region::Canada)).await?;
/// # Ok(())
/// # }
/// ```
pub async fn trending(
    region: Option<Region>,
) -> Result<Vec<crate::models::trending::TrendingQuote>> {
    let client = YahooClient::new(ClientConfig::default()).await?;
    client.get_trending(region).await
}

// ============================================================================
// SEC EDGAR API
// ============================================================================

/// Look up a CIK number for a ticker symbol via SEC EDGAR.
///
/// The CIK (Central Index Key) is the unique identifier used by the SEC
/// to identify companies. The mapping is cached on first call.
///
/// # Arguments
///
/// * `edgar` - An EDGAR client instance (constructed via [`crate::EdgarClientBuilder`])
/// * `symbol` - Ticker symbol (e.g., "AAPL", "MSFT")
///
/// # Examples
///
/// ```no_run
/// use finance_query::{finance, EdgarClientBuilder};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let edgar = EdgarClientBuilder::new("user@example.com").build()?;
/// let cik = finance::edgar_cik(&edgar, "AAPL").await?;
/// println!("AAPL CIK: {}", cik);
/// # Ok(())
/// # }
/// ```
pub async fn edgar_cik(edgar: &crate::edgar::EdgarClient, symbol: &str) -> Result<u64> {
    edgar.resolve_cik(symbol).await
}

/// Fetch SEC EDGAR filing history (submissions) for a symbol.
///
/// Returns company metadata and the most recent ~1000 filings, with references
/// to additional history files for older filings.
///
/// # Arguments
///
/// * `edgar` - An EDGAR client instance
/// * `symbol` - Ticker symbol (e.g., "AAPL")
///
/// # Examples
///
/// ```no_run
/// use finance_query::{finance, EdgarClientBuilder};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let edgar = EdgarClientBuilder::new("user@example.com").build()?;
/// let submissions = finance::edgar_submissions(&edgar, "AAPL").await?;
/// println!("Company: {:?}", submissions.name);
///
/// if let Some(filings) = &submissions.filings {
///     if let Some(recent) = &filings.recent {
///         for filing in recent.to_filings().iter().take(5) {
///             println!("{}: {} ({})", filing.filing_date, filing.form, filing.primary_doc_description);
///         }
///     }
/// }
/// # Ok(())
/// # }
/// ```
pub async fn edgar_submissions(
    edgar: &crate::edgar::EdgarClient,
    symbol: &str,
) -> Result<crate::models::edgar::EdgarSubmissions> {
    let cik = edgar.resolve_cik(symbol).await?;
    edgar.submissions(cik).await
}

/// Fetch SEC EDGAR company facts (structured XBRL financial data) for a symbol.
///
/// Returns all extracted XBRL facts organized by taxonomy (us-gaap, ifrs, dei).
/// This includes historical financial statement values like revenue, assets,
/// liabilities, EPS, etc.
///
/// # Arguments
///
/// * `edgar` - An EDGAR client instance
/// * `symbol` - Ticker symbol (e.g., "AAPL")
///
/// # Examples
///
/// ```no_run
/// use finance_query::{finance, EdgarClientBuilder};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let edgar = EdgarClientBuilder::new("user@example.com").build()?;
/// let facts = finance::edgar_company_facts(&edgar, "AAPL").await?;
///
/// if let Some(revenue) = facts.get_us_gaap_fact("Revenue") {
///     if let Some(usd) = revenue.units.get("USD") {
///         for point in usd.iter().take(3) {
///             println!("FY{:?} {:?}: ${:?}", point.fy, point.fp, point.val);
///         }
///     }
/// }
/// # Ok(())
/// # }
/// ```
pub async fn edgar_company_facts(
    edgar: &crate::edgar::EdgarClient,
    symbol: &str,
) -> Result<crate::models::edgar::CompanyFacts> {
    let cik = edgar.resolve_cik(symbol).await?;
    edgar.company_facts(cik).await
}

/// Search SEC EDGAR filings by text content.
///
/// Searches across all EDGAR filing text since 2001 using the
/// full-text search (EFTS) API.
///
/// # Arguments
///
/// * `edgar` - An EDGAR client instance
/// * `query` - Search term or phrase
/// * `forms` - Optional form type filter (e.g., `&["10-K", "10-Q"]`)
/// * `start_date` - Optional start date filter (YYYY-MM-DD)
/// * `end_date` - Optional end date filter (YYYY-MM-DD)
///
/// # Examples
///
/// ```no_run
/// use finance_query::{finance, EdgarClientBuilder};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let edgar = EdgarClientBuilder::new("user@example.com").build()?;
/// let results = finance::edgar_search(
///     &edgar,
///     "artificial intelligence",
///     Some(&["10-K"]),
///     Some("2024-01-01"),
///     None,
///     None,
///     None,
/// ).await?;
///
/// if let Some(hits_container) = &results.hits {
///     for hit in &hits_container.hits {
///         if let Some(source) = &hit._source {
///             println!("{}: {:?} ({:?})",
///                 source.form.as_deref().unwrap_or("?"),
///                 source.display_names,
///                 source.file_date,
///             );
///         }
///     }
/// }
/// # Ok(())
/// # }
/// ```
pub async fn edgar_search(
    edgar: &crate::edgar::EdgarClient,
    query: &str,
    forms: Option<&[&str]>,
    start_date: Option<&str>,
    end_date: Option<&str>,
    from: Option<usize>,
    size: Option<usize>,
) -> Result<crate::models::edgar::EdgarSearchResults> {
    edgar
        .search(query, forms, start_date, end_date, from, size)
        .await
}
