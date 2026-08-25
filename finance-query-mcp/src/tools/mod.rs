pub mod analysis;
#[cfg(feature = "backtesting")]
pub mod backtest;
pub mod calendar;
pub mod chart;
pub mod commodity_futures;
pub mod crypto;
pub mod dividends;
pub mod edgar;
pub mod feeds;
pub mod filings;
pub mod financials;
pub mod forex;
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
use finance_query::{
    Frequency, IndicesRegion, Industry, Interval, LookupType, Region, Screener, Sector,
    StatementType, TimeRange,
};
use finance_query_server::graphql::FinanceSchema;
use finance_query_server::params::{
    AnalysisType, FilingSectionFormParam, HolderType, MarketCalendarKindParam, Quarter,
};
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
    #[schemars(with = "String")]
    pub range: Option<TimeRange>,
    /// Comma-separated list of GraphQL field names to include; omitted = all fields
    pub fields: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct MarketCalendarParams {
    /// Which market-wide calendar to fetch: earnings | ipo | dividend | split | economic | market-holiday | market-status
    #[schemars(with = "String")]
    pub kind: MarketCalendarKindParam,
    /// Start date (YYYY-MM-DD); ignored by market-holiday/market-status
    pub from: Option<String>,
    /// End date (YYYY-MM-DD); ignored by market-holiday/market-status
    pub to: Option<String>,
    /// Comma-separated list of GraphQL field names to include; omitted = all fields
    pub fields: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct CommodityParams {
    /// Commodity symbol (e.g. "GCUSD" for gold, "CLUSD" for crude oil)
    pub symbol: String,
    /// Comma-separated list of GraphQL field names to include; omitted = all fields
    pub fields: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct FuturesParams {
    /// Futures contract symbol (e.g. "ESM26" for E-mini S&P June 2026)
    pub symbol: String,
    /// Comma-separated list of GraphQL field names to include; omitted = all fields
    pub fields: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ForexParams {
    /// Base currency code (e.g. "USD")
    pub from: String,
    /// Quote currency code (e.g. "EUR")
    pub to: String,
    /// Comma-separated list of GraphQL field names to include; omitted = all fields
    pub fields: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct IndexConstituentsParams {
    /// Index symbol (e.g. "^GSPC" for the S&P 500; Wikipedia-backed, S&P 500 only)
    pub symbol: String,
    /// Comma-separated list of GraphQL field names to include; omitted = all fields
    pub fields: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ChartParams {
    /// One or more comma-separated ticker symbols (e.g., "AAPL" or "AAPL,MSFT,GOOG")
    pub symbols: String,
    /// Candle interval: 1m|5m|15m|30m|1h|1d|1wk|1mo|3mo (default: 1d)
    #[schemars(with = "String")]
    pub interval: Option<Interval>,
    /// Time range: 1d|5d|1mo|3mo|6mo|1y|2y|5y|10y|ytd|max (default: 1mo). Ignored when `start` is set.
    #[schemars(with = "String")]
    pub range: Option<TimeRange>,
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
    #[schemars(with = "String")]
    pub statement: StatementType,
    /// Reporting frequency: annual | quarterly (default: annual)
    #[schemars(with = "String")]
    pub frequency: Option<Frequency>,
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
    #[schemars(with = "String")]
    pub interval: Option<Interval>,
    /// Time range: 1mo|3mo|6mo|1y|2y|5y (default: 1y)
    #[schemars(with = "String")]
    pub range: Option<TimeRange>,
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
    #[schemars(with = "String")]
    pub screener_type: Screener,
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
    #[schemars(with = "String")]
    pub region: Option<Region>,
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
pub struct CongressionalTradesParams {
    /// Stock ticker symbol
    pub symbol: String,
    /// Comma-separated list of GraphQL field names to include; omitted = all fields
    pub fields: Option<String>,
    /// Maximum entries per page; omitted = curated default (25)
    pub limit: Option<u32>,
    /// Opaque continuation token from a previous response's `pageInfo.endCursor`; omitted = first page
    pub cursor: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct FailsToDeliverParams {
    /// Stock ticker symbol
    pub symbol: String,
    /// Comma-separated list of GraphQL field names to include; omitted = all fields
    pub fields: Option<String>,
    /// Maximum entries per page; omitted = curated default (25)
    pub limit: Option<u32>,
    /// Opaque continuation token from a previous response's `pageInfo.endCursor`; omitted = first page
    pub cursor: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct FilingSectionsParams {
    /// Stock ticker symbol
    pub symbol: String,
    /// SEC accession number, e.g. "0000320193-24-000123"
    pub accession_number: String,
    /// Which filing form to fetch sectioned text for: ten-k | eight-k
    #[schemars(with = "String")]
    pub form: FilingSectionFormParam,
    /// Comma-separated list of GraphQL field names to include; omitted = all fields
    pub fields: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct RiskFactorsParams {
    /// Stock ticker symbol
    pub symbol: String,
    /// Comma-separated list of GraphQL field names to include; omitted = all fields
    pub fields: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct PressReleasesParams {
    /// Stock ticker symbol
    pub symbol: String,
    /// Maximum number of releases to return (default: 10)
    pub limit: Option<u32>,
    /// Comma-separated list of GraphQL field names to include; omitted = all fields
    pub fields: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct DividendsParams {
    /// One or more comma-separated ticker symbols (e.g., "AAPL" or "AAPL,KO,JNJ")
    pub symbols: String,
    /// Time range: 1y|2y|5y|10y|max (default: max)
    #[schemars(with = "String")]
    pub range: Option<TimeRange>,
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
    #[schemars(with = "String")]
    pub interval: Option<Interval>,
    /// Time range: 1y|2y|5y (default: 1y)
    #[schemars(with = "String")]
    pub range: Option<TimeRange>,
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

#[cfg(feature = "backtesting")]
#[derive(Deserialize, JsonSchema)]
pub struct RunBacktestParams {
    /// Stock ticker symbol (e.g., "AAPL")
    pub symbol: String,
    /// Prebuilt strategy: sma_crossover | rsi_reversal | macd_signal | bollinger_mean_reversion | supertrend_follow | donchian_breakout
    pub strategy: String,
    /// Candle interval: 1m|5m|15m|30m|1h|1d|1wk|1mo|3mo (default: 1d)
    pub interval: Option<String>,
    /// Time range: 1d|5d|1mo|3mo|6mo|1y|2y|5y|10y|ytd|max (default: 1y)
    pub range: Option<String>,

    // ── Strategy parameters (only fields relevant to `strategy` are used) ──
    /// Fast SMA/EMA period (sma_crossover, macd_signal)
    pub fast_period: Option<u32>,
    /// Slow SMA/EMA period (sma_crossover, macd_signal)
    pub slow_period: Option<u32>,
    /// Generic lookback period (rsi_reversal, bollinger_mean_reversion, supertrend_follow, donchian_breakout)
    pub period: Option<u32>,
    /// MACD signal-line period (macd_signal only, default: 9)
    pub signal_period: Option<u32>,
    /// Bollinger Bands standard-deviation multiplier (bollinger_mean_reversion only, default: 2.0)
    pub std_dev: Option<f64>,
    /// SuperTrend ATR multiplier (supertrend_follow only, default: 3.0)
    pub multiplier: Option<f64>,
    /// RSI oversold threshold (rsi_reversal only, default: 30.0)
    pub oversold: Option<f64>,
    /// RSI overbought threshold (rsi_reversal only, default: 70.0)
    pub overbought: Option<f64>,
    /// Exit at middle band/channel instead of the opposite band (bollinger_mean_reversion, donchian_breakout)
    pub exit_at_middle: Option<bool>,

    // ── Backtest configuration (all optional; defaults match the library's) ──
    /// Starting portfolio capital (default: 10000.0)
    pub initial_capital: Option<f64>,
    /// Commission as a fraction of trade value (default: 0.001)
    pub commission_pct: Option<f64>,
    /// Slippage as a fraction of price (default: 0.001)
    pub slippage_pct: Option<f64>,
    /// Fraction of available capital used per trade (default: 1.0)
    pub position_size_pct: Option<f64>,
    /// Allow short selling (default: false)
    pub allow_short: Option<bool>,
    /// Stop-loss as a fraction of entry price; auto-exit if loss exceeds this
    pub stop_loss_pct: Option<f64>,
    /// Take-profit as a fraction of entry price; auto-exit if profit exceeds this
    pub take_profit_pct: Option<f64>,

    // ── Pagination (equity curve and trade list paginate independently) ──
    /// Maximum equity-curve points per page; omitted = curated default (25)
    pub equity_limit: Option<u32>,
    /// Opaque continuation token for the equity curve from a previous response's `equityCurve.pageInfo.endCursor`
    pub equity_cursor: Option<String>,
    /// Maximum trades per page; omitted = curated default (25)
    pub trades_limit: Option<u32>,
    /// Opaque continuation token for the trade list from a previous response's `trades.pageInfo.endCursor`
    pub trades_cursor: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct CryptoNewsParams {
    /// Overall cap on articles fetched (default: 20)
    pub count: Option<u32>,
    /// Comma-separated list of GraphQL field names to include; omitted = curated default set
    pub fields: Option<String>,
    /// Maximum articles per page; omitted = curated default (25)
    pub limit: Option<u32>,
    /// Opaque continuation token from a previous response's `pageInfo.endCursor`; omitted = first page
    pub cursor: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ForexNewsParams {
    /// Overall cap on articles fetched (default: 20)
    pub count: Option<u32>,
    /// Comma-separated list of GraphQL field names to include; omitted = curated default set
    pub fields: Option<String>,
    /// Maximum articles per page; omitted = curated default (25)
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
    #[schemars(with = "String")]
    pub range: Option<TimeRange>,
    /// Comma-separated list of GraphQL field names to include; omitted = curated default set
    pub fields: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct TrendingParams {
    /// Region code: US|GB|DE|CA|AU|FR|IN|CN|HK|BR|TW|SG (default: US)
    #[schemars(with = "String")]
    pub region: Option<Region>,
    /// Comma-separated list of GraphQL field names to include; omitted = curated default set
    pub fields: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct IndicesParams {
    /// Region: americas|europe|asia-pacific|middle-east-africa|currencies (default: all)
    #[schemars(with = "String")]
    pub region: Option<IndicesRegion>,
    /// Comma-separated list of GraphQL field names to include; omitted = curated default set
    pub fields: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct MarketHoursParams {
    /// Region code: US|GB|DE|CA|AU|FR|IN|CN|HK|BR|TW|SG (default: US)
    #[schemars(with = "String")]
    pub region: Option<Region>,
    /// Comma-separated list of GraphQL field names to include; omitted = all fields
    pub fields: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct SectorPerformanceParams {
    /// Comma-separated list of GraphQL field names to include; omitted = all fields
    pub fields: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct SectorPeParams {
    /// Comma-separated list of GraphQL field names to include; omitted = all fields
    pub fields: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct SectorParams {
    /// Sector slug: technology|financial-services|consumer-cyclical|communication-services|
    /// healthcare|industrials|consumer-defensive|energy|basic-materials|real-estate|utilities
    #[schemars(with = "String")]
    pub sector: Sector,
    /// Target language for translated text fields (BCP 47, e.g. "ja", "zh-Hant"); English or omitted = no translation
    pub lang: Option<String>,
    /// Comma-separated list of GraphQL field names to include; omitted = curated default set
    pub fields: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct IndustryParams {
    /// Industry slug (e.g., semiconductors, biotechnology, banks-diversified)
    #[schemars(with = "String")]
    pub industry: Industry,
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
    #[schemars(with = "String")]
    pub holder_type: HolderType,
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
    #[schemars(with = "String")]
    pub analysis_type: AnalysisType,
    /// Comma-separated list of GraphQL field names to include; omitted = curated default set
    pub fields: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct CompanyProfileParams {
    /// Stock ticker symbol
    pub symbol: String,
    /// Comma-separated list of GraphQL field names to include; omitted = all fields
    pub fields: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct EarningsSurprisesParams {
    /// Stock ticker symbol
    pub symbol: String,
    /// Comma-separated list of GraphQL field names to include; omitted = all fields
    pub fields: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct RatingConsensusParams {
    /// Stock ticker symbol
    pub symbol: String,
    /// Comma-separated list of GraphQL field names to include; omitted = all fields
    pub fields: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct PriceTargetConsensusParams {
    /// Stock ticker symbol
    pub symbol: String,
    /// Comma-separated list of GraphQL field names to include; omitted = all fields
    pub fields: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct EtfProfileParams {
    /// ETF ticker symbol
    pub symbol: String,
    /// Comma-separated list of GraphQL field names to include; omitted = all fields
    pub fields: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct EarningsTranscriptParams {
    /// Stock ticker symbol
    pub symbol: String,
    /// Fiscal quarter (Q1, Q2, Q3, Q4). Required alongside `year` for providers
    /// with no "latest" shortcut (Alpha Vantage); Yahoo defaults to latest when omitted
    #[schemars(with = "String")]
    pub quarter: Option<Quarter>,
    /// Fiscal year. See `quarter`
    pub year: Option<i32>,
    /// Comma-separated list of GraphQL field names to include; omitted = all fields
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
    #[schemars(with = "String")]
    pub query_type: Option<LookupType>,
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
    #[schemars(with = "String")]
    pub interval: Option<Interval>,
    /// Time range: 1d|5d|1mo|3mo|6mo|1y|2y|5y|10y|ytd|max (default: 1mo)
    #[schemars(with = "String")]
    pub range: Option<TimeRange>,
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
    // Manual replica of the #[tool_handler]-generated call_tool; the macro
    // skips methods that already exist, so this is the one metrics chokepoint.
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
        #[allow(unused_mut)]
        let mut tool_router = Self::tool_router();
        // Each feature-gated tool group lives in its own `#[tool_router]`-annotated
        // impl block (see below) with its own named router, merged in here —
        // `#[tool_router]` naively scans every `#[tool]`-attributed method in its
        // impl block at the token level, ignoring any `#[cfg]` on the method
        // itself, so gating has to happen at the impl-block boundary instead.
        #[cfg(feature = "backtesting")]
        {
            tool_router += Self::backtest_tool_router();
        }
        Self {
            tool_router,
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
        description = "Get a market-wide event calendar over a date range: earnings, IPOs, dividends, splits, economic releases, market holidays, or live exchange open/closed status. Unlike get_calendar (per-symbol), this spans the whole market."
    )]
    async fn get_market_calendar(
        &self,
        p: Parameters<MarketCalendarParams>,
    ) -> Result<CallToolResult, McpError> {
        calendar::get_market_calendar(&self.schema, p.0.kind, p.0.from, p.0.to, p.0.fields).await
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

    #[tool(
        description = "Get congressional (House and Senate) trading disclosures for a symbol. Uses FMP when FMP_API_KEY is set, keyless House/Senate PTR filings otherwise."
    )]
    async fn get_congressional_trades(
        &self,
        p: Parameters<CongressionalTradesParams>,
    ) -> Result<CallToolResult, McpError> {
        filings::get_congressional_trades(
            &self.schema,
            p.0.symbol,
            p.0.fields,
            p.0.limit,
            p.0.cursor,
        )
        .await
    }

    #[tool(
        description = "Get fails-to-deliver records for a symbol. Uses FMP when FMP_API_KEY is set, keyless EDGAR otherwise."
    )]
    async fn get_fails_to_deliver(
        &self,
        p: Parameters<FailsToDeliverParams>,
    ) -> Result<CallToolResult, McpError> {
        filings::get_fails_to_deliver(&self.schema, p.0.symbol, p.0.fields, p.0.limit, p.0.cursor)
            .await
    }

    #[tool(
        description = "Get sectioned text of one SEC filing by accession number (10-K or 8-K). Routes through EDGAR (best-effort HTML extraction) or Polygon when configured."
    )]
    async fn get_filing_sections(
        &self,
        p: Parameters<FilingSectionsParams>,
    ) -> Result<CallToolResult, McpError> {
        filings::get_filing_sections(
            &self.schema,
            p.0.symbol,
            p.0.accession_number,
            p.0.form,
            p.0.fields,
        )
        .await
    }

    #[tool(
        description = "Get risk factors extracted from a symbol's SEC filings. Routes through EDGAR (best-effort HTML extraction) or Polygon when configured."
    )]
    async fn get_risk_factors(
        &self,
        p: Parameters<RiskFactorsParams>,
    ) -> Result<CallToolResult, McpError> {
        filings::get_risk_factors(&self.schema, p.0.symbol, p.0.fields).await
    }

    #[tool(
        description = "Get a company's own press releases, distinct from get_news (press coverage). Routes through EDGAR 8-K exhibits, falling back to FMP/Alpha Vantage when configured."
    )]
    async fn get_press_releases(
        &self,
        p: Parameters<PressReleasesParams>,
    ) -> Result<CallToolResult, McpError> {
        news::get_press_releases(&self.schema, p.0.symbol, p.0.limit, p.0.fields).await
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

    #[tool(
        description = "Get a commodity's current quote (e.g. gold, silver, crude oil), provider-routed (Yahoo, keyless)."
    )]
    async fn get_commodity(
        &self,
        p: Parameters<CommodityParams>,
    ) -> Result<CallToolResult, McpError> {
        commodity_futures::get_commodity(&self.schema, p.0.symbol, p.0.fields).await
    }

    #[tool(
        description = "Get a futures contract's current quote, provider-routed (Yahoo, keyless)."
    )]
    async fn get_futures(&self, p: Parameters<FuturesParams>) -> Result<CallToolResult, McpError> {
        commodity_futures::get_futures(&self.schema, p.0.symbol, p.0.fields).await
    }

    #[tool(
        description = "Get an index's current constituent list, provider-routed (Wikipedia, S&P 500 only)."
    )]
    async fn get_index_constituents(
        &self,
        p: Parameters<IndexConstituentsParams>,
    ) -> Result<CallToolResult, McpError> {
        commodity_futures::get_index_constituents(&self.schema, p.0.symbol, p.0.fields).await
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
        description = "Get aggregate performance for every market sector, provider-routed (Yahoo screener fan-out, keyless). Distinct from get_sector (per-sector Yahoo-only shortcut)."
    )]
    async fn get_sector_performance(
        &self,
        p: Parameters<SectorPerformanceParams>,
    ) -> Result<CallToolResult, McpError> {
        market::get_sector_performance(&self.schema, p.0.fields).await
    }

    #[tool(
        description = "Get price/earnings ratios by market sector, provider-routed (Yahoo screener fan-out, keyless)."
    )]
    async fn get_sector_pe(
        &self,
        p: Parameters<SectorPeParams>,
    ) -> Result<CallToolResult, McpError> {
        market::get_sector_pe(&self.schema, p.0.fields).await
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
        description = "Get a company's identity/classification profile (name, description, asset type, exchange, currency, country, sector, industry, market capitalization)."
    )]
    async fn get_company_profile(
        &self,
        p: Parameters<CompanyProfileParams>,
    ) -> Result<CallToolResult, McpError> {
        analysis::get_company_profile(&self.schema, p.0.symbol, p.0.fields).await
    }

    #[tool(description = "Get a stock's earnings-surprise history (actual vs. estimated EPS).")]
    async fn get_earnings_surprises(
        &self,
        p: Parameters<EarningsSurprisesParams>,
    ) -> Result<CallToolResult, McpError> {
        analysis::get_earnings_surprises(&self.schema, p.0.symbol, p.0.fields).await
    }

    #[tool(
        description = "Get a stock's consensus analyst rating rollup (strong buy/buy/hold/sell/strong sell counts and a headline consensus label)."
    )]
    async fn get_rating_consensus(
        &self,
        p: Parameters<RatingConsensusParams>,
    ) -> Result<CallToolResult, McpError> {
        analysis::get_rating_consensus(&self.schema, p.0.symbol, p.0.fields).await
    }

    #[tool(description = "Get a stock's consensus analyst price target (high/low/mean/median).")]
    async fn get_price_target_consensus(
        &self,
        p: Parameters<PriceTargetConsensusParams>,
    ) -> Result<CallToolResult, McpError> {
        analysis::get_price_target_consensus(&self.schema, p.0.symbol, p.0.fields).await
    }

    #[tool(
        description = "Get an ETF's profile and holdings (net assets, expense ratio, sector/country weightings, top holdings)."
    )]
    async fn get_etf_profile(
        &self,
        p: Parameters<EtfProfileParams>,
    ) -> Result<CallToolResult, McpError> {
        analysis::get_etf_profile(&self.schema, p.0.symbol, p.0.fields).await
    }

    #[tool(
        description = "Get an earnings call transcript for a symbol. Provide quarter and year for a specific call, or omit both for the latest (Yahoo only)."
    )]
    async fn get_earnings_transcript(
        &self,
        p: Parameters<EarningsTranscriptParams>,
    ) -> Result<CallToolResult, McpError> {
        analysis::get_earnings_transcript(
            &self.schema,
            p.0.symbol,
            p.0.quarter,
            p.0.year,
            p.0.fields,
        )
        .await
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

    #[tool(description = "Get market-wide crypto news (currently FMP only, requires FMP_API_KEY).")]
    async fn get_crypto_news(
        &self,
        p: Parameters<CryptoNewsParams>,
    ) -> Result<CallToolResult, McpError> {
        crypto::get_crypto_news(&self.schema, p.0.count, p.0.fields, p.0.limit, p.0.cursor).await
    }

    #[tool(description = "Get market-wide forex news (currently FMP only, requires FMP_API_KEY).")]
    async fn get_forex_news(
        &self,
        p: Parameters<ForexNewsParams>,
    ) -> Result<CallToolResult, McpError> {
        forex::get_forex_news(&self.schema, p.0.count, p.0.fields, p.0.limit, p.0.cursor).await
    }
}

// Each feature-gated tool group below lives in its own `#[tool_router]`
// impl block with a distinctly-named router (merged into `Self::new`'s
// `tool_router` above). `#[tool_router]` scans every `#[tool]`-attributed
// method in ITS OWN impl block at the raw-token level, before any `#[cfg]`
// on that method is evaluated — so putting a `#[cfg(feature = ...)]`
// method directly inside the main router-producing impl block above (as a
// sibling to always-on tools) does NOT prevent it from being referenced by
// the generated router when the feature is off; the whole impl block must
// be the cfg boundary instead.

#[cfg(feature = "backtesting")]
#[tool_router(router = backtest_tool_router)]
impl FinanceTools {
    #[tool(
        description = "Run a backtest of a prebuilt trading strategy (SMA/RSI/MACD/Bollinger/SuperTrend/Donchian) against a symbol's historical data. Returns summary performance metrics, a paginated equity curve, and a paginated trade list."
    )]
    async fn run_backtest(
        &self,
        p: Parameters<RunBacktestParams>,
    ) -> Result<CallToolResult, McpError> {
        backtest::run_backtest(p.0).await
    }
}

// ── Param-struct deserialization ─────────────────────────────────────────────
//
// Every `#[tool]` param struct must (a) deserialize when only its required
// fields are present, defaulting every `Option<T>` to `None`, and (b)
// preserve every field's value when the caller supplies all of them. These
// are the two contracts an LLM caller actually relies on — no network, no
// schema execution, just serde against canned JSON.

#[cfg(test)]
mod param_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn edgar_submissions_params_defaults_optionals_to_none() {
        let p: EdgarSubmissionsParams = serde_json::from_value(json!({"symbol": "AAPL"})).unwrap();
        assert_eq!(p.symbol, "AAPL");
        assert_eq!(p.fields, None);
        assert_eq!(p.limit, None);
        assert_eq!(p.cursor, None);
    }

    #[test]
    fn edgar_submissions_params_preserves_all_fields() {
        let p: EdgarSubmissionsParams = serde_json::from_value(json!({
            "symbol": "AAPL", "fields": "formType,filingDate", "limit": 10, "cursor": "abc"
        }))
        .unwrap();
        assert_eq!(p.fields, Some("formType,filingDate".to_string()));
        assert_eq!(p.limit, Some(10));
        assert_eq!(p.cursor, Some("abc".to_string()));
    }

    #[test]
    fn edgar_facts_params_defaults_optionals_to_none() {
        let p: EdgarFactsParams = serde_json::from_value(json!({"symbol": "AAPL"})).unwrap();
        assert_eq!(p.symbol, "AAPL");
        assert_eq!(p.taxonomy, None);
        assert_eq!(p.concepts, None);
        assert_eq!(p.fields, None);
        assert_eq!(p.limit, None);
        assert_eq!(p.cursor, None);
    }

    #[test]
    fn edgar_facts_params_preserves_all_fields() {
        let p: EdgarFactsParams = serde_json::from_value(json!({
            "symbol": "AAPL", "taxonomy": "ifrs-full", "concepts": "Revenues,Assets",
            "fields": "concept,unit", "limit": 5, "cursor": "xyz"
        }))
        .unwrap();
        assert_eq!(p.taxonomy, Some("ifrs-full".to_string()));
        assert_eq!(p.concepts, Some("Revenues,Assets".to_string()));
        assert_eq!(p.limit, Some(5));
    }

    #[test]
    fn symbols_params_defaults_optionals_to_none() {
        let p: SymbolsParams = serde_json::from_value(json!({"symbols": "AAPL,MSFT"})).unwrap();
        assert_eq!(p.symbols, "AAPL,MSFT");
        assert_eq!(p.lang, None);
        assert_eq!(p.fields, None);
        assert_eq!(p.limit, None);
        assert_eq!(p.cursor, None);
    }

    #[test]
    fn symbols_params_preserves_all_fields() {
        let p: SymbolsParams = serde_json::from_value(json!({
            "symbols": "AAPL", "lang": "ja", "fields": "symbol,shortName", "limit": 25, "cursor": "c1"
        }))
        .unwrap();
        assert_eq!(p.lang, Some("ja".to_string()));
        assert_eq!(p.fields, Some("symbol,shortName".to_string()));
    }

    #[test]
    fn calendar_params_defaults_optionals_to_none() {
        let p: CalendarParams = serde_json::from_value(json!({"symbols": "AAPL,MSFT"})).unwrap();
        assert_eq!(p.symbols, "AAPL,MSFT");
        assert_eq!(p.range, None);
        assert_eq!(p.fields, None);
    }

    #[test]
    fn chart_params_defaults_optionals_to_none() {
        let p: ChartParams = serde_json::from_value(json!({"symbols": "AAPL"})).unwrap();
        assert_eq!(p.interval, None);
        assert_eq!(p.range, None);
        assert_eq!(p.start, None);
        assert_eq!(p.end, None);
        assert_eq!(p.fields, None);
        assert_eq!(p.limit, None);
        assert_eq!(p.cursor, None);
    }

    #[test]
    fn chart_params_preserves_all_fields() {
        let p: ChartParams = serde_json::from_value(json!({
            "symbols": "AAPL", "interval": "1d", "range": "1mo", "start": 1_700_000_000,
            "end": 1_700_100_000, "fields": "meta,candles", "limit": 25, "cursor": "c1"
        }))
        .unwrap();
        assert_eq!(p.start, Some(1_700_000_000));
        assert_eq!(p.end, Some(1_700_100_000));
    }

    #[test]
    fn financials_params_requires_statement() {
        let err = serde_json::from_value::<FinancialsParams>(json!({"symbols": "AAPL"}));
        assert!(err.is_err());
    }

    #[test]
    fn financials_params_defaults_optionals_to_none() {
        let p: FinancialsParams =
            serde_json::from_value(json!({"symbols": "AAPL", "statement": "income"})).unwrap();
        assert_eq!(p.statement, StatementType::Income);
        assert_eq!(p.frequency, None);
        assert_eq!(p.metrics, None);
        assert_eq!(p.fields, None);
    }

    #[test]
    fn financials_params_preserves_metrics_and_fields() {
        let p: FinancialsParams = serde_json::from_value(json!({
            "symbols": "AAPL", "statement": "balance", "frequency": "quarterly",
            "metrics": "totalRevenue,netIncome", "fields": "reportDate"
        }))
        .unwrap();
        assert_eq!(p.metrics, Some("totalRevenue,netIncome".to_string()));
        assert_eq!(p.frequency, Some(Frequency::Quarterly));
    }

    #[test]
    fn indicators_params_defaults_optionals_to_none() {
        let p: IndicatorsParams = serde_json::from_value(json!({"symbols": "AAPL"})).unwrap();
        assert_eq!(p.interval, None);
        assert_eq!(p.range, None);
        assert_eq!(p.limit, None);
        assert_eq!(p.cursor, None);
    }

    #[test]
    fn search_params_requires_query() {
        let err = serde_json::from_value::<SearchParams>(json!({}));
        assert!(err.is_err());
    }

    #[test]
    fn search_params_defaults_optionals_to_none() {
        let p: SearchParams = serde_json::from_value(json!({"query": "Apple"})).unwrap();
        assert_eq!(p.query, "Apple");
        assert_eq!(p.lang, None);
        assert_eq!(p.limit, None);
        assert_eq!(p.cursor, None);
    }

    #[test]
    fn screener_params_requires_screener_type() {
        let err = serde_json::from_value::<ScreenerParams>(json!({}));
        assert!(err.is_err());
    }

    #[test]
    fn screener_params_defaults_optionals_to_none() {
        let p: ScreenerParams =
            serde_json::from_value(json!({"screener_type": "day-gainers"})).unwrap();
        assert_eq!(p.screener_type, Screener::DayGainers);
        assert_eq!(p.count, None);
        assert_eq!(p.fields, None);
    }

    #[test]
    fn news_params_allows_omitting_symbol_for_general_news() {
        let p: NewsParams = serde_json::from_value(json!({})).unwrap();
        assert_eq!(p.symbol, None);
        assert_eq!(p.lang, None);
        assert_eq!(p.fields, None);
        assert_eq!(p.limit, None);
        assert_eq!(p.cursor, None);
    }

    #[test]
    fn news_params_preserves_symbol_when_present() {
        let p: NewsParams = serde_json::from_value(json!({"symbol": "AAPL"})).unwrap();
        assert_eq!(p.symbol, Some("AAPL".to_string()));
    }

    #[test]
    fn market_summary_params_all_optional() {
        let p: MarketSummaryParams = serde_json::from_value(json!({})).unwrap();
        assert_eq!(p.region, None);
        assert_eq!(p.lang, None);
        assert_eq!(p.fields, None);
    }

    #[test]
    fn fear_and_greed_params_all_optional() {
        let p: FearAndGreedParams = serde_json::from_value(json!({})).unwrap();
        assert_eq!(p.fields, None);
    }

    #[test]
    fn dividends_params_defaults_optionals_to_none() {
        let p: DividendsParams = serde_json::from_value(json!({"symbols": "AAPL,KO"})).unwrap();
        assert_eq!(p.range, None);
        assert_eq!(p.limit, None);
        assert_eq!(p.cursor, None);
    }

    #[test]
    fn risk_params_defaults_optionals_to_none() {
        let p: RiskParams = serde_json::from_value(json!({"symbol": "AAPL"})).unwrap();
        assert_eq!(p.interval, None);
        assert_eq!(p.range, None);
        assert_eq!(p.benchmark, None);
        assert_eq!(p.fields, None);
    }

    #[test]
    fn risk_params_preserves_benchmark() {
        let p: RiskParams =
            serde_json::from_value(json!({"symbol": "AAPL", "benchmark": "SPY"})).unwrap();
        assert_eq!(p.benchmark, Some("SPY".to_string()));
    }

    #[test]
    fn crypto_params_all_optional() {
        let p: CryptoParams = serde_json::from_value(json!({})).unwrap();
        assert_eq!(p.count, None);
        assert_eq!(p.vs_currency, None);
        assert_eq!(p.fields, None);
        assert_eq!(p.limit, None);
        assert_eq!(p.cursor, None);
    }

    #[cfg(feature = "backtesting")]
    #[test]
    fn run_backtest_params_requires_symbol_and_strategy() {
        let err = serde_json::from_value::<RunBacktestParams>(json!({"symbol": "AAPL"}));
        assert!(err.is_err());
    }

    #[cfg(feature = "backtesting")]
    #[test]
    fn run_backtest_params_defaults_optionals_to_none() {
        let p: RunBacktestParams = serde_json::from_value(json!({
            "symbol": "AAPL", "strategy": "sma_crossover"
        }))
        .unwrap();
        assert_eq!(p.symbol, "AAPL");
        assert_eq!(p.strategy, "sma_crossover");
        assert_eq!(p.interval, None);
        assert_eq!(p.range, None);
        assert_eq!(p.fast_period, None);
        assert_eq!(p.position_size_pct, None);
        assert_eq!(p.equity_limit, None);
        assert_eq!(p.equity_cursor, None);
        assert_eq!(p.trades_limit, None);
        assert_eq!(p.trades_cursor, None);
    }

    #[cfg(feature = "backtesting")]
    #[test]
    fn run_backtest_params_preserves_all_fields() {
        let p: RunBacktestParams = serde_json::from_value(json!({
            "symbol": "AAPL", "strategy": "sma_crossover", "interval": "1d", "range": "1y",
            "fast_period": 10, "slow_period": 20, "position_size_pct": 0.5,
            "equity_limit": 10, "equity_cursor": "5", "trades_limit": 5, "trades_cursor": "2"
        }))
        .unwrap();
        assert_eq!(p.fast_period, Some(10));
        assert_eq!(p.slow_period, Some(20));
        assert_eq!(p.position_size_pct, Some(0.5));
        assert_eq!(p.equity_limit, Some(10));
        assert_eq!(p.equity_cursor, Some("5".to_string()));
        assert_eq!(p.trades_limit, Some(5));
        assert_eq!(p.trades_cursor, Some("2".to_string()));
    }

    #[test]
    fn options_params_defaults_optionals_to_none() {
        let p: OptionsParams = serde_json::from_value(json!({"symbol": "AAPL"})).unwrap();
        assert_eq!(p.expiration, None);
        assert_eq!(p.limit, None);
        assert_eq!(p.cursor, None);
    }

    #[test]
    fn recommendations_params_defaults_optionals_to_none() {
        let p: RecommendationsParams = serde_json::from_value(json!({"symbol": "AAPL"})).unwrap();
        assert_eq!(p.limit, None);
        assert_eq!(p.fields, None);
    }

    #[test]
    fn splits_params_defaults_optionals_to_none() {
        let p: SplitsParams = serde_json::from_value(json!({"symbol": "AAPL"})).unwrap();
        assert_eq!(p.range, None);
        assert_eq!(p.fields, None);
    }

    #[test]
    fn trending_params_all_optional() {
        let p: TrendingParams = serde_json::from_value(json!({})).unwrap();
        assert_eq!(p.region, None);
        assert_eq!(p.fields, None);
    }

    #[test]
    fn indices_params_all_optional() {
        let p: IndicesParams = serde_json::from_value(json!({})).unwrap();
        assert_eq!(p.region, None);
        assert_eq!(p.fields, None);
    }

    #[test]
    fn market_hours_params_all_optional() {
        let p: MarketHoursParams = serde_json::from_value(json!({})).unwrap();
        assert_eq!(p.region, None);
        assert_eq!(p.fields, None);
    }

    #[test]
    fn sector_params_requires_sector() {
        let err = serde_json::from_value::<SectorParams>(json!({}));
        assert!(err.is_err());
    }

    #[test]
    fn sector_params_defaults_optionals_to_none() {
        let p: SectorParams = serde_json::from_value(json!({"sector": "technology"})).unwrap();
        assert_eq!(p.lang, None);
        assert_eq!(p.fields, None);
    }

    #[test]
    fn industry_params_requires_industry() {
        let err = serde_json::from_value::<IndustryParams>(json!({}));
        assert!(err.is_err());
    }

    #[test]
    fn holders_params_requires_holder_type() {
        let err = serde_json::from_value::<HoldersParams>(json!({"symbol": "AAPL"}));
        assert!(err.is_err());
    }

    #[test]
    fn holders_params_defaults_optionals_to_none() {
        let p: HoldersParams =
            serde_json::from_value(json!({"symbol": "AAPL", "holder_type": "institutional"}))
                .unwrap();
        assert_eq!(p.limit, None);
        assert_eq!(p.cursor, None);
        assert_eq!(p.fields, None);
    }

    #[test]
    fn analysis_params_requires_analysis_type() {
        let err = serde_json::from_value::<AnalysisParams>(json!({"symbol": "AAPL"}));
        assert!(err.is_err());
    }

    #[test]
    fn fred_series_params_defaults_optionals_to_none() {
        let p: FredSeriesParams = serde_json::from_value(json!({"id": "FEDFUNDS"})).unwrap();
        assert_eq!(p.fields, None);
        assert_eq!(p.limit, None);
        assert_eq!(p.cursor, None);
    }

    #[test]
    fn treasury_yields_params_all_optional() {
        let p: TreasuryYieldsParams = serde_json::from_value(json!({})).unwrap();
        assert_eq!(p.year, None);
        assert_eq!(p.fields, None);
        assert_eq!(p.limit, None);
        assert_eq!(p.cursor, None);
    }

    #[test]
    fn transcripts_params_defaults_optionals_to_none() {
        let p: TranscriptsParams = serde_json::from_value(json!({"symbol": "AAPL"})).unwrap();
        assert_eq!(p.limit, None);
        assert_eq!(p.lang, None);
        assert_eq!(p.fields, None);
        assert_eq!(p.paragraph_limit, None);
        assert_eq!(p.paragraph_cursor, None);
    }

    #[test]
    fn feeds_params_all_optional() {
        let p: FeedsParams = serde_json::from_value(json!({})).unwrap();
        assert_eq!(p.sources, None);
        assert_eq!(p.fields, None);
        assert_eq!(p.limit, None);
        assert_eq!(p.cursor, None);
    }

    #[test]
    fn lookup_params_requires_query() {
        let err = serde_json::from_value::<LookupParams>(json!({}));
        assert!(err.is_err());
    }

    #[test]
    fn lookup_params_defaults_optionals_to_none() {
        let p: LookupParams = serde_json::from_value(json!({"query": "Apple"})).unwrap();
        assert_eq!(p.query_type, None);
        assert_eq!(p.lang, None);
        assert_eq!(p.fields, None);
        assert_eq!(p.logo, None);
    }

    #[test]
    fn lookup_params_preserves_logo_flag() {
        let p: LookupParams =
            serde_json::from_value(json!({"query": "Apple", "logo": true})).unwrap();
        assert_eq!(p.logo, Some(true));
    }

    #[test]
    fn batch_symbols_params_defaults_optionals_to_none() {
        let p: BatchSymbolsParams =
            serde_json::from_value(json!({"symbols": "AAPL,MSFT"})).unwrap();
        assert_eq!(p.interval, None);
        assert_eq!(p.range, None);
        assert_eq!(p.fields, None);
    }

    #[test]
    fn edgar_search_params_requires_query() {
        let err = serde_json::from_value::<EdgarSearchParams>(json!({}));
        assert!(err.is_err());
    }

    #[test]
    fn edgar_search_params_defaults_optionals_to_none() {
        let p: EdgarSearchParams = serde_json::from_value(json!({"query": "revenue"})).unwrap();
        assert_eq!(p.forms, None);
        assert_eq!(p.start_date, None);
        assert_eq!(p.end_date, None);
        assert_eq!(p.from, None);
        assert_eq!(p.size, None);
    }

    #[test]
    fn edgar_search_params_preserves_date_filters() {
        let p: EdgarSearchParams = serde_json::from_value(json!({
            "query": "revenue", "forms": "10-K,10-Q", "start_date": "2024-01-01",
            "end_date": "2024-12-31", "from": 0, "size": 50
        }))
        .unwrap();
        assert_eq!(p.forms, Some("10-K,10-Q".to_string()));
        assert_eq!(p.from, Some(0));
        assert_eq!(p.size, Some(50));
    }
}
