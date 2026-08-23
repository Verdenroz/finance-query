//! Route/spec reconciliation manifest (`cargo soothfast spec check`).

soothfast::bench_main!();

/// Get current quote and company data (price, market cap, PE ratio, 52-week range, etc.) for one or more stock symbols (comma-separated). A single symbol returns one quote object; multiple symbols return a paginated batch of quotes plus per-symbol errors.
///
/// Implements `tools::FinanceTools::get_quote`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "mcp-tools.json",
    operation = "get_quote",
    method = "TOOL",
    path = "get_quote",
    params = "SymbolsParams"
)]
fn route_mcp_get_quote() {}

/// Get similar stock recommendations and analyst ratings for a symbol.
///
/// Implements `tools::FinanceTools::get_recommendations`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "mcp-tools.json",
    operation = "get_recommendations",
    method = "TOOL",
    path = "get_recommendations",
    params = "RecommendationsParams"
)]
fn route_mcp_get_recommendations() {}

/// Get historical stock split history for a symbol.
///
/// Implements `tools::FinanceTools::get_splits`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "mcp-tools.json",
    operation = "get_splits",
    method = "TOOL",
    path = "get_splits",
    params = "SplitsParams"
)]
fn route_mcp_get_splits() {}

/// Get historical OHLCV candlestick chart data for one or more stock symbols (comma-separated). A single symbol supports start/end absolute timestamps and returns one chart; multiple symbols return a batch of charts plus per-symbol errors (interval/range only, no start/end).
///
/// Implements `tools::FinanceTools::get_chart`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "mcp-tools.json",
    operation = "get_chart",
    method = "TOOL",
    path = "get_chart",
    params = "ChartParams"
)]
fn route_mcp_get_chart() {}

/// Get a time-sorted calendar of upcoming financial events (earnings with estimates, ex-dividend and dividend-payment dates, options expirations, and — when FRED is configured — market-wide economic releases) across multiple symbols. Answers 'what's coming up for my portfolio?' in one call.
///
/// Implements `tools::FinanceTools::get_calendar`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "mcp-tools.json",
    operation = "get_calendar",
    method = "TOOL",
    path = "get_calendar",
    params = "CalendarParams"
)]
fn route_mcp_get_calendar() {}

/// Get a market-wide event calendar over a date range: earnings, IPOs, dividends, splits, economic releases, market holidays, or live exchange open/closed status. Unlike get_calendar (per-symbol), this spans the whole market.
///
/// Implements `tools::FinanceTools::get_market_calendar`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "mcp-tools.json",
    operation = "get_market_calendar",
    method = "TOOL",
    path = "get_market_calendar",
    params = "MarketCalendarParams"
)]
fn route_mcp_get_market_calendar() {}

/// Get lightweight close-price sparklines for multiple symbols. Faster and smaller than get_charts — use when you only need price direction/trend across many symbols.
///
/// Implements `tools::FinanceTools::get_spark`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "mcp-tools.json",
    operation = "get_spark",
    method = "TOOL",
    path = "get_spark",
    params = "BatchSymbolsParams"
)]
fn route_mcp_get_spark() {}

/// Get income statement, balance sheet, or cash flow statement for one or more stock symbols (comma-separated). A single symbol returns one statement; multiple symbols return a batch plus per-symbol errors.
///
/// Implements `tools::FinanceTools::get_financials`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "mcp-tools.json",
    operation = "get_financials",
    method = "TOOL",
    path = "get_financials",
    params = "FinancialsParams"
)]
fn route_mcp_get_financials() {}

/// Get all 42 technical analysis indicators (SMA, EMA, RSI, MACD, Bollinger Bands, Ichimoku, etc.) for one or more stock symbols (comma-separated). A single symbol returns one indicators object; multiple symbols return a paginated batch plus per-symbol errors.
///
/// Implements `tools::FinanceTools::get_indicators`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "mcp-tools.json",
    operation = "get_indicators",
    method = "TOOL",
    path = "get_indicators",
    params = "IndicatorsParams"
)]
fn route_mcp_get_indicators() {}

/// Search for stocks, ETFs, and companies by name or ticker symbol.
///
/// Implements `tools::FinanceTools::search`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "mcp-tools.json",
    operation = "search",
    method = "TOOL",
    path = "search",
    params = "SearchParams"
)]
fn route_mcp_search() {}

/// Discover tickers filtered by type (equity, ETF, mutual fund, index, future, currency, cryptocurrency).
///
/// Implements `tools::FinanceTools::lookup`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "mcp-tools.json",
    operation = "lookup",
    method = "TOOL",
    path = "lookup",
    params = "LookupParams"
)]
fn route_mcp_lookup() {}

/// Get results from a predefined stock screener (e.g., most-actives, day-gainers, undervalued-growth-stocks).
///
/// Implements `tools::FinanceTools::screener`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "mcp-tools.json",
    operation = "screener",
    method = "TOOL",
    path = "screener",
    params = "ScreenerParams"
)]
fn route_mcp_screener() {}

/// Get recent news. If a symbol is provided, returns news for that stock; otherwise returns general market news.
///
/// Implements `tools::FinanceTools::get_news`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "mcp-tools.json",
    operation = "get_news",
    method = "TOOL",
    path = "get_news",
    params = "NewsParams"
)]
fn route_mcp_get_news() {}

/// Fetch RSS/Atom news from financial publishers (Bloomberg, WSJ, MarketWatch, FT, SEC, etc.).
///
/// Implements `tools::FinanceTools::get_feeds`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "mcp-tools.json",
    operation = "get_feeds",
    method = "TOOL",
    path = "get_feeds",
    params = "FeedsParams"
)]
fn route_mcp_get_feeds() {}

/// Get market overview with major indices and currencies for a region.
///
/// Implements `tools::FinanceTools::get_market_summary`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "mcp-tools.json",
    operation = "get_market_summary",
    method = "TOOL",
    path = "get_market_summary",
    params = "MarketSummaryParams"
)]
fn route_mcp_get_market_summary() {}

/// Get the CNN Fear & Greed Index — market sentiment from extreme fear (0) to extreme greed (100).
///
/// Implements `tools::FinanceTools::get_fear_and_greed`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "mcp-tools.json",
    operation = "get_fear_and_greed",
    method = "TOOL",
    path = "get_fear_and_greed",
    params = "FearAndGreedParams"
)]
fn route_mcp_get_fear_and_greed() {}

/// Get currently trending stock tickers for a region.
///
/// Implements `tools::FinanceTools::get_trending`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "mcp-tools.json",
    operation = "get_trending",
    method = "TOOL",
    path = "get_trending",
    params = "TrendingParams"
)]
fn route_mcp_get_trending() {}

/// Get world market indices (S&P 500, DAX, Nikkei, etc.), optionally filtered by region.
///
/// Implements `tools::FinanceTools::get_indices`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "mcp-tools.json",
    operation = "get_indices",
    method = "TOOL",
    path = "get_indices",
    params = "IndicesParams"
)]
fn route_mcp_get_indices() {}

/// Get a commodity's current quote (e.g. gold, silver, crude oil), provider-routed (Yahoo, keyless).
///
/// Implements `tools::FinanceTools::get_commodity`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "mcp-tools.json",
    operation = "get_commodity",
    method = "TOOL",
    path = "get_commodity",
    params = "CommodityParams"
)]
fn route_mcp_get_commodity() {}

/// Get a futures contract's current quote, provider-routed (Yahoo, keyless).
///
/// Implements `tools::FinanceTools::get_futures`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "mcp-tools.json",
    operation = "get_futures",
    method = "TOOL",
    path = "get_futures",
    params = "FuturesParams"
)]
fn route_mcp_get_futures() {}

/// Get an index's current constituent list, provider-routed (Wikipedia, S&P 500 only).
///
/// Implements `tools::FinanceTools::get_index_constituents`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "mcp-tools.json",
    operation = "get_index_constituents",
    method = "TOOL",
    path = "get_index_constituents",
    params = "IndexConstituentsParams"
)]
fn route_mcp_get_index_constituents() {}

/// Get current market hours and open/closed status for a region.
///
/// Implements `tools::FinanceTools::get_market_hours`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "mcp-tools.json",
    operation = "get_market_hours",
    method = "TOOL",
    path = "get_market_hours",
    params = "MarketHoursParams"
)]
fn route_mcp_get_market_hours() {}

/// Get comprehensive sector data (overview, performance, top companies, ETFs) for one of the 11 GICS sectors.
///
/// Implements `tools::FinanceTools::get_sector`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "mcp-tools.json",
    operation = "get_sector",
    method = "TOOL",
    path = "get_sector",
    params = "SectorParams"
)]
fn route_mcp_get_sector() {}

/// Get aggregate performance for every market sector, provider-routed (Yahoo screener fan-out, keyless). Distinct from get_sector (per-sector Yahoo-only shortcut).
///
/// Implements `tools::FinanceTools::get_sector_performance`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "mcp-tools.json",
    operation = "get_sector_performance",
    method = "TOOL",
    path = "get_sector_performance",
    params = "SectorPerformanceParams"
)]
fn route_mcp_get_sector_performance() {}

/// Get price/earnings ratios by market sector, provider-routed (Yahoo screener fan-out, keyless).
///
/// Implements `tools::FinanceTools::get_sector_pe`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "mcp-tools.json",
    operation = "get_sector_pe",
    method = "TOOL",
    path = "get_sector_pe",
    params = "SectorPeParams"
)]
fn route_mcp_get_sector_pe() {}

/// Get comprehensive industry data (overview, performance, top companies) for a specific industry slug.
///
/// Implements `tools::FinanceTools::get_industry`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "mcp-tools.json",
    operation = "get_industry",
    method = "TOOL",
    path = "get_industry",
    params = "IndustryParams"
)]
fn route_mcp_get_industry() {}

/// Get ownership data for a stock: major holders, institutional/fund ownership, or insider activity.
///
/// Implements `tools::FinanceTools::get_holders`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "mcp-tools.json",
    operation = "get_holders",
    method = "TOOL",
    path = "get_holders",
    params = "HoldersParams"
)]
fn route_mcp_get_holders() {}

/// Get analyst data for a stock: recommendation trends, upgrades/downgrades, earnings estimates, or earnings history.
///
/// Implements `tools::FinanceTools::get_analysis`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "mcp-tools.json",
    operation = "get_analysis",
    method = "TOOL",
    path = "get_analysis",
    params = "AnalysisParams"
)]
fn route_mcp_get_analysis() {}

/// Get a stock's consensus analyst rating rollup (strong buy/buy/hold/sell/strong sell counts and a headline consensus label).
///
/// Implements `tools::FinanceTools::get_rating_consensus`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "mcp-tools.json",
    operation = "get_rating_consensus",
    method = "TOOL",
    path = "get_rating_consensus",
    params = "RatingConsensusParams"
)]
fn route_mcp_get_rating_consensus() {}

/// Get a stock's consensus analyst price target (high/low/mean/median).
///
/// Implements `tools::FinanceTools::get_price_target_consensus`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "mcp-tools.json",
    operation = "get_price_target_consensus",
    method = "TOOL",
    path = "get_price_target_consensus",
    params = "PriceTargetConsensusParams"
)]
fn route_mcp_get_price_target_consensus() {}

/// Get an ETF's profile and holdings (net assets, expense ratio, sector/country weightings, top holdings).
///
/// Implements `tools::FinanceTools::get_etf_profile`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "mcp-tools.json",
    operation = "get_etf_profile",
    method = "TOOL",
    path = "get_etf_profile",
    params = "EtfProfileParams"
)]
fn route_mcp_get_etf_profile() {}

/// Get dividend history for one or more dividend-paying stocks (comma-separated symbols). A single symbol returns paginated dividend history plus analytics (CAGR, average payment, payout count); multiple symbols return a batch of dividend histories plus per-symbol errors (no analytics for batch).
///
/// Implements `tools::FinanceTools::get_dividends`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "mcp-tools.json",
    operation = "get_dividends",
    method = "TOOL",
    path = "get_dividends",
    params = "DividendsParams"
)]
fn route_mcp_get_dividends() {}

/// Get risk analytics: VaR (95/99%), Sharpe/Sortino/Calmar ratios, beta, and maximum drawdown for a symbol.
///
/// Implements `tools::FinanceTools::get_risk`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "mcp-tools.json",
    operation = "get_risk",
    method = "TOOL",
    path = "get_risk",
    params = "RiskParams"
)]
fn route_mcp_get_risk() {}

/// Get sectioned text of one SEC filing by accession number (10-K or 8-K). Routes through EDGAR (best-effort HTML extraction) or Polygon when configured.
///
/// Implements `tools::FinanceTools::get_filing_sections`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "mcp-tools.json",
    operation = "get_filing_sections",
    method = "TOOL",
    path = "get_filing_sections",
    params = "FilingSectionsParams"
)]
fn route_mcp_get_filing_sections() {}

/// Get risk factors extracted from a symbol's SEC filings. Routes through EDGAR (best-effort HTML extraction) or Polygon when configured.
///
/// Implements `tools::FinanceTools::get_risk_factors`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "mcp-tools.json",
    operation = "get_risk_factors",
    method = "TOOL",
    path = "get_risk_factors",
    params = "RiskFactorsParams"
)]
fn route_mcp_get_risk_factors() {}

/// Get a company's own press releases, distinct from get_news (press coverage). Routes through EDGAR 8-K exhibits, falling back to FMP/Alpha Vantage when configured.
///
/// Implements `tools::FinanceTools::get_press_releases`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "mcp-tools.json",
    operation = "get_press_releases",
    method = "TOOL",
    path = "get_press_releases",
    params = "PressReleasesParams"
)]
fn route_mcp_get_press_releases() {}

/// Get the options chain for a symbol. Provide an expiration timestamp to get a specific expiry, or omit for the nearest expiration.
///
/// Implements `tools::FinanceTools::get_options`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "mcp-tools.json",
    operation = "get_options",
    method = "TOOL",
    path = "get_options",
    params = "OptionsParams"
)]
fn route_mcp_get_options() {}

/// Get SEC EDGAR XBRL structured financial data (all reported accounting concepts) for a company. Requires EDGAR_EMAIL env var.
///
/// Implements `tools::FinanceTools::get_edgar_facts`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "mcp-tools.json",
    operation = "get_edgar_facts",
    method = "TOOL",
    path = "get_edgar_facts",
    params = "EdgarFactsParams"
)]
fn route_mcp_get_edgar_facts() {}

/// Get SEC filing history and company metadata from EDGAR (up to 1000 most recent filings). Requires EDGAR_EMAIL env var.
///
/// Implements `tools::FinanceTools::get_edgar_submissions`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "mcp-tools.json",
    operation = "get_edgar_submissions",
    method = "TOOL",
    path = "get_edgar_submissions",
    params = "EdgarSubmissionsParams"
)]
fn route_mcp_get_edgar_submissions() {}

/// Full-text search across SEC EDGAR filings with optional form type and date filters. Requires EDGAR_EMAIL env var.
///
/// Implements `tools::FinanceTools::get_edgar_search`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "mcp-tools.json",
    operation = "get_edgar_search",
    method = "TOOL",
    path = "get_edgar_search",
    params = "EdgarSearchParams"
)]
fn route_mcp_get_edgar_search() {}

/// Get earnings call transcripts for a company. Returns paragraph-by-paragraph text (speaker, timestamp, text), paginated via paragraph_limit/paragraph_cursor since a full call can be tens of thousands of tokens.
///
/// Implements `tools::FinanceTools::get_transcripts`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "mcp-tools.json",
    operation = "get_transcripts",
    method = "TOOL",
    path = "get_transcripts",
    params = "TranscriptsParams"
)]
fn route_mcp_get_transcripts() {}

/// Get FRED macroeconomic time series data (e.g., FEDFUNDS, CPIAUCSL, GDP, UNRATE). Requires FRED_API_KEY env var.
///
/// Implements `tools::FinanceTools::get_fred_series`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "mcp-tools.json",
    operation = "get_fred_series",
    method = "TOOL",
    path = "get_fred_series",
    params = "FredSeriesParams"
)]
fn route_mcp_get_fred_series() {}

/// Get US Treasury yield curve data (1m through 30y) for a given year. No API key required.
///
/// Implements `tools::FinanceTools::get_treasury_yields`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "mcp-tools.json",
    operation = "get_treasury_yields",
    method = "TOOL",
    path = "get_treasury_yields",
    params = "TreasuryYieldsParams"
)]
fn route_mcp_get_treasury_yields() {}

/// Get top cryptocurrency coins by market cap from CoinGecko (no API key required).
///
/// Implements `tools::FinanceTools::get_crypto`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "mcp-tools.json",
    operation = "get_crypto",
    method = "TOOL",
    path = "get_crypto",
    params = "CryptoParams"
)]
fn route_mcp_get_crypto() {}
