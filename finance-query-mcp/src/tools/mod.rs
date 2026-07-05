pub mod analysis;
pub mod calendar;
pub mod chart;
pub mod crypto;
pub mod dividends;
pub mod edgar;
pub mod feeds;
pub mod financials;
pub mod fred;
pub mod gql;
pub mod helpers;
pub mod indicators;
pub mod market;
pub mod news;
pub mod options;
pub mod quotes;
pub mod risk;
pub mod search;
pub mod transcripts;

use crate::metrics::ToolCallTimer;
use finance_query_server::graphql::FinanceSchema;
use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler,
    handler::server::tool::{ToolCallContext, ToolRouter},
    handler::server::wrapper::Parameters,
    model::{CallToolRequestParams, CallToolResult},
    service::RequestContext,
    tool, tool_handler, tool_router,
};
use schemars::JsonSchema;
use serde::Deserialize;

// ── Parameter structs ─────────────────────────────────────────────────────────

#[derive(Deserialize, JsonSchema)]
pub struct EdgarSubmissionsParams {
    /// Stock ticker symbol (e.g., "AAPL", "MSFT", "TSLA")
    pub symbol: String,
    /// Comma-separated list of GraphQL field names to include; omitted = all fields
    pub fields: Option<String>,
    /// Maximum filings per page; omitted = curated default (25)
    pub limit: Option<u32>,
    /// Opaque continuation token from a previous response's `pageInfo.endCursor`; omitted = first page
    pub cursor: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct EdgarFactsParams {
    /// Stock ticker symbol (e.g., "AAPL", "MSFT", "TSLA")
    pub symbol: String,
    /// XBRL taxonomy (default: "us-gaap"); also try "ifrs-full" or "dei"
    pub taxonomy: Option<String>,
    /// Comma-separated XBRL concept names to filter to (e.g. "Revenues,Assets");
    /// omitted = curated defaults (headline financials)
    pub concepts: Option<String>,
    /// Comma-separated GraphQL sub-fields to include per concept (e.g. "concept,dataPoints");
    /// omitted = curated default set
    pub fields: Option<String>,
    /// Maximum data points per concept per page; omitted = curated default (25).
    /// Applied uniformly across every returned concept.
    pub limit: Option<u32>,
    /// Opaque continuation token from a previous response's `pageInfo.endCursor`; omitted = first page
    pub cursor: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct SymbolsParams {
    /// Comma-separated list of ticker symbols (e.g., "AAPL,MSFT,GOOG")
    pub symbols: String,
    /// Target language for translated text fields (BCP 47, e.g. "ja", "zh-Hant"); English or omitted = no translation
    pub lang: Option<String>,
    /// Comma-separated list of GraphQL field names to include; omitted = curated default set
    pub fields: Option<String>,
    /// Maximum symbols per page; omitted = curated default (25)
    pub limit: Option<u32>,
    /// Opaque continuation token from a previous response's `pageInfo.endCursor`; omitted = first page
    pub cursor: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct CalendarParams {
    /// Comma-separated list of ticker symbols (e.g., "AAPL,MSFT,TSLA")
    pub symbols: String,
    /// Forward time window: 1d|5d|1mo|3mo|6mo|1y|2y|5y|10y|ytd|max (default: 1mo)
    pub range: Option<String>,
    /// Comma-separated list of GraphQL field names to include; omitted = all fields
    pub fields: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ChartParams {
    /// One or more comma-separated ticker symbols (e.g., "AAPL" or "AAPL,MSFT,GOOG")
    pub symbols: String,
    /// Candle interval: 1m|5m|15m|30m|1h|1d|1wk|1mo|3mo (default: 1d)
    pub interval: Option<String>,
    /// Time range: 1d|5d|1mo|3mo|6mo|1y|2y|5y|10y|ytd|max (default: 1mo). Ignored when `start` is set.
    pub range: Option<String>,
    /// Start date as Unix timestamp (seconds). When provided, overrides `range`.
    pub start: Option<i64>,
    /// End date as Unix timestamp (seconds). Defaults to now when `start` is set.
    pub end: Option<i64>,
    /// Comma-separated list of GraphQL field names to include; omitted = curated default set
    pub fields: Option<String>,
    /// Maximum candles per page; omitted = curated default (25)
    pub limit: Option<u32>,
    /// Opaque continuation token from a previous response's `pageInfo.endCursor`; omitted = first page
    pub cursor: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct FinancialsParams {
    /// One or more comma-separated ticker symbols (e.g., "AAPL" or "AAPL,MSFT,GOOGL")
    pub symbols: String,
    /// Statement type: income | balance | cashflow
    pub statement: String,
    /// Reporting frequency: annual | quarterly (default: annual)
    pub frequency: Option<String>,
    /// Comma-separated list of line-item metrics to filter to; omitted = all reported metrics
    pub metrics: Option<String>,
    /// Comma-separated list of GraphQL field names to include; omitted = curated default set
    pub fields: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct IndicatorsParams {
    /// One or more comma-separated ticker symbols (e.g., "AAPL" or "AAPL,MSFT,GOOG")
    pub symbols: String,
    /// Candle interval: 1d|1wk|1mo (default: 1d)
    pub interval: Option<String>,
    /// Time range: 1mo|3mo|6mo|1y|2y|5y (default: 1y)
    pub range: Option<String>,
    /// Comma-separated list of GraphQL field names to include; omitted = curated default set
    pub fields: Option<String>,
    /// Maximum symbols per page (only applies when multiple symbols given); omitted = curated default (25)
    pub limit: Option<u32>,
    /// Opaque continuation token from a previous response's `pageInfo.endCursor`; omitted = first page
    pub cursor: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct SearchParams {
    /// Search query string (company name or ticker symbol)
    pub query: String,
    /// Target language for translated text fields (BCP 47, e.g. "ja", "zh-Hant"); English or omitted = no translation
    pub lang: Option<String>,
    /// Comma-separated list of GraphQL field names to include; omitted = curated default set
    pub fields: Option<String>,
    /// Maximum quotes per page; omitted = curated default (25)
    pub limit: Option<u32>,
    /// Opaque continuation token from a previous response's `pageInfo.endCursor`; omitted = first page
    pub cursor: Option<String>,
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
    /// Comma-separated list of GraphQL field names to include; omitted = curated default set
    pub fields: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct NewsParams {
    /// Stock ticker symbol (optional; omit for general market news)
    pub symbol: Option<String>,
    /// Target language for translated text fields (BCP 47, e.g. "ja", "zh-Hant"); English or omitted = no translation
    pub lang: Option<String>,
    /// Comma-separated list of GraphQL field names to include; omitted = curated default set
    pub fields: Option<String>,
    /// Maximum articles per page; omitted = curated default (25)
    pub limit: Option<u32>,
    /// Opaque continuation token from a previous response's `pageInfo.endCursor`; omitted = first page
    pub cursor: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct MarketSummaryParams {
    /// Region code: US|GB|DE|CA|AU|FR|IN|CN|HK|BR|TW|SG (default: US)
    pub region: Option<String>,
    /// Target language for translated text fields (BCP 47, e.g. "ja", "zh-Hant"); English or omitted = no translation
    pub lang: Option<String>,
    /// Comma-separated list of GraphQL field names to include; omitted = curated default set
    pub fields: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct FearAndGreedParams {
    /// Comma-separated list of GraphQL field names to include; omitted = curated default set
    pub fields: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct DividendsParams {
    /// One or more comma-separated ticker symbols (e.g., "AAPL" or "AAPL,KO,JNJ")
    pub symbols: String,
    /// Time range: 1y|2y|5y|10y|max (default: max)
    pub range: Option<String>,
    /// Comma-separated list of GraphQL field names to include; omitted = curated default set
    pub fields: Option<String>,
    /// Maximum dividend payments per page; omitted = curated default (25)
    pub limit: Option<u32>,
    /// Opaque continuation token from a previous response's `pageInfo.endCursor`; omitted = first page
    pub cursor: Option<String>,
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
    /// Comma-separated list of GraphQL field names to include; omitted = curated default set
    pub fields: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct CryptoParams {
    /// Number of top coins to return (default: 50, max: 250)
    pub count: Option<u32>,
    /// Quote currency (default: usd)
    pub vs_currency: Option<String>,
    /// Comma-separated list of GraphQL field names to include; omitted = all fields
    pub fields: Option<String>,
    /// Maximum coins per page; omitted = curated default (25)
    pub limit: Option<u32>,
    /// Opaque continuation token from a previous response's `pageInfo.endCursor`; omitted = first page
    pub cursor: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct OptionsParams {
    /// Stock ticker symbol
    pub symbol: String,
    /// Expiration date as Unix timestamp in seconds (optional; defaults to nearest expiration)
    pub expiration: Option<i64>,
    /// Comma-separated list of GraphQL field names to include; omitted = curated default set
    pub fields: Option<String>,
    /// Maximum contracts per side (calls/puts) per page; omitted = curated default (25)
    pub limit: Option<u32>,
    /// Opaque continuation token from a previous response's `pageInfo.endCursor` (applied to both calls and puts); omitted = first page
    pub cursor: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct RecommendationsParams {
    /// Stock ticker symbol
    pub symbol: String,
    /// Maximum number of recommendations to return (default: 5)
    pub limit: Option<u32>,
    /// Comma-separated list of GraphQL field names to include; omitted = all fields
    pub fields: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct SplitsParams {
    /// Stock ticker symbol
    pub symbol: String,
    /// Time range: 1y|2y|5y|10y|max (default: max)
    pub range: Option<String>,
    /// Comma-separated list of GraphQL field names to include; omitted = curated default set
    pub fields: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct TrendingParams {
    /// Region code: US|GB|DE|CA|AU|FR|IN|CN|HK|BR|TW|SG (default: US)
    pub region: Option<String>,
    /// Comma-separated list of GraphQL field names to include; omitted = curated default set
    pub fields: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct IndicesParams {
    /// Region: americas|europe|asia-pacific|middle-east-africa|currencies (default: all)
    pub region: Option<String>,
    /// Comma-separated list of GraphQL field names to include; omitted = curated default set
    pub fields: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct MarketHoursParams {
    /// Region code: US|GB|DE|CA|AU|FR|IN|CN|HK|BR|TW|SG (default: US)
    pub region: Option<String>,
    /// Comma-separated list of GraphQL field names to include; omitted = all fields
    pub fields: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct SectorParams {
    /// Sector slug: technology|financial-services|consumer-cyclical|communication-services|
    /// healthcare|industrials|consumer-defensive|energy|basic-materials|real-estate|utilities
    pub sector: String,
    /// Target language for translated text fields (BCP 47, e.g. "ja", "zh-Hant"); English or omitted = no translation
    pub lang: Option<String>,
    /// Comma-separated list of GraphQL field names to include; omitted = curated default set
    pub fields: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct IndustryParams {
    /// Industry slug (e.g., semiconductors, biotechnology, banks-diversified)
    pub industry: String,
    /// Target language for translated text fields (BCP 47, e.g. "ja", "zh-Hant"); English or omitted = no translation
    pub lang: Option<String>,
    /// Comma-separated list of GraphQL field names to include; omitted = curated default set
    pub fields: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct HoldersParams {
    /// Stock ticker symbol
    pub symbol: String,
    /// Holder type: major | institutional | mutualfund | insider-transactions | insider-purchases | insider-roster
    pub holder_type: String,
    /// Comma-separated list of GraphQL field names to include; omitted = curated default set
    pub fields: Option<String>,
    /// Maximum entries per page for holder types with a list (institutional,
    /// mutualfund, insider-transactions, insider-roster); omitted = curated
    /// default (25). No-op for major/insider-purchases (no list).
    pub limit: Option<u32>,
    /// Opaque continuation token from a previous response's `pageInfo.endCursor`; omitted = first page
    pub cursor: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct AnalysisParams {
    /// Stock ticker symbol
    pub symbol: String,
    /// Analysis type: recommendations | upgrades-downgrades | earnings-estimate | earnings-history
    pub analysis_type: String,
    /// Comma-separated list of GraphQL field names to include; omitted = curated default set
    pub fields: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct FredSeriesParams {
    /// FRED series ID (e.g., "FEDFUNDS", "CPIAUCSL", "GDP", "UNRATE")
    pub id: String,
    /// Comma-separated list of GraphQL field names to include; omitted = all fields
    pub fields: Option<String>,
    /// Maximum observations per page; omitted = curated default (25)
    pub limit: Option<u32>,
    /// Opaque continuation token from a previous response's `pageInfo.endCursor`; omitted = first page
    pub cursor: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct TreasuryYieldsParams {
    /// Year to fetch yield curve data for (default: current year)
    pub year: Option<u32>,
    /// Comma-separated list of GraphQL field names to include; omitted = all fields
    pub fields: Option<String>,
    /// Maximum rows per page; omitted = curated default (25)
    pub limit: Option<u32>,
    /// Opaque continuation token from a previous response's `pageInfo.endCursor`; omitted = first page
    pub cursor: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct TranscriptsParams {
    /// Stock ticker symbol
    pub symbol: String,
    /// Maximum number of transcripts to return (default: all)
    pub limit: Option<u32>,
    /// Target language for translated text fields (BCP 47, e.g. "ja", "zh-Hant"); English or omitted = no translation
    pub lang: Option<String>,
    /// Comma-separated list of GraphQL field names to include; omitted = curated default set
    pub fields: Option<String>,
    /// Maximum transcript paragraphs per page; omitted = curated default (25). A
    /// full call's `text` is returned as paginated paragraphs, not one giant blob
    pub paragraph_limit: Option<u32>,
    /// Opaque continuation token from a previous response's paragraphs `pageInfo.endCursor`; omitted = first page
    pub paragraph_cursor: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct FeedsParams {
    /// Comma-separated feed sources: federal-reserve|sec|marketwatch|cnbc|bloomberg|ft|nyt|
    /// guardian|investing|bea|ecb|cfpb|wsj|fortune|businesswire|coindesk|cointelegraph|
    /// techcrunch|hackernews|oilprice|calculatedrisk|scmp|nikkei|boe|venturebeat|yc|
    /// economist|financialpost|ftlex|ritholtz
    /// (default: marketwatch, bloomberg, wsj, fortune)
    pub sources: Option<String>,
    /// Comma-separated list of GraphQL field names to include; omitted = curated default set
    pub fields: Option<String>,
    /// Maximum entries per page; omitted = curated default (25)
    pub limit: Option<u32>,
    /// Opaque continuation token from a previous response's `pageInfo.endCursor`; omitted = first page
    pub cursor: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct LookupParams {
    /// Search query (company name or ticker symbol)
    pub query: String,
    /// Filter by type: equity|etf|mutualfund|index|future|currency|cryptocurrency (default: all)
    pub query_type: Option<String>,
    /// Target language for translated text fields (BCP 47, e.g. "ja", "zh-Hant"); English or omitted = no translation
    pub lang: Option<String>,
    /// Comma-separated list of GraphQL field names to include; omitted = curated default set
    pub fields: Option<String>,
    /// Include logo URLs (requires an additional upstream API call); default: false
    pub logo: Option<bool>,
}

#[derive(Deserialize, JsonSchema)]
pub struct BatchSymbolsParams {
    /// Comma-separated list of ticker symbols (e.g., "AAPL,MSFT,GOOG")
    pub symbols: String,
    /// Candle interval: 1m|5m|15m|30m|1h|1d|1wk|1mo|3mo (default: 1d)
    pub interval: Option<String>,
    /// Time range: 1d|5d|1mo|3mo|6mo|1y|2y|5y|10y|ytd|max (default: 1mo)
    pub range: Option<String>,
    /// Comma-separated list of GraphQL field names to include; omitted = curated default set
    pub fields: Option<String>,
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
    /// Pagination offset; omitted = 0 (first page)
    pub from: Option<u32>,
    /// Page size; omitted = 100, max 100
    pub size: Option<u32>,
}

// ── Tool handler ──────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct FinanceTools {
    tool_router: ToolRouter<Self>,
    schema: FinanceSchema,
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for FinanceTools {
    // Manual replica of the #[tool_handler]-generated call_tool (the macro skips
    // methods that already exist) — the single metrics chokepoint for all tools.
    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let timer = ToolCallTimer::new(request.name.clone());
        let tcc = ToolCallContext::new(self, request, context);
        let result = self.tool_router.call(tcc).await;
        // A tool-level failure surfaces as Ok(result) with is_error set, not Err.
        let ok = matches!(&result, Ok(r) if r.is_error != Some(true));
        timer.observe(ok);
        result
    }
}

#[tool_router(router = tool_router)]
impl FinanceTools {
    pub fn new(schema: FinanceSchema) -> Self {
        Self {
            tool_router: Self::tool_router(),
            schema,
        }
    }

    #[tool(
        description = "Get current quote and company data (price, market cap, PE ratio, 52-week range, etc.) for one or more stock symbols (comma-separated). A single symbol returns one quote object; multiple symbols return a paginated batch of quotes plus per-symbol errors."
    )]
    async fn get_quote(&self, p: Parameters<SymbolsParams>) -> Result<CallToolResult, McpError> {
        quotes::get_quote(
            &self.schema,
            p.0.symbols,
            p.0.lang,
            p.0.fields,
            p.0.limit,
            p.0.cursor,
        )
        .await
    }

    #[tool(description = "Get similar stock recommendations and analyst ratings for a symbol.")]
    async fn get_recommendations(
        &self,
        p: Parameters<RecommendationsParams>,
    ) -> Result<CallToolResult, McpError> {
        quotes::get_recommendations(&self.schema, p.0.symbol, p.0.limit, p.0.fields).await
    }

    #[tool(description = "Get historical stock split history for a symbol.")]
    async fn get_splits(&self, p: Parameters<SplitsParams>) -> Result<CallToolResult, McpError> {
        quotes::get_splits(&self.schema, p.0.symbol, p.0.range, p.0.fields).await
    }

    #[tool(
        description = "Get historical OHLCV candlestick chart data for one or more stock symbols (comma-separated). A single symbol supports start/end absolute timestamps and returns one chart; multiple symbols return a batch of charts plus per-symbol errors (interval/range only, no start/end)."
    )]
    async fn get_chart(&self, p: Parameters<ChartParams>) -> Result<CallToolResult, McpError> {
        chart::get_chart(
            &self.schema,
            p.0.symbols,
            p.0.interval,
            p.0.range,
            p.0.start,
            p.0.end,
            p.0.fields,
            p.0.limit,
            p.0.cursor,
        )
        .await
    }

    #[tool(
        description = "Get a time-sorted calendar of upcoming financial events (earnings with estimates, ex-dividend and dividend-payment dates, options expirations, and — when FRED is configured — market-wide economic releases) across multiple symbols. Answers 'what's coming up for my portfolio?' in one call."
    )]
    async fn get_calendar(
        &self,
        p: Parameters<CalendarParams>,
    ) -> Result<CallToolResult, McpError> {
        calendar::get_calendar(&self.schema, p.0.symbols, p.0.range, p.0.fields).await
    }

    #[tool(
        description = "Get lightweight close-price sparklines for multiple symbols. Faster and smaller than get_charts — use when you only need price direction/trend across many symbols."
    )]
    async fn get_spark(
        &self,
        p: Parameters<BatchSymbolsParams>,
    ) -> Result<CallToolResult, McpError> {
        chart::get_spark(
            &self.schema,
            p.0.symbols,
            p.0.interval,
            p.0.range,
            p.0.fields,
        )
        .await
    }

    #[tool(
        description = "Get income statement, balance sheet, or cash flow statement for one or more stock symbols (comma-separated). A single symbol returns one statement; multiple symbols return a batch plus per-symbol errors."
    )]
    async fn get_financials(
        &self,
        p: Parameters<FinancialsParams>,
    ) -> Result<CallToolResult, McpError> {
        financials::get_financials(
            &self.schema,
            p.0.symbols,
            p.0.statement,
            p.0.frequency,
            p.0.metrics,
            p.0.fields,
        )
        .await
    }

    #[tool(
        description = "Get all 42 technical analysis indicators (SMA, EMA, RSI, MACD, Bollinger Bands, Ichimoku, etc.) for one or more stock symbols (comma-separated). A single symbol returns one indicators object; multiple symbols return a paginated batch plus per-symbol errors."
    )]
    async fn get_indicators(
        &self,
        p: Parameters<IndicatorsParams>,
    ) -> Result<CallToolResult, McpError> {
        indicators::get_indicators(
            &self.schema,
            p.0.symbols,
            p.0.interval,
            p.0.range,
            p.0.fields,
            p.0.limit,
            p.0.cursor,
        )
        .await
    }

    #[tool(description = "Search for stocks, ETFs, and companies by name or ticker symbol.")]
    async fn search(&self, p: Parameters<SearchParams>) -> Result<CallToolResult, McpError> {
        search::search(
            &self.schema,
            p.0.query,
            p.0.lang,
            p.0.fields,
            p.0.limit,
            p.0.cursor,
        )
        .await
    }

    #[tool(
        description = "Discover tickers filtered by type (equity, ETF, mutual fund, index, future, currency, cryptocurrency)."
    )]
    async fn lookup(&self, p: Parameters<LookupParams>) -> Result<CallToolResult, McpError> {
        search::get_lookup(
            &self.schema,
            p.0.query,
            p.0.query_type,
            p.0.lang,
            p.0.fields,
            p.0.logo,
        )
        .await
    }

    #[tool(
        description = "Get results from a predefined stock screener (e.g., most-actives, day-gainers, undervalued-growth-stocks)."
    )]
    async fn screener(&self, p: Parameters<ScreenerParams>) -> Result<CallToolResult, McpError> {
        search::screener(&self.schema, p.0.screener_type, p.0.count, p.0.fields).await
    }

    #[tool(
        description = "Get recent news. If a symbol is provided, returns news for that stock; otherwise returns general market news."
    )]
    async fn get_news(&self, p: Parameters<NewsParams>) -> Result<CallToolResult, McpError> {
        news::get_news(
            &self.schema,
            p.0.symbol,
            p.0.lang,
            p.0.fields,
            p.0.limit,
            p.0.cursor,
        )
        .await
    }

    #[tool(
        description = "Fetch RSS/Atom news from financial publishers (Bloomberg, WSJ, MarketWatch, FT, SEC, etc.)."
    )]
    async fn get_feeds(&self, p: Parameters<FeedsParams>) -> Result<CallToolResult, McpError> {
        feeds::get_feeds(&self.schema, p.0.sources, p.0.fields, p.0.limit, p.0.cursor).await
    }

    #[tool(description = "Get market overview with major indices and currencies for a region.")]
    async fn get_market_summary(
        &self,
        p: Parameters<MarketSummaryParams>,
    ) -> Result<CallToolResult, McpError> {
        market::get_market_summary(&self.schema, p.0.region, p.0.lang, p.0.fields).await
    }

    #[tool(
        description = "Get the CNN Fear & Greed Index — market sentiment from extreme fear (0) to extreme greed (100)."
    )]
    async fn get_fear_and_greed(
        &self,
        p: Parameters<FearAndGreedParams>,
    ) -> Result<CallToolResult, McpError> {
        market::get_fear_and_greed(&self.schema, p.0.fields).await
    }

    #[tool(description = "Get currently trending stock tickers for a region.")]
    async fn get_trending(
        &self,
        p: Parameters<TrendingParams>,
    ) -> Result<CallToolResult, McpError> {
        market::get_trending(&self.schema, p.0.region, p.0.fields).await
    }

    #[tool(
        description = "Get world market indices (S&P 500, DAX, Nikkei, etc.), optionally filtered by region."
    )]
    async fn get_indices(&self, p: Parameters<IndicesParams>) -> Result<CallToolResult, McpError> {
        market::get_indices(&self.schema, p.0.region, p.0.fields).await
    }

    #[tool(description = "Get current market hours and open/closed status for a region.")]
    async fn get_market_hours(
        &self,
        p: Parameters<MarketHoursParams>,
    ) -> Result<CallToolResult, McpError> {
        market::get_market_hours(&self.schema, p.0.region, p.0.fields).await
    }

    #[tool(
        description = "Get comprehensive sector data (overview, performance, top companies, ETFs) for one of the 11 GICS sectors."
    )]
    async fn get_sector(&self, p: Parameters<SectorParams>) -> Result<CallToolResult, McpError> {
        market::get_sector(&self.schema, p.0.sector, p.0.lang, p.0.fields).await
    }

    #[tool(
        description = "Get comprehensive industry data (overview, performance, top companies) for a specific industry slug."
    )]
    async fn get_industry(
        &self,
        p: Parameters<IndustryParams>,
    ) -> Result<CallToolResult, McpError> {
        market::get_industry(&self.schema, p.0.industry, p.0.lang, p.0.fields).await
    }

    #[tool(
        description = "Get ownership data for a stock: major holders, institutional/fund ownership, or insider activity."
    )]
    async fn get_holders(&self, p: Parameters<HoldersParams>) -> Result<CallToolResult, McpError> {
        analysis::get_holders(
            &self.schema,
            p.0.symbol,
            p.0.holder_type,
            p.0.fields,
            p.0.limit,
            p.0.cursor,
        )
        .await
    }

    #[tool(
        description = "Get analyst data for a stock: recommendation trends, upgrades/downgrades, earnings estimates, or earnings history."
    )]
    async fn get_analysis(
        &self,
        p: Parameters<AnalysisParams>,
    ) -> Result<CallToolResult, McpError> {
        analysis::get_analysis(&self.schema, p.0.symbol, p.0.analysis_type, p.0.fields).await
    }

    #[tool(
        description = "Get dividend history for one or more dividend-paying stocks (comma-separated symbols). A single symbol returns paginated dividend history plus analytics (CAGR, average payment, payout count); multiple symbols return a batch of dividend histories plus per-symbol errors (no analytics for batch)."
    )]
    async fn get_dividends(
        &self,
        p: Parameters<DividendsParams>,
    ) -> Result<CallToolResult, McpError> {
        dividends::get_dividends(
            &self.schema,
            p.0.symbols,
            p.0.range,
            p.0.fields,
            p.0.limit,
            p.0.cursor,
        )
        .await
    }

    #[tool(
        description = "Get risk analytics: VaR (95/99%), Sharpe/Sortino/Calmar ratios, beta, and maximum drawdown for a symbol."
    )]
    async fn get_risk(&self, p: Parameters<RiskParams>) -> Result<CallToolResult, McpError> {
        risk::get_risk(
            &self.schema,
            p.0.symbol,
            p.0.interval,
            p.0.range,
            p.0.benchmark,
            p.0.fields,
        )
        .await
    }

    #[tool(
        description = "Get the options chain for a symbol. Provide an expiration timestamp to get a specific expiry, or omit for the nearest expiration."
    )]
    async fn get_options(&self, p: Parameters<OptionsParams>) -> Result<CallToolResult, McpError> {
        options::get_options(
            &self.schema,
            p.0.symbol,
            p.0.expiration,
            p.0.fields,
            p.0.limit,
            p.0.cursor,
        )
        .await
    }

    #[tool(
        description = "Get SEC EDGAR XBRL structured financial data (all reported accounting concepts) for a company. Requires EDGAR_EMAIL env var."
    )]
    async fn get_edgar_facts(
        &self,
        p: Parameters<EdgarFactsParams>,
    ) -> Result<CallToolResult, McpError> {
        edgar::get_edgar_facts(
            &self.schema,
            p.0.symbol,
            p.0.taxonomy,
            p.0.concepts,
            p.0.fields,
            p.0.limit,
            p.0.cursor,
        )
        .await
    }

    #[tool(
        description = "Get SEC filing history and company metadata from EDGAR (up to 1000 most recent filings). Requires EDGAR_EMAIL env var."
    )]
    async fn get_edgar_submissions(
        &self,
        p: Parameters<EdgarSubmissionsParams>,
    ) -> Result<CallToolResult, McpError> {
        edgar::get_edgar_submissions(&self.schema, p.0.symbol, p.0.fields, p.0.limit, p.0.cursor)
            .await
    }

    #[tool(
        description = "Full-text search across SEC EDGAR filings with optional form type and date filters. Requires EDGAR_EMAIL env var."
    )]
    async fn get_edgar_search(
        &self,
        p: Parameters<EdgarSearchParams>,
    ) -> Result<CallToolResult, McpError> {
        edgar::get_edgar_search(
            &self.schema,
            p.0.query,
            p.0.forms,
            p.0.start_date,
            p.0.end_date,
            p.0.from,
            p.0.size,
        )
        .await
    }

    #[tool(
        description = "Get earnings call transcripts for a company. Returns paragraph-by-paragraph text (speaker, timestamp, text), paginated via paragraph_limit/paragraph_cursor since a full call can be tens of thousands of tokens."
    )]
    async fn get_transcripts(
        &self,
        p: Parameters<TranscriptsParams>,
    ) -> Result<CallToolResult, McpError> {
        transcripts::get_transcripts(
            &self.schema,
            p.0.symbol,
            p.0.limit,
            p.0.lang,
            p.0.fields,
            p.0.paragraph_limit,
            p.0.paragraph_cursor,
        )
        .await
    }

    #[tool(
        description = "Get FRED macroeconomic time series data (e.g., FEDFUNDS, CPIAUCSL, GDP, UNRATE). Requires FRED_API_KEY env var."
    )]
    async fn get_fred_series(
        &self,
        p: Parameters<FredSeriesParams>,
    ) -> Result<CallToolResult, McpError> {
        fred::get_fred_series(&self.schema, p.0.id, p.0.fields, p.0.limit, p.0.cursor).await
    }

    #[tool(
        description = "Get US Treasury yield curve data (1m through 30y) for a given year. No API key required."
    )]
    async fn get_treasury_yields(
        &self,
        p: Parameters<TreasuryYieldsParams>,
    ) -> Result<CallToolResult, McpError> {
        fred::get_treasury_yields(&self.schema, p.0.year, p.0.fields, p.0.limit, p.0.cursor).await
    }

    #[tool(
        description = "Get top cryptocurrency coins by market cap from CoinGecko (no API key required)."
    )]
    async fn get_crypto(&self, p: Parameters<CryptoParams>) -> Result<CallToolResult, McpError> {
        crypto::get_crypto_coins(
            &self.schema,
            p.0.count,
            p.0.vs_currency,
            p.0.fields,
            p.0.limit,
            p.0.cursor,
        )
        .await
    }
}
