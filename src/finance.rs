//! Non-symbol-specific Yahoo Finance operations
//!
//! This module provides functions for operations that don't require a specific stock symbol,
//! such as searching for symbols and fetching screener data.

use crate::adapters::yahoo::client::{ClientConfig, YahooClient};
use crate::constants::Region;
use crate::constants::screeners::Screener;
use crate::constants::sectors::Sector;
use crate::error::Result;
use crate::models::corporate::transcript::{Transcript, TranscriptWithMeta};
use crate::models::discovery::screeners::ScreenerResults;
use crate::models::discovery::search::SearchResults;
use crate::models::market::industries::IndustryData;
use crate::models::market::sectors::SectorData;

#[cfg(any(feature = "fmp", feature = "alphavantage"))]
use serde::{Deserialize, Serialize};

// Re-export options for convenience
pub use crate::adapters::yahoo::discovery::lookup::{LookupOptions, LookupType};
pub use crate::adapters::yahoo::discovery::search::SearchOptions;

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
) -> Result<crate::models::discovery::lookup::LookupResults> {
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
/// use finance_query::{finance, Screener};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Get top gainers
/// let gainers = finance::screener(Screener::DayGainers, 25).await?;
/// println!("Top gainers: {:#?}", gainers);
///
/// // Get most shorted stocks
/// let shorted = finance::screener(Screener::MostShortedStocks, 25).await?;
///
/// // Get growth technology stocks
/// let tech = finance::screener(Screener::GrowthTechnologyStocks, 25).await?;
/// # Ok(())
/// # }
/// ```
pub async fn screener(screener_type: Screener, count: u32) -> Result<ScreenerResults> {
    let client = YahooClient::new(ClientConfig::default()).await?;
    crate::adapters::yahoo::discovery::screeners::fetch(&client, screener_type, count).await
}

/// Execute a custom screener query
///
/// Allows flexible filtering of stocks/funds/ETFs based on various criteria.
/// Use [`EquityScreenerQuery`][crate::EquityScreenerQuery] for stock screeners
/// or [`FundScreenerQuery`][crate::FundScreenerQuery] for mutual fund screeners.
///
/// # Arguments
///
/// * `query` - The custom screener query to execute
///
/// # Examples
///
/// ```no_run
/// use finance_query::{finance, EquityField, EquityScreenerQuery, ScreenerFieldExt};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Find US large-cap stocks with high volume
/// let query = EquityScreenerQuery::new()
///     .size(25)
///     .sort_by(EquityField::IntradayMarketCap, false)
///     .add_condition(EquityField::Region.eq_str("us"))
///     .add_condition(EquityField::AvgDailyVol3M.gt(200_000.0))
///     .add_condition(EquityField::IntradayMarketCap.gt(10_000_000_000.0));
///
/// let result = finance::custom_screener(query).await?;
/// println!("Found {} stocks", result.quotes.len());
/// # Ok(())
/// # }
/// ```
pub async fn custom_screener<F: crate::models::discovery::screeners::ScreenerField>(
    query: crate::models::discovery::screeners::ScreenerQuery<F>,
) -> Result<ScreenerResults> {
    let client = YahooClient::new(ClientConfig::default()).await?;
    crate::adapters::yahoo::discovery::screeners::fetch_custom(&client, query).await
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
pub async fn news() -> Result<Vec<crate::models::corporate::news::News>> {
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
    crate::adapters::yahoo::corporate::transcripts::fetch_for_symbol(&client, symbol, quarter, year)
        .await
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
    crate::adapters::yahoo::corporate::transcripts::fetch_all_for_symbol(&client, symbol, limit)
        .await
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
pub async fn hours(region: Option<&str>) -> Result<crate::models::market::hours::MarketHours> {
    let client = YahooClient::new(ClientConfig::default()).await?;
    crate::adapters::yahoo::market::hours::fetch(&client, region).await
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
    tickers.quotes().await
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
/// use finance_query::{finance, Sector};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let sector = finance::sector(Sector::Technology).await?;
/// println!("Sector: {} ({} companies)", sector.name,
///     sector.overview.as_ref().map(|o| o.companies_count.unwrap_or(0)).unwrap_or(0));
///
/// for company in sector.top_companies.iter().take(5) {
///     println!("  {} - {:?}", company.symbol, company.name);
/// }
/// # Ok(())
/// # }
/// ```
pub async fn sector(sector_type: Sector) -> Result<SectorData> {
    let client = YahooClient::new(ClientConfig::default()).await?;
    crate::adapters::yahoo::market::sectors::fetch(&client, sector_type).await
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
pub async fn industry(industry_key: impl AsRef<str>) -> Result<IndustryData> {
    let client = YahooClient::new(ClientConfig::default()).await?;
    crate::adapters::yahoo::market::industries::fetch(&client, industry_key.as_ref()).await
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
pub async fn currencies() -> Result<Vec<crate::models::market::currencies::Currency>> {
    let client = YahooClient::new(ClientConfig::default()).await?;
    crate::adapters::yahoo::market::currencies::fetch(&client).await
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
pub async fn exchanges() -> Result<Vec<crate::models::market::exchanges::Exchange>> {
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
) -> Result<Vec<crate::models::market::market_summary::MarketSummaryQuote>> {
    let client = YahooClient::new(ClientConfig::default()).await?;
    crate::adapters::yahoo::market::market_summary::fetch(&client, region).await
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
) -> Result<Vec<crate::models::discovery::trending::TrendingQuote>> {
    let client = YahooClient::new(ClientConfig::default()).await?;
    crate::adapters::yahoo::market::trending::fetch(&client, region).await
}

/// Fetch the current CNN Fear & Greed Index from Alternative.me.
///
/// Returns a 0–100 sentiment score and its classification. No API key required.
///
/// # Examples
///
/// ```no_run
/// use finance_query::finance;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let fg = finance::fear_and_greed().await?;
/// println!("Fear & Greed: {} ({})", fg.value, fg.classification.as_str());
/// # Ok(())
/// # }
/// ```
pub async fn fear_and_greed() -> Result<crate::models::sentiment::FearAndGreed> {
    crate::adapters::yahoo::market::fear_and_greed::fetch().await
}

// ── Financial Modeling Prep (FMP) ───────────────────────────────────

/// Time period for analyst estimates.
#[cfg(feature = "fmp")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Period {
    /// Annual (yearly) estimates.
    Annual,
    /// Quarterly estimates.
    Quarter,
}

#[cfg(feature = "fmp")]
impl From<Period> for crate::adapters::fmp::models::Period {
    fn from(p: Period) -> Self {
        match p {
            Period::Annual => Self::Annual,
            Period::Quarter => Self::Quarter,
        }
    }
}

/// An insider trading transaction record.
#[cfg(feature = "fmp")]
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsiderTransaction {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Filing date (YYYY-MM-DD).
    pub filing_date: Option<String>,
    /// Transaction date (YYYY-MM-DD).
    pub transaction_date: Option<String>,
    /// Reporting person name.
    pub reporting_name: Option<String>,
    /// Transaction type (e.g., "P-Purchase", "S-Sale").
    pub transaction_type: Option<String>,
    /// Number of securities transacted.
    pub securities_transacted: Option<f64>,
    /// Price per share.
    pub price: Option<f64>,
    /// Securities owned after transaction.
    pub securities_owned: Option<f64>,
    /// Form type / owner type description.
    pub type_of_owner: Option<String>,
    /// Link to SEC filing.
    pub link: Option<String>,
}

#[cfg(feature = "fmp")]
impl From<crate::adapters::fmp::corporate::insider_trading::InsiderTradeDTO>
    for InsiderTransaction
{
    fn from(d: crate::adapters::fmp::corporate::insider_trading::InsiderTradeDTO) -> Self {
        use crate::adapters::fmp::corporate::insider_trading::InsiderTradeDTO;
        let InsiderTradeDTO {
            symbol,
            filing_date,
            transaction_date,
            reporting_name,
            transaction_type,
            securities_transacted,
            price,
            securities_owned,
            type_of_owner,
            link,
            ..
        } = d;
        Self {
            symbol,
            filing_date,
            transaction_date,
            reporting_name,
            transaction_type,
            securities_transacted,
            price,
            securities_owned,
            type_of_owner,
            link,
        }
    }
}

/// An analyst estimate entry (revenue, EBITDA, EPS forecasts).
#[cfg(feature = "fmp")]
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalystEstimate {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Estimate date.
    pub date: Option<String>,
    /// Estimated revenue low.
    pub estimated_revenue_low: Option<f64>,
    /// Estimated revenue high.
    pub estimated_revenue_high: Option<f64>,
    /// Estimated revenue avg.
    pub estimated_revenue_avg: Option<f64>,
    /// Estimated EBITDA low.
    pub estimated_ebitda_low: Option<f64>,
    /// Estimated EBITDA high.
    pub estimated_ebitda_high: Option<f64>,
    /// Estimated EBITDA avg.
    pub estimated_ebitda_avg: Option<f64>,
    /// Estimated EPS avg.
    pub estimated_eps_avg: Option<f64>,
    /// Estimated EPS high.
    pub estimated_eps_high: Option<f64>,
    /// Estimated EPS low.
    pub estimated_eps_low: Option<f64>,
    /// Number of analysts covering revenue.
    pub number_analyst_estimated_revenue: Option<i32>,
    /// Number of analysts covering EPS.
    pub number_analysts_estimated_eps: Option<i32>,
}

#[cfg(feature = "fmp")]
impl From<crate::adapters::fmp::fundamentals::estimates::AnalystEstimateDTO> for AnalystEstimate {
    fn from(d: crate::adapters::fmp::fundamentals::estimates::AnalystEstimateDTO) -> Self {
        use crate::adapters::fmp::fundamentals::estimates::AnalystEstimateDTO;
        let AnalystEstimateDTO {
            symbol,
            date,
            estimated_revenue_low,
            estimated_revenue_high,
            estimated_revenue_avg,
            estimated_ebitda_low,
            estimated_ebitda_high,
            estimated_ebitda_avg,
            estimated_eps_avg,
            estimated_eps_high,
            estimated_eps_low,
            number_analyst_estimated_revenue,
            number_analysts_estimated_eps,
        } = d;
        Self {
            symbol,
            date,
            estimated_revenue_low,
            estimated_revenue_high,
            estimated_revenue_avg,
            estimated_ebitda_low,
            estimated_ebitda_high,
            estimated_ebitda_avg,
            estimated_eps_avg,
            estimated_eps_high,
            estimated_eps_low,
            number_analyst_estimated_revenue,
            number_analysts_estimated_eps,
        }
    }
}

/// An analyst stock recommendation (buy/hold/sell counts).
#[cfg(feature = "fmp")]
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalystRecommendation {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Recommendation date.
    pub date: Option<String>,
    /// Number of buy ratings.
    pub analyst_ratings_buy: Option<i32>,
    /// Number of hold ratings.
    pub analyst_ratings_hold: Option<i32>,
    /// Number of sell ratings.
    pub analyst_ratings_sell: Option<i32>,
    /// Number of strong buy ratings.
    pub analyst_ratings_strong_buy: Option<i32>,
    /// Number of strong sell ratings.
    pub analyst_ratings_strong_sell: Option<i32>,
}

#[cfg(feature = "fmp")]
impl From<crate::adapters::fmp::fundamentals::estimates::AnalystRecommendationDTO>
    for AnalystRecommendation
{
    fn from(d: crate::adapters::fmp::fundamentals::estimates::AnalystRecommendationDTO) -> Self {
        use crate::adapters::fmp::fundamentals::estimates::AnalystRecommendationDTO;
        let AnalystRecommendationDTO {
            symbol,
            date,
            analyst_ratings_buy,
            analyst_ratings_hold,
            analyst_ratings_sell,
            analyst_ratings_strong_buy,
            analyst_ratings_strong_sell,
        } = d;
        Self {
            symbol,
            date,
            analyst_ratings_buy,
            analyst_ratings_hold,
            analyst_ratings_sell,
            analyst_ratings_strong_buy,
            analyst_ratings_strong_sell,
        }
    }
}

/// Fetch insider trading transactions for a symbol.
#[cfg(feature = "fmp")]
pub async fn insider_trading(symbol: &str, limit: u32) -> Result<Vec<InsiderTransaction>> {
    crate::adapters::fmp::corporate::insider_trading::insider_trading(symbol, limit)
        .await
        .map(|v| v.into_iter().map(Into::into).collect())
}

/// Fetch analyst estimates for a symbol.
#[cfg(feature = "fmp")]
pub async fn analyst_estimates(symbol: &str, period: Period) -> Result<Vec<AnalystEstimate>> {
    crate::adapters::fmp::fundamentals::estimates::analyst_estimates(symbol, period.into(), 4)
        .await
        .map(|v| v.into_iter().map(Into::into).collect())
}

/// Fetch analyst stock recommendations for a symbol.
#[cfg(feature = "fmp")]
pub async fn analyst_recommendations(symbol: &str) -> Result<Vec<AnalystRecommendation>> {
    crate::adapters::fmp::fundamentals::estimates::analyst_recommendations(symbol)
        .await
        .map(|v| v.into_iter().map(Into::into).collect())
}

// ── Polygon.io ──────────────────────────────────────────────────────

/// Fetch sentiment analysis for a symbol based on recent Polygon.io news.
#[cfg(feature = "polygon")]
pub async fn symbol_sentiment(symbol: &str) -> Result<crate::models::sentiment::SymbolSentiment> {
    use crate::adapters::polygon;
    let paginated = polygon::stock_news(&[("ticker", symbol), ("limit", "10")]).await?;
    let articles = paginated.results.unwrap_or_default();

    let mut positive = 0u32;
    let mut negative = 0u32;
    let total = articles.len().max(1) as f64;
    for article in &articles {
        if let Some(ref insights) = article.insights {
            for insight in insights {
                if insight.ticker.as_deref() == Some(symbol) {
                    match insight.sentiment.as_deref() {
                        Some("positive") => positive += 1,
                        Some("negative") => negative += 1,
                        _ => {}
                    }
                }
            }
        }
    }

    let (score, label): (Option<f64>, Option<String>) = if total > 0.0 {
        let s = (positive as f64 - negative as f64) / total;
        let l = if s > 0.2 {
            "positive"
        } else if s < -0.2 {
            "negative"
        } else {
            "neutral"
        };
        (Some(s), Some(l.to_string()))
    } else {
        (None, None)
    };

    Ok(crate::models::sentiment::SymbolSentiment { score, label })
}

// ── Alpha Vantage ───────────────────────────────────────────────────

/// An upcoming earnings calendar entry.
#[cfg(feature = "alphavantage")]
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EarningsCalendarEntry {
    /// Ticker symbol.
    pub symbol: String,
    /// Company name.
    pub name: Option<String>,
    /// Report date.
    pub report_date: Option<String>,
    /// Fiscal date ending.
    pub fiscal_date_ending: Option<String>,
    /// Estimated EPS.
    pub estimate: Option<f64>,
    /// Currency.
    pub currency: Option<String>,
}

#[cfg(feature = "alphavantage")]
impl From<crate::adapters::alphavantage::models::EarningsCalendarEntryDTO>
    for EarningsCalendarEntry
{
    fn from(d: crate::adapters::alphavantage::models::EarningsCalendarEntryDTO) -> Self {
        Self {
            symbol: d.symbol,
            name: d.name,
            report_date: d.report_date,
            fiscal_date_ending: d.fiscal_date_ending,
            estimate: d.estimate,
            currency: d.currency,
        }
    }
}

/// An upcoming IPO calendar entry.
#[cfg(feature = "alphavantage")]
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpoCalendarEntry {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Company name.
    pub name: Option<String>,
    /// IPO date.
    pub ipo_date: Option<String>,
    /// Price range (e.g., `"$15-$17"`).
    pub price_range: Option<String>,
    /// Exchange.
    pub exchange: Option<String>,
}

#[cfg(feature = "alphavantage")]
impl From<crate::adapters::alphavantage::models::IpoCalendarEntryDTO> for IpoCalendarEntry {
    fn from(d: crate::adapters::alphavantage::models::IpoCalendarEntryDTO) -> Self {
        Self {
            symbol: d.symbol,
            name: d.name,
            ipo_date: d.ipo_date,
            price_range: d.price_range,
            exchange: d.exchange,
        }
    }
}

/// Fetch the upcoming earnings calendar (market-wide, not symbol-filtered).
#[cfg(feature = "alphavantage")]
pub async fn earnings_calendar() -> Result<Vec<EarningsCalendarEntry>> {
    crate::adapters::alphavantage::fundamentals::earnings_calendar()
        .await
        .map(|v| v.into_iter().map(Into::into).collect())
}

/// Fetch the upcoming IPO calendar (market-wide, not symbol-filtered).
#[cfg(feature = "alphavantage")]
pub async fn ipo_calendar() -> Result<Vec<IpoCalendarEntry>> {
    crate::adapters::alphavantage::fundamentals::ipo_calendar()
        .await
        .map(|v| v.into_iter().map(Into::into).collect())
}
