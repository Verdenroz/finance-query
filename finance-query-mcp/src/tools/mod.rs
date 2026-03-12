pub mod analysis;
pub mod chart;
pub mod crypto;
pub mod dividends;
pub mod edgar;
pub mod feeds;
pub mod financials;
pub mod fred;
pub mod helpers;
pub mod indicators;
pub mod market;
pub mod news;
pub mod options;
pub mod quotes;
pub mod risk;
pub mod search;
pub mod transcripts;

use rmcp::{
    ErrorData as McpError, ServerHandler, handler::server::tool::ToolRouter,
    handler::server::wrapper::Parameters, model::CallToolResult, tool, tool_handler, tool_router,
};
use schemars::JsonSchema;
use serde::Deserialize;

// ── Parameter structs ─────────────────────────────────────────────────────────

#[derive(Deserialize, JsonSchema)]
pub struct SymbolParams {
    /// Stock ticker symbol (e.g., "AAPL", "MSFT", "TSLA")
    pub symbol: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct SymbolsParams {
    /// Comma-separated list of ticker symbols (e.g., "AAPL,MSFT,GOOG")
    pub symbols: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct ChartParams {
    /// Stock ticker symbol
    pub symbol: String,
    /// Candle interval: 1m|5m|15m|30m|1h|1d|1wk|1mo|3mo (default: 1d)
    pub interval: Option<String>,
    /// Time range: 1d|5d|1mo|3mo|6mo|1y|2y|5y|10y|ytd|max (default: 1mo)
    pub range: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct FinancialsParams {
    /// Stock ticker symbol
    pub symbol: String,
    /// Statement type: income | balance | cashflow
    pub statement: String,
    /// Reporting frequency: annual | quarterly (default: annual)
    pub frequency: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct IndicatorsParams {
    /// Stock ticker symbol
    pub symbol: String,
    /// Candle interval: 1d|1wk|1mo (default: 1d)
    pub interval: Option<String>,
    /// Time range: 1mo|3mo|6mo|1y|2y|5y (default: 1y)
    pub range: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct SearchParams {
    /// Search query string (company name or ticker symbol)
    pub query: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct ScreenerParams {
    /// Screener type: most-actives | day-gainers | day-losers | growth-technology-stocks |
    /// undervalued-growth-stocks | undervalued-large-caps | aggressive-small-caps |
    /// small-cap-gainers | most-shorted-stocks | high-yield-bond | top-mutual-funds |
    /// conservative-foreign-funds | portfolio-anchors | solid-large-growth-funds |
    /// solid-midcap-growth-funds
    pub screener_type: String,
    /// Number of results to return (default: 25, max: 250)
    pub count: Option<u32>,
}

#[derive(Deserialize, JsonSchema)]
pub struct NewsParams {
    /// Stock ticker symbol (optional; omit for general market news)
    pub symbol: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct MarketSummaryParams {
    /// Region code: US|GB|DE|CA|AU|FR|IN|CN|HK|BR|TW|SG (default: US)
    pub region: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct DividendsParams {
    /// Stock ticker symbol
    pub symbol: String,
    /// Time range: 1y|2y|5y|10y|max (default: max)
    pub range: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct RiskParams {
    /// Stock ticker symbol
    pub symbol: String,
    /// Candle interval: 1d|1wk (default: 1d)
    pub interval: Option<String>,
    /// Time range: 1y|2y|5y (default: 1y)
    pub range: Option<String>,
    /// Benchmark symbol for beta calculation (e.g., "SPY")
    pub benchmark: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct CryptoParams {
    /// Number of top coins to return (default: 50, max: 250)
    pub count: Option<u32>,
    /// Quote currency (default: usd)
    pub vs_currency: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct OptionsParams {
    /// Stock ticker symbol
    pub symbol: String,
    /// Expiration date as Unix timestamp in seconds (optional; defaults to nearest expiration)
    pub expiration: Option<i64>,
}

#[derive(Deserialize, JsonSchema)]
pub struct RecommendationsParams {
    /// Stock ticker symbol
    pub symbol: String,
    /// Maximum number of recommendations to return (default: 5)
    pub limit: Option<u32>,
}

#[derive(Deserialize, JsonSchema)]
pub struct SplitsParams {
    /// Stock ticker symbol
    pub symbol: String,
    /// Time range: 1y|2y|5y|10y|max (default: max)
    pub range: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct TrendingParams {
    /// Region code: US|GB|DE|CA|AU|FR|IN|CN|HK|BR|TW|SG (default: US)
    pub region: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct IndicesParams {
    /// Region: americas|europe|asia-pacific|middle-east-africa|currencies (default: all)
    pub region: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct MarketHoursParams {
    /// Region code: US|GB|DE|CA|AU|FR|IN|CN|HK|BR|TW|SG (default: US)
    pub region: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct SectorParams {
    /// Sector slug: technology|financial-services|consumer-cyclical|communication-services|
    /// healthcare|industrials|consumer-defensive|energy|basic-materials|real-estate|utilities
    pub sector: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct IndustryParams {
    /// Industry slug (e.g., semiconductors, biotechnology, banks-diversified)
    pub industry: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct HoldersParams {
    /// Stock ticker symbol
    pub symbol: String,
    /// Holder type: major | institutional | mutualfund | insider-transactions | insider-purchases | insider-roster
    pub holder_type: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct AnalysisParams {
    /// Stock ticker symbol
    pub symbol: String,
    /// Analysis type: recommendations | upgrades-downgrades | earnings-estimate | earnings-history
    pub analysis_type: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct FredSeriesParams {
    /// FRED series ID (e.g., "FEDFUNDS", "CPIAUCSL", "GDP", "UNRATE")
    pub id: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct TreasuryYieldsParams {
    /// Year to fetch yield curve data for (default: current year)
    pub year: Option<u32>,
}

#[derive(Deserialize, JsonSchema)]
pub struct TranscriptsParams {
    /// Stock ticker symbol
    pub symbol: String,
    /// Maximum number of transcripts to return (default: all)
    pub limit: Option<u32>,
}

#[derive(Deserialize, JsonSchema)]
pub struct FeedsParams {
    /// Comma-separated feed sources: federal-reserve|sec|marketwatch|cnbc|bloomberg|ft|nyt|
    /// guardian|investing|bea|ecb|cfpb|wsj|fortune|businesswire|coindesk|cointelegraph|
    /// techcrunch|hackernews|oilprice|calculatedrisk|scmp|nikkei|boe|venturebeat|yc|
    /// economist|financialpost|ftlex|ritholtz
    /// (default: marketwatch, bloomberg, wsj, fortune)
    pub sources: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct LookupParams {
    /// Search query (company name or ticker symbol)
    pub query: String,
    /// Filter by type: equity|etf|mutualfund|index|future|currency|cryptocurrency (default: all)
    pub query_type: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct BatchSymbolsParams {
    /// Comma-separated list of ticker symbols (e.g., "AAPL,MSFT,GOOG")
    pub symbols: String,
    /// Candle interval: 1m|5m|15m|30m|1h|1d|1wk|1mo|3mo (default: 1d)
    pub interval: Option<String>,
    /// Time range: 1d|5d|1mo|3mo|6mo|1y|2y|5y|10y|ytd|max (default: 1mo)
    pub range: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct BatchDividendsParams {
    /// Comma-separated list of ticker symbols (e.g., "AAPL,KO,JNJ")
    pub symbols: String,
    /// Time range: 1y|2y|5y|10y|max (default: 1y)
    pub range: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct BatchFinancialsParams {
    /// Comma-separated list of ticker symbols (e.g., "AAPL,MSFT,GOOGL")
    pub symbols: String,
    /// Statement type: income | balance | cashflow
    pub statement: String,
    /// Reporting frequency: annual | quarterly (default: annual)
    pub frequency: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct EdgarSearchParams {
    /// Search query (keywords in filing text)
    pub query: String,
    /// Comma-separated form types to filter (e.g., "10-K,10-Q,8-K")
    pub forms: Option<String>,
    /// Start date filter in YYYY-MM-DD format
    pub start_date: Option<String>,
    /// End date filter in YYYY-MM-DD format
    pub end_date: Option<String>,
}

// ── Tool handler ──────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct FinanceTools {
    tool_router: ToolRouter<Self>,
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for FinanceTools {}

#[tool_router(router = tool_router)]
impl FinanceTools {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(
        description = "Get current quote and company data for a stock symbol (price, market cap, PE ratio, 52-week range, etc.)"
    )]
    async fn get_quote(&self, p: Parameters<SymbolParams>) -> Result<CallToolResult, McpError> {
        quotes::get_quote(p.0.symbol).await
    }

    #[tool(description = "Get current quotes for multiple stock symbols in one request.")]
    async fn get_quotes(&self, p: Parameters<SymbolsParams>) -> Result<CallToolResult, McpError> {
        quotes::get_quotes(p.0.symbols).await
    }

    #[tool(description = "Get similar stock recommendations and analyst ratings for a symbol.")]
    async fn get_recommendations(
        &self,
        p: Parameters<RecommendationsParams>,
    ) -> Result<CallToolResult, McpError> {
        quotes::get_recommendations(p.0.symbol, p.0.limit).await
    }

    #[tool(description = "Get historical stock split history for a symbol.")]
    async fn get_splits(&self, p: Parameters<SplitsParams>) -> Result<CallToolResult, McpError> {
        quotes::get_splits(p.0.symbol, p.0.range).await
    }

    #[tool(description = "Get historical OHLCV candlestick chart data for a symbol.")]
    async fn get_chart(&self, p: Parameters<ChartParams>) -> Result<CallToolResult, McpError> {
        chart::get_chart(p.0.symbol, p.0.interval, p.0.range).await
    }

    #[tool(
        description = "Get historical OHLCV candlestick data for multiple symbols in one request. Use for portfolio analysis and cross-symbol comparison."
    )]
    async fn get_charts(
        &self,
        p: Parameters<BatchSymbolsParams>,
    ) -> Result<CallToolResult, McpError> {
        chart::get_charts(p.0.symbols, p.0.interval, p.0.range).await
    }

    #[tool(
        description = "Get lightweight close-price sparklines for multiple symbols. Faster and smaller than get_charts — use when you only need price direction/trend across many symbols."
    )]
    async fn get_spark(
        &self,
        p: Parameters<BatchSymbolsParams>,
    ) -> Result<CallToolResult, McpError> {
        chart::get_spark(p.0.symbols, p.0.interval, p.0.range).await
    }

    #[tool(
        description = "Get income statement, balance sheet, or cash flow statement for a symbol."
    )]
    async fn get_financials(
        &self,
        p: Parameters<FinancialsParams>,
    ) -> Result<CallToolResult, McpError> {
        financials::get_financials(p.0.symbol, p.0.statement, p.0.frequency).await
    }

    #[tool(
        description = "Get financial statements for multiple symbols in one request. Use for comparing fundamentals across companies."
    )]
    async fn get_batch_financials(
        &self,
        p: Parameters<BatchFinancialsParams>,
    ) -> Result<CallToolResult, McpError> {
        financials::get_batch_financials(p.0.symbols, p.0.statement, p.0.frequency).await
    }

    #[tool(
        description = "Get all 42 technical analysis indicators (SMA, EMA, RSI, MACD, Bollinger Bands, Ichimoku, etc.) for a symbol."
    )]
    async fn get_indicators(
        &self,
        p: Parameters<IndicatorsParams>,
    ) -> Result<CallToolResult, McpError> {
        indicators::get_indicators(p.0.symbol, p.0.interval, p.0.range).await
    }

    #[tool(
        description = "Get all technical indicators for multiple symbols in one request. Use for portfolio-wide technical screening."
    )]
    async fn get_batch_indicators(
        &self,
        p: Parameters<BatchSymbolsParams>,
    ) -> Result<CallToolResult, McpError> {
        indicators::get_batch_indicators(p.0.symbols, p.0.interval, p.0.range).await
    }

    #[tool(description = "Search for stocks, ETFs, and companies by name or ticker symbol.")]
    async fn search(&self, p: Parameters<SearchParams>) -> Result<CallToolResult, McpError> {
        search::search(p.0.query).await
    }

    #[tool(
        description = "Discover tickers filtered by type (equity, ETF, mutual fund, index, future, currency, cryptocurrency)."
    )]
    async fn lookup(&self, p: Parameters<LookupParams>) -> Result<CallToolResult, McpError> {
        search::get_lookup(p.0.query, p.0.query_type).await
    }

    #[tool(
        description = "Get results from a predefined stock screener (e.g., most-actives, day-gainers, undervalued-growth-stocks)."
    )]
    async fn screener(&self, p: Parameters<ScreenerParams>) -> Result<CallToolResult, McpError> {
        search::screener(p.0.screener_type, p.0.count).await
    }

    #[tool(
        description = "Get recent news. If a symbol is provided, returns news for that stock; otherwise returns general market news."
    )]
    async fn get_news(&self, p: Parameters<NewsParams>) -> Result<CallToolResult, McpError> {
        news::get_news(p.0.symbol).await
    }

    #[tool(
        description = "Fetch RSS/Atom news from financial publishers (Bloomberg, WSJ, MarketWatch, FT, SEC, etc.)."
    )]
    async fn get_feeds(&self, p: Parameters<FeedsParams>) -> Result<CallToolResult, McpError> {
        feeds::get_feeds(p.0.sources).await
    }

    #[tool(description = "Get market overview with major indices and currencies for a region.")]
    async fn get_market_summary(
        &self,
        p: Parameters<MarketSummaryParams>,
    ) -> Result<CallToolResult, McpError> {
        market::get_market_summary(p.0.region).await
    }

    #[tool(
        description = "Get the CNN Fear & Greed Index — market sentiment from extreme fear (0) to extreme greed (100)."
    )]
    async fn get_fear_and_greed(&self) -> Result<CallToolResult, McpError> {
        market::get_fear_and_greed().await
    }

    #[tool(description = "Get currently trending stock tickers for a region.")]
    async fn get_trending(
        &self,
        p: Parameters<TrendingParams>,
    ) -> Result<CallToolResult, McpError> {
        market::get_trending(p.0.region).await
    }

    #[tool(
        description = "Get world market indices (S&P 500, DAX, Nikkei, etc.), optionally filtered by region."
    )]
    async fn get_indices(&self, p: Parameters<IndicesParams>) -> Result<CallToolResult, McpError> {
        market::get_indices(p.0.region).await
    }

    #[tool(description = "Get current market hours and open/closed status for a region.")]
    async fn get_market_hours(
        &self,
        p: Parameters<MarketHoursParams>,
    ) -> Result<CallToolResult, McpError> {
        market::get_market_hours(p.0.region).await
    }

    #[tool(
        description = "Get comprehensive sector data (overview, performance, top companies, ETFs) for one of the 11 GICS sectors."
    )]
    async fn get_sector(&self, p: Parameters<SectorParams>) -> Result<CallToolResult, McpError> {
        market::get_sector(p.0.sector).await
    }

    #[tool(
        description = "Get comprehensive industry data (overview, performance, top companies) for a specific industry slug."
    )]
    async fn get_industry(
        &self,
        p: Parameters<IndustryParams>,
    ) -> Result<CallToolResult, McpError> {
        market::get_industry(p.0.industry).await
    }

    #[tool(
        description = "Get dividend history and analytics (CAGR, average payment, payout count) for a dividend-paying stock."
    )]
    async fn get_dividends(
        &self,
        p: Parameters<DividendsParams>,
    ) -> Result<CallToolResult, McpError> {
        dividends::get_dividends(p.0.symbol, p.0.range).await
    }

    #[tool(
        description = "Get dividend history for multiple symbols in one request. Use for portfolio income analysis."
    )]
    async fn get_batch_dividends(
        &self,
        p: Parameters<BatchDividendsParams>,
    ) -> Result<CallToolResult, McpError> {
        dividends::get_batch_dividends(p.0.symbols, p.0.range).await
    }

    #[tool(
        description = "Get ownership data for a stock: major holders, institutional/fund ownership, or insider activity."
    )]
    async fn get_holders(&self, p: Parameters<HoldersParams>) -> Result<CallToolResult, McpError> {
        analysis::get_holders(p.0.symbol, p.0.holder_type).await
    }

    #[tool(
        description = "Get analyst data for a stock: recommendation trends, upgrades/downgrades, earnings estimates, or earnings history."
    )]
    async fn get_analysis(
        &self,
        p: Parameters<AnalysisParams>,
    ) -> Result<CallToolResult, McpError> {
        analysis::get_analysis(p.0.symbol, p.0.analysis_type).await
    }

    #[tool(
        description = "Get SEC EDGAR XBRL structured financial data (all reported accounting concepts) for a company. Requires EDGAR_EMAIL env var."
    )]
    async fn get_edgar_facts(
        &self,
        p: Parameters<SymbolParams>,
    ) -> Result<CallToolResult, McpError> {
        edgar::get_edgar_facts(p.0.symbol).await
    }

    #[tool(
        description = "Get SEC filing history and company metadata from EDGAR (up to 1000 most recent filings). Requires EDGAR_EMAIL env var."
    )]
    async fn get_edgar_submissions(
        &self,
        p: Parameters<SymbolParams>,
    ) -> Result<CallToolResult, McpError> {
        edgar::get_edgar_submissions(p.0.symbol).await
    }

    #[tool(
        description = "Full-text search across SEC EDGAR filings with optional form type and date filters. Requires EDGAR_EMAIL env var."
    )]
    async fn get_edgar_search(
        &self,
        p: Parameters<EdgarSearchParams>,
    ) -> Result<CallToolResult, McpError> {
        edgar::get_edgar_search(p.0.query, p.0.forms, p.0.start_date, p.0.end_date).await
    }

    #[tool(
        description = "Get risk analytics: VaR (95/99%), Sharpe/Sortino/Calmar ratios, beta, and maximum drawdown for a symbol."
    )]
    async fn get_risk(&self, p: Parameters<RiskParams>) -> Result<CallToolResult, McpError> {
        risk::get_risk(p.0.symbol, p.0.interval, p.0.range, p.0.benchmark).await
    }

    #[tool(
        description = "Get top cryptocurrency coins by market cap from CoinGecko (no API key required)."
    )]
    async fn get_crypto_coins(
        &self,
        p: Parameters<CryptoParams>,
    ) -> Result<CallToolResult, McpError> {
        crypto::get_crypto_coins(p.0.count, p.0.vs_currency).await
    }

    #[tool(
        description = "Get the options chain for a symbol. Provide an expiration timestamp to get a specific expiry, or omit for the nearest expiration."
    )]
    async fn get_options(&self, p: Parameters<OptionsParams>) -> Result<CallToolResult, McpError> {
        options::get_options(p.0.symbol, p.0.expiration).await
    }

    #[tool(
        description = "Get FRED macroeconomic time series data (e.g., FEDFUNDS, CPIAUCSL, GDP, UNRATE). Requires FRED_API_KEY env var."
    )]
    async fn get_fred_series(
        &self,
        p: Parameters<FredSeriesParams>,
    ) -> Result<CallToolResult, McpError> {
        fred::get_fred_series(p.0.id).await
    }

    #[tool(
        description = "Get US Treasury yield curve data (1m through 30y) for a given year. No API key required."
    )]
    async fn get_treasury_yields(
        &self,
        p: Parameters<TreasuryYieldsParams>,
    ) -> Result<CallToolResult, McpError> {
        fred::get_treasury_yields(p.0.year).await
    }

    #[tool(
        description = "Get earnings call transcripts for a company. Returns the full text of earnings presentations."
    )]
    async fn get_transcripts(
        &self,
        p: Parameters<TranscriptsParams>,
    ) -> Result<CallToolResult, McpError> {
        transcripts::get_transcripts(p.0.symbol, p.0.limit).await
    }
}
