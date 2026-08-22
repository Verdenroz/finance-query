//! Route/spec reconciliation manifest (`cargo soothfast spec check`).

soothfast::bench_main!();

// -- OpenAPI (server/openapi.yaml) ----------------------------------------

/// Get analysis data
///
/// Implements `handlers::analysis::get_analysis`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getAnalysis",
    method = "GET",
    path = "/v2/analysis/{symbol}/{type}",
    params = "AnalysisQuery",
    path_params = "type: AnalysisType",
    response = "AnalysisResponse"
)]
fn route_analysis_get_analysis() {}

/// Get upcoming financial event calendar
///
/// Implements `handlers::calendar::get_calendar`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getCalendar",
    method = "GET",
    path = "/v2/calendar",
    params = "CalendarQuery",
    response = "[GqlCalendarEvent]"
)]
fn route_calendar_get_calendar() {}

/// Get batch capital gains history
///
/// Implements `handlers::events::get_batch_capital_gains`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getBatchCapitalGains",
    method = "GET",
    path = "/v2/capital-gains",
    params = "BatchCapitalGainsQuery",
    response = "GqlCapitalGainsBatch"
)]
fn route_events_get_batch_capital_gains() {}

/// Get capital gains distribution history
///
/// Implements `handlers::events::get_capital_gains`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getCapitalGains",
    method = "GET",
    path = "/v2/capital-gains/{symbol}",
    params = "RangeQuery",
    response = "[GqlCapitalGain]"
)]
fn route_events_get_capital_gains() {}

/// Weekly CFTC Commitments of Traders positioning (keyless)
///
/// Implements `handlers::keyless::get_commitments_of_traders`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getCommitmentsOfTraders",
    method = "GET",
    path = "/v2/cftc/cot/{symbol}",
    params = "CotQuery",
    response = "GqlCommitmentsOfTraders"
)]
fn route_keyless_get_commitments_of_traders() {}

/// Get historical chart data
///
/// Implements `handlers::chart::get_chart`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getChart",
    method = "GET",
    path = "/v2/chart/{symbol}",
    params = "ChartQuery",
    response = "GqlChart"
)]
fn route_chart_get_chart() {}

/// Get batch historical chart data
///
/// Implements `handlers::chart::get_batch_charts`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getBatchCharts",
    method = "GET",
    path = "/v2/charts",
    params = "BatchChartsQuery",
    response = "GqlChartsBatch"
)]
fn route_chart_get_batch_charts() {}

/// Get company profile
///
/// Implements `handlers::analysis::get_company_profile`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getCompanyProfile",
    method = "GET",
    path = "/v2/company-profile/{symbol}",
    params = "AnalysisQuery",
    response = "GqlCompanyProfile"
)]
fn route_analysis_get_company_profile() {}

/// Congressional (senate) trading disclosures for a symbol
///
/// Implements `handlers::filings::get_congressional_trades`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getCongressionalTrades",
    method = "GET",
    path = "/v2/filings/{symbol}/congressional-trades",
    params = "FilingsQuery",
    response = "[GqlCongressionalTrade]"
)]
fn route_filings_get_congressional_trades() {}

/// Top coins by market cap
///
/// Implements `handlers::crypto::get_crypto_coins`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getCryptoCoins",
    method = "GET",
    path = "/v2/crypto/coins",
    params = "CryptoCoinsQuery",
    response = "[GqlCoinQuote]"
)]
fn route_crypto_get_crypto_coins() {}

/// Single coin by CoinGecko ID
///
/// Implements `handlers::crypto::get_crypto_coin`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getCryptoCoin",
    method = "GET",
    path = "/v2/crypto/coins/{id}",
    params = "CryptoCoinQuery",
    response = "GqlCoinQuote"
)]
fn route_crypto_get_crypto_coin() {}

/// Aggregate global cryptocurrency market statistics
///
/// Implements `handlers::crypto::get_crypto_global`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getCryptoGlobal",
    method = "GET",
    path = "/v2/crypto/global",
    params = "CryptoGlobalQuery",
    response = "GqlGlobalCryptoStats"
)]
fn route_crypto_get_crypto_global() {}

/// Search CoinGecko's coin universe by free-text query
///
/// Implements `handlers::crypto::get_crypto_search`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getCryptoSearch",
    method = "GET",
    path = "/v2/crypto/search",
    params = "CryptoSearchQuery",
    response = "[GqlSymbolMatch]"
)]
fn route_crypto_get_crypto_search() {}

/// Coins trending on CoinGecko over the last 24h
///
/// Implements `handlers::crypto::get_crypto_trending`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getCryptoTrending",
    method = "GET",
    path = "/v2/crypto/trending",
    params = "CryptoTrendingQuery",
    response = "[GqlTrendingCoin]"
)]
fn route_crypto_get_crypto_trending() {}

/// Market-wide crypto news
///
/// Implements `handlers::crypto::get_crypto_news`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getCryptoNews",
    method = "GET",
    path = "/v2/crypto/news",
    params = "MarketNewsQuery",
    response = "[GqlNews]"
)]
fn route_crypto_get_crypto_news() {}

/// Get available currencies
///
/// Implements `handlers::metadata::get_currencies`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getCurrencies",
    method = "GET",
    path = "/v2/currencies",
    params = "CurrenciesQuery",
    response = "[GqlCurrency]"
)]
fn route_metadata_get_currencies() {}

/// Get batch dividend history
///
/// Implements `handlers::events::get_batch_dividends`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getBatchDividends",
    method = "GET",
    path = "/v2/dividends",
    params = "BatchDividendsQuery",
    response = "GqlDividendsBatch"
)]
fn route_events_get_batch_dividends() {}

/// Get dividend history and analytics
///
/// Implements `handlers::events::get_dividends`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getDividends",
    method = "GET",
    path = "/v2/dividends/{symbol}",
    params = "DividendsQuery",
    response = "GqlDividends"
)]
fn route_events_get_dividends() {}

/// Get earnings-surprise history
///
/// Implements `handlers::analysis::get_earnings_surprises`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getEarningsSurprises",
    method = "GET",
    path = "/v2/earnings-surprises/{symbol}",
    params = "AnalysisQuery",
    response = "GqlEarningsSurpriseHistory"
)]
fn route_analysis_get_earnings_surprises() {}

/// Get earnings call transcript
///
/// Implements `handlers::analysis::get_earnings_transcript`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getEarningsTranscript",
    method = "GET",
    path = "/v2/earnings-transcript/{symbol}",
    params = "EarningsTranscriptV2Query",
    response = "GqlEarningsTranscript"
)]
fn route_analysis_get_earnings_transcript() {}

/// Resolve ticker to CIK
///
/// Implements `handlers::edgar::get_edgar_cik`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getEdgarCik",
    method = "GET",
    path = "/v2/edgar/cik/{symbol}",
    params = "EdgarFieldsQuery",
    response = "GqlEdgarCik"
)]
fn route_edgar_get_edgar_cik() {}

/// Get XBRL financial data
///
/// Implements `handlers::edgar::get_edgar_facts`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getEdgarFacts",
    method = "GET",
    path = "/v2/edgar/facts/{symbol}",
    params = "EdgarFactsQuery",
    response = "[GqlFactConcept]"
)]
fn route_edgar_get_edgar_facts() {}

/// Search SEC filings
///
/// Implements `handlers::edgar::get_edgar_search`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getEdgarSearch",
    method = "GET",
    path = "/v2/edgar/search",
    params = "EdgarSearchQuery",
    response = "GqlEdgarSearchResults"
)]
fn route_edgar_get_edgar_search() {}

/// Get SEC filing history
///
/// Implements `handlers::edgar::get_edgar_submissions`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getEdgarSubmissions",
    method = "GET",
    path = "/v2/edgar/submissions/{symbol}",
    params = "EdgarSubmissionsQuery",
    response = "GqlEdgarSubmissions"
)]
fn route_edgar_get_edgar_submissions() {}

/// Get supported exchanges
///
/// Implements `handlers::metadata::get_exchanges`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getExchanges",
    method = "GET",
    path = "/v2/exchanges",
    params = "ExchangesQuery",
    response = "[GqlExchange]"
)]
fn route_metadata_get_exchanges() {}

/// Fails-to-deliver records for a symbol
///
/// Implements `handlers::filings::get_fails_to_deliver`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getFailsToDeliver",
    method = "GET",
    path = "/v2/filings/{symbol}/fails-to-deliver",
    params = "FilingsQuery",
    response = "[GqlFailToDeliver]"
)]
fn route_filings_get_fails_to_deliver() {}

/// Fear & Greed Index
///
/// Implements `handlers::market::get_fear_and_greed`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getFearAndGreed",
    method = "GET",
    path = "/v2/fear-and-greed",
    params = "FearAndGreedQuery",
    response = "GqlFearAndGreed"
)]
fn route_market_get_fear_and_greed() {}

/// RSS/Atom news feeds
///
/// Implements `handlers::feeds::get_feeds`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getFeeds",
    method = "GET",
    path = "/v2/feeds",
    params = "FeedsQuery",
    response = "[GqlFeedEntry]"
)]
fn route_feeds_get_feeds() {}

/// Get batch financial statements
///
/// Implements `handlers::financials::get_batch_financials`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getBatchFinancials",
    method = "GET",
    path = "/v2/financials",
    params = "BatchFinancialsQuery",
    response = "GqlFinancialsBatch"
)]
fn route_financials_get_batch_financials() {}

/// Get financial statements
///
/// Implements `handlers::financials::get_financials`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getFinancials",
    method = "GET",
    path = "/v2/financials/{symbol}/{statement}",
    params = "FinancialsQuery",
    path_params = "statement: StatementType",
    response = "[GqlFinancialLineItem]"
)]
fn route_financials_get_financials() {}

/// Market-wide forex news
///
/// Implements `handlers::forex::get_forex_news`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getForexNews",
    method = "GET",
    path = "/v2/forex/news",
    params = "MarketNewsQuery",
    response = "[GqlNews]"
)]
fn route_forex_get_forex_news() {}

/// FRED time series
///
/// Implements `handlers::fred::get_fred_series`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getFredSeries",
    method = "GET",
    path = "/v2/fred/series/{id}",
    params = "FredSeriesQuery",
    response = "GqlMacroSeries"
)]
fn route_fred_get_fred_series() {}

/// US Treasury yield curve
///
/// Implements `handlers::fred::get_fred_treasury_yields`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getFredTreasuryYields",
    method = "GET",
    path = "/v2/fred/treasury-yields",
    params = "TreasuryYieldsQuery",
    response = "[GqlTreasuryYield]"
)]
fn route_fred_get_fred_treasury_yields() {}

/// Worldwide news mentioning a symbol (GDELT, keyless)
///
/// Implements `handlers::keyless::get_gdelt_news`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getGdeltNews",
    method = "GET",
    path = "/v2/gdelt/news/{symbol}",
    params = "GdeltNewsQuery",
    response = "[GqlNews]"
)]
fn route_keyless_get_gdelt_news() {}

/// Health check
///
/// Implements `handlers::system::health_check`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "healthCheck",
    method = "GET",
    path = "/v2/health",
    response = "HealthResponse"
)]
fn route_system_health_check() {}

/// Get holder data
///
/// Implements `handlers::holders::get_holders`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getHolders",
    method = "GET",
    path = "/v2/holders/{symbol}/{type}",
    params = "HoldersQuery",
    path_params = "type: HolderType",
    response = "HoldersResponse"
)]
fn route_holders_get_holders() {}

/// Get market hours
///
/// Implements `handlers::metadata::get_hours`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getHours",
    method = "GET",
    path = "/v2/hours",
    params = "HoursQuery",
    response = "GqlMarketHours"
)]
fn route_metadata_get_hours() {}

/// Get batch technical indicators
///
/// Implements `handlers::indicators::get_batch_indicators`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getBatchIndicators",
    method = "GET",
    path = "/v2/indicators",
    params = "BatchIndicatorsQuery",
    response = "GqlIndicatorsBatch"
)]
fn route_indicators_get_batch_indicators() {}

/// Get technical indicators
///
/// Implements `handlers::indicators::get_indicators`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getIndicators",
    method = "GET",
    path = "/v2/indicators/{symbol}",
    params = "IndicatorsQuery",
    response = "GqlIndicatorsSummary"
)]
fn route_indicators_get_indicators() {}

/// Get world market indices
///
/// Implements `handlers::market::get_indices`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getIndices",
    method = "GET",
    path = "/v2/indices",
    params = "IndicesQuery",
    response = "[GqlQuote]"
)]
fn route_market_get_indices() {}

/// Get industry data
///
/// Implements `handlers::sector::get_industry`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getIndustry",
    method = "GET",
    path = "/v2/industries/{industry}",
    params = "SectorQuery",
    path_params = "industry: Industry",
    response = "GqlIndustryData"
)]
fn route_sector_get_industry() {}

/// Look up symbols by type
///
/// Implements `handlers::search::lookup`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "lookup",
    method = "GET",
    path = "/v2/lookup",
    params = "LookupQuery",
    response = "GqlLookupResults"
)]
fn route_search_lookup() {}

/// Get market summary
///
/// Implements `handlers::market::get_market_summary`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getMarketSummary",
    method = "GET",
    path = "/v2/market-summary",
    params = "MarketSummaryQuery",
    response = "[GqlMarketSummaryQuote]"
)]
fn route_market_get_market_summary() {}

/// Prometheus metrics
///
/// Implements `handlers::system::get_metrics`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getMetrics",
    method = "GET",
    path = "/v2/metrics",
    response = "MetricsText"
)]
fn route_system_get_metrics() {}

/// Get general market news
///
/// Implements `handlers::news::get_general_news`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getGeneralNews",
    method = "GET",
    path = "/v2/news",
    params = "NewsQuery",
    response = "[GqlNews]"
)]
fn route_news_get_general_news() {}

/// Get news for a symbol
///
/// Implements `handlers::news::get_news`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getNews",
    method = "GET",
    path = "/v2/news/{symbol}",
    params = "NewsQuery",
    response = "[GqlNews]"
)]
fn route_news_get_news() {}

/// Get batch options chains
///
/// Implements `handlers::options::get_batch_options`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getBatchOptions",
    method = "GET",
    path = "/v2/options",
    params = "BatchOptionsQuery",
    response = "GqlOptionsBatch"
)]
fn route_options_get_batch_options() {}

/// Get options chain
///
/// Implements `handlers::options::get_options`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getOptions",
    method = "GET",
    path = "/v2/options/{symbol}",
    params = "OptionsQuery",
    response = "GqlOptions"
)]
fn route_options_get_options() {}

/// Ping endpoint
///
/// Implements `handlers::system::ping`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "ping",
    method = "GET",
    path = "/v2/ping",
    response = "PingResponse"
)]
fn route_system_ping() {}

/// Get quote type
///
/// Implements `handlers::metadata::get_quote_type`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getQuoteType",
    method = "GET",
    path = "/v2/quote-type/{symbol}",
    params = "QuoteTypeQuery",
    response = "GqlQuoteTypeData"
)]
fn route_metadata_get_quote_type() {}

/// Get quote for a symbol
///
/// Implements `handlers::quote::get_quote`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getQuote",
    method = "GET",
    path = "/v2/quote/{symbol}",
    params = "QuoteQuery",
    response = "GqlQuote"
)]
fn route_quote_get_quote() {}

/// Get quotes for multiple symbols
///
/// Implements `handlers::quote::get_quotes`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getQuotes",
    method = "GET",
    path = "/v2/quotes",
    params = "QuotesQuery",
    response = "GqlQuotesBatch"
)]
fn route_quote_get_quotes() {}

/// Get batch stock recommendations
///
/// Implements `handlers::analysis::get_batch_recommendations`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getBatchRecommendations",
    method = "GET",
    path = "/v2/recommendations",
    params = "BatchRecommendationsQuery",
    response = "GqlRecommendationsBatch"
)]
fn route_analysis_get_batch_recommendations() {}

/// Get similar stock recommendations
///
/// Implements `handlers::analysis::get_recommendations`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getRecommendations",
    method = "GET",
    path = "/v2/recommendations/{symbol}",
    params = "RecommendationsQuery",
    response = "GqlRecommendation"
)]
fn route_analysis_get_recommendations() {}

/// Risk analytics
///
/// Implements `handlers::risk::get_risk`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getRisk",
    method = "GET",
    path = "/v2/risk/{symbol}",
    params = "RiskQuery",
    response = "GqlRiskSummary"
)]
fn route_risk_get_risk() {}

/// Custom screener query
///
/// Implements `handlers::screener::post_custom_screener`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "postCustomScreener",
    method = "POST",
    path = "/v2/screeners/custom",
    request = "CustomScreenerRequest",
    response = "GqlScreenerResults"
)]
fn route_screener_post_custom_screener() {}

/// Get screener results
///
/// Implements `handlers::screener::get_screeners`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getScreeners",
    method = "GET",
    path = "/v2/screeners/{screener}",
    params = "ScreenersQuery",
    path_params = "screener: Screener",
    response = "GqlScreenerResults"
)]
fn route_screener_get_screeners() {}

/// Search symbols and companies
///
/// Implements `handlers::search::search`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "search",
    method = "GET",
    path = "/v2/search",
    params = "SearchQuery",
    response = "GqlSearchResults"
)]
fn route_search_search() {}

/// Get sector data
///
/// Implements `handlers::sector::get_sector`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getSector",
    method = "GET",
    path = "/v2/sectors/{sector}",
    params = "SectorQuery",
    path_params = "sector: Sector",
    response = "GqlSectorData"
)]
fn route_sector_get_sector() {}

/// Get batch sparkline data
///
/// Implements `handlers::chart::get_spark`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getSpark",
    method = "GET",
    path = "/v2/spark",
    params = "SparkQuery",
    response = "GqlSparkBatch"
)]
fn route_chart_get_spark() {}

/// Get batch stock split history
///
/// Implements `handlers::events::get_batch_splits`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getBatchSplits",
    method = "GET",
    path = "/v2/splits",
    params = "BatchSplitsQuery",
    response = "GqlSplitsBatch"
)]
fn route_events_get_batch_splits() {}

/// Get stock split history
///
/// Implements `handlers::events::get_splits`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getSplits",
    method = "GET",
    path = "/v2/splits/{symbol}",
    params = "RangeQuery",
    response = "[GqlSplit]"
)]
fn route_events_get_splits() {}

/// Get earnings transcript
///
/// Implements `handlers::transcripts::get_transcript`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getTranscript",
    method = "GET",
    path = "/v2/transcripts/{symbol}",
    params = "EarningsTranscriptQuery",
    response = "GqlTranscript"
)]
fn route_transcripts_get_transcript() {}

/// Get all earnings transcripts
///
/// Implements `handlers::transcripts::get_transcripts`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getTranscripts",
    method = "GET",
    path = "/v2/transcripts/{symbol}/all",
    params = "EarningsTranscriptsQuery",
    response = "[GqlTranscriptWithMeta]"
)]
fn route_transcripts_get_transcripts() {}

/// Get trending tickers
///
/// Implements `handlers::market::get_trending`.
#[allow(dead_code)]
#[soothfast::route(
    spec = "openapi.yaml",
    operation = "getTrending",
    method = "GET",
    path = "/v2/trending",
    params = "TrendingQuery",
    response = "[GqlTrendingQuote]"
)]
fn route_market_get_trending() {}

// -- AsyncAPI (server/asyncapi.yaml) --------------------------------------
// Each websocket handler is bidirectional (one Rust fn implements both
// directions of its channel). The two directions get separate marker fns —
// not stacked attrs on one — because Rust merges stacked `///` comments
// into a single doc string for the item, so a shared fn can't carry two
// distinct summaries.

/// Real-time price ticks, or fired alert events while alert rules are active.
///
/// Implements `handlers::stream::ws_stream_handler` (subscribe direction).
#[allow(dead_code)]
#[soothfast::route(
    spec = "asyncapi.yaml",
    operation = "receiveMessage",
    method = "SUBSCRIBE",
    path = "/v2/stream",
    response = "PriceStreamMessage"
)]
fn route_stream_ws_stream_handler_receive() {}

/// Subscribe/unsubscribe to symbols, or set alert rules.
///
/// Implements `handlers::stream::ws_stream_handler` (publish direction).
#[allow(dead_code)]
#[soothfast::route(
    spec = "asyncapi.yaml",
    operation = "sendMessage",
    method = "PUBLISH",
    path = "/v2/stream",
    request = "StreamCommand"
)]
fn route_stream_ws_stream_handler_send() {}

/// Newly-seen RSS/Atom feed entries as the server polls each source.
///
/// Implements `handlers::feeds_stream::ws_feeds_stream_handler` (subscribe direction).
#[allow(dead_code)]
#[soothfast::route(
    spec = "asyncapi.yaml",
    operation = "receiveFeedEntry",
    method = "SUBSCRIBE",
    path = "/v2/feeds/stream",
    response = "GqlFeedEntry"
)]
fn route_feeds_stream_ws_feeds_stream_handler_receive() {}

/// Subscribe/unsubscribe to feed sources.
///
/// Implements `handlers::feeds_stream::ws_feeds_stream_handler` (publish direction).
#[allow(dead_code)]
#[soothfast::route(
    spec = "asyncapi.yaml",
    operation = "sendFeedMessage",
    method = "PUBLISH",
    path = "/v2/feeds/stream",
    request = "FeedStreamCommand"
)]
fn route_feeds_stream_ws_feeds_stream_handler_send() {}

// -- GraphQL (server/schema.graphql) --------------------------------------
//
// soothfast's GraphQL provider only sees ROOT-level Query/Subscription/
// Mutation fields (it line-scans `type <Root>` blocks, not the type graph),
// so only these 44 fields are reconcilable. `GqlTicker`'s ~26 nested
// per-ticker sub-fields (reached via `ticker(symbol: ...) { ... }`,
// TickerCoreQuery/TickerEventsQuery/TickerHoldersQuery/TickerAnalysisQuery
// in graphql/query/ticker_*.rs) and the 4 nested pagination resolvers in
// graphql/types/holders.rs are outside this — a real GraphQL schema-graph
// walk would be needed to cover those; not attempted here.

/// Implements `graphql::query::root_batch::ticker` (field `ticker`).
#[allow(dead_code)]
#[soothfast::route(
    spec = "schema.graphql",
    operation = "ticker",
    method = "QUERY",
    path = "ticker"
)]
fn route_gql_root_batch_ticker() {}

/// Implements `graphql::query::root_batch::quotes` (field `quotes`).
#[allow(dead_code)]
#[soothfast::route(
    spec = "schema.graphql",
    operation = "quotes",
    method = "QUERY",
    path = "quotes"
)]
fn route_gql_root_batch_quotes() {}

/// Implements `graphql::query::root_batch::charts` (field `charts`).
#[allow(dead_code)]
#[soothfast::route(
    spec = "schema.graphql",
    operation = "charts",
    method = "QUERY",
    path = "charts"
)]
fn route_gql_root_batch_charts() {}

/// Implements `graphql::query::root_batch::spark` (field `spark`).
#[allow(dead_code)]
#[soothfast::route(
    spec = "schema.graphql",
    operation = "spark",
    method = "QUERY",
    path = "spark"
)]
fn route_gql_root_batch_spark() {}

/// Implements `graphql::query::root_batch::options_batch` (field `optionsBatch`).
#[allow(dead_code)]
#[soothfast::route(
    spec = "schema.graphql",
    operation = "optionsBatch",
    method = "QUERY",
    path = "optionsBatch"
)]
fn route_gql_root_batch_options_batch() {}

/// Implements `graphql::query::root_batch::splits_batch` (field `splitsBatch`).
#[allow(dead_code)]
#[soothfast::route(
    spec = "schema.graphql",
    operation = "splitsBatch",
    method = "QUERY",
    path = "splitsBatch"
)]
fn route_gql_root_batch_splits_batch() {}

/// Implements `graphql::query::root_batch::capital_gains_batch` (field `capitalGainsBatch`).
#[allow(dead_code)]
#[soothfast::route(
    spec = "schema.graphql",
    operation = "capitalGainsBatch",
    method = "QUERY",
    path = "capitalGainsBatch"
)]
fn route_gql_root_batch_capital_gains_batch() {}

/// Implements `graphql::query::root_batch::dividends_batch` (field `dividendsBatch`).
#[allow(dead_code)]
#[soothfast::route(
    spec = "schema.graphql",
    operation = "dividendsBatch",
    method = "QUERY",
    path = "dividendsBatch"
)]
fn route_gql_root_batch_dividends_batch() {}

/// Implements `graphql::query::root_batch::indicators_batch` (field `indicatorsBatch`).
#[allow(dead_code)]
#[soothfast::route(
    spec = "schema.graphql",
    operation = "indicatorsBatch",
    method = "QUERY",
    path = "indicatorsBatch"
)]
fn route_gql_root_batch_indicators_batch() {}

/// Implements `graphql::query::root_batch::recommendations_batch` (field `recommendationsBatch`).
#[allow(dead_code)]
#[soothfast::route(
    spec = "schema.graphql",
    operation = "recommendationsBatch",
    method = "QUERY",
    path = "recommendationsBatch"
)]
fn route_gql_root_batch_recommendations_batch() {}

/// Implements `graphql::query::root_batch::financials_batch` (field `financialsBatch`).
#[allow(dead_code)]
#[soothfast::route(
    spec = "schema.graphql",
    operation = "financialsBatch",
    method = "QUERY",
    path = "financialsBatch"
)]
fn route_gql_root_batch_financials_batch() {}

/// Implements `graphql::query::root_discovery::search` (field `search`).
#[allow(dead_code)]
#[soothfast::route(
    spec = "schema.graphql",
    operation = "search",
    method = "QUERY",
    path = "search"
)]
fn route_gql_root_discovery_search() {}

/// Implements `graphql::query::root_discovery::lookup` (field `lookup`).
#[allow(dead_code)]
#[soothfast::route(
    spec = "schema.graphql",
    operation = "lookup",
    method = "QUERY",
    path = "lookup"
)]
fn route_gql_root_discovery_lookup() {}

/// Implements `graphql::query::root_discovery::screener` (field `screener`).
#[allow(dead_code)]
#[soothfast::route(
    spec = "schema.graphql",
    operation = "screener",
    method = "QUERY",
    path = "screener"
)]
fn route_gql_root_discovery_screener() {}

/// Implements `graphql::query::root_discovery::custom_screener` (field `customScreener`).
#[allow(dead_code)]
#[soothfast::route(
    spec = "schema.graphql",
    operation = "customScreener",
    method = "QUERY",
    path = "customScreener"
)]
fn route_gql_root_discovery_custom_screener() {}

/// Implements `graphql::query::root_market::trending` (field `trending`).
#[allow(dead_code)]
#[soothfast::route(
    spec = "schema.graphql",
    operation = "trending",
    method = "QUERY",
    path = "trending"
)]
fn route_gql_root_market_trending() {}

/// Implements `graphql::query::root_market::fear_and_greed` (field `fearAndGreed`).
#[allow(dead_code)]
#[soothfast::route(
    spec = "schema.graphql",
    operation = "fearAndGreed",
    method = "QUERY",
    path = "fearAndGreed"
)]
fn route_gql_root_market_fear_and_greed() {}

/// Implements `graphql::query::root_market::market_summary` (field `marketSummary`).
#[allow(dead_code)]
#[soothfast::route(
    spec = "schema.graphql",
    operation = "marketSummary",
    method = "QUERY",
    path = "marketSummary"
)]
fn route_gql_root_market_market_summary() {}

/// Implements `graphql::query::root_market::news` (field `news`).
#[allow(dead_code)]
#[soothfast::route(
    spec = "schema.graphql",
    operation = "news",
    method = "QUERY",
    path = "news"
)]
fn route_gql_root_market_news() {}

/// Implements `graphql::query::root_market::feeds` (field `feeds`).
#[allow(dead_code)]
#[soothfast::route(
    spec = "schema.graphql",
    operation = "feeds",
    method = "QUERY",
    path = "feeds"
)]
fn route_gql_root_market_feeds() {}

/// Implements `graphql::query::root_market::indices` (field `indices`).
#[allow(dead_code)]
#[soothfast::route(
    spec = "schema.graphql",
    operation = "indices",
    method = "QUERY",
    path = "indices"
)]
fn route_gql_root_market_indices() {}

/// Implements `graphql::query::root_market::sector` (field `sector`).
#[allow(dead_code)]
#[soothfast::route(
    spec = "schema.graphql",
    operation = "sector",
    method = "QUERY",
    path = "sector"
)]
fn route_gql_root_market_sector() {}

/// Implements `graphql::query::root_market::industry` (field `industry`).
#[allow(dead_code)]
#[soothfast::route(
    spec = "schema.graphql",
    operation = "industry",
    method = "QUERY",
    path = "industry"
)]
fn route_gql_root_market_industry() {}

/// Implements `graphql::query::root_metadata::crypto_coins` (field `cryptoCoins`).
#[allow(dead_code)]
#[soothfast::route(
    spec = "schema.graphql",
    operation = "cryptoCoins",
    method = "QUERY",
    path = "cryptoCoins"
)]
fn route_gql_root_metadata_crypto_coins() {}

/// Implements `graphql::query::root_metadata::crypto_coin` (field `cryptoCoin`).
#[allow(dead_code)]
#[soothfast::route(
    spec = "schema.graphql",
    operation = "cryptoCoin",
    method = "QUERY",
    path = "cryptoCoin"
)]
fn route_gql_root_metadata_crypto_coin() {}

/// Implements `graphql::query::root_metadata::edgar_cik` (field `edgarCik`).
#[allow(dead_code)]
#[soothfast::route(
    spec = "schema.graphql",
    operation = "edgarCik",
    method = "QUERY",
    path = "edgarCik"
)]
fn route_gql_root_metadata_edgar_cik() {}

/// Implements `graphql::query::root_metadata::market_hours` (field `marketHours`).
#[allow(dead_code)]
#[soothfast::route(
    spec = "schema.graphql",
    operation = "marketHours",
    method = "QUERY",
    path = "marketHours"
)]
fn route_gql_root_metadata_market_hours() {}

/// Implements `graphql::query::root_metadata::quote_type` (field `quoteType`).
#[allow(dead_code)]
#[soothfast::route(
    spec = "schema.graphql",
    operation = "quoteType",
    method = "QUERY",
    path = "quoteType"
)]
fn route_gql_root_metadata_quote_type() {}

/// Implements `graphql::query::root_metadata::currencies` (field `currencies`).
#[allow(dead_code)]
#[soothfast::route(
    spec = "schema.graphql",
    operation = "currencies",
    method = "QUERY",
    path = "currencies"
)]
fn route_gql_root_metadata_currencies() {}

/// Implements `graphql::query::root_metadata::exchanges` (field `exchanges`).
#[allow(dead_code)]
#[soothfast::route(
    spec = "schema.graphql",
    operation = "exchanges",
    method = "QUERY",
    path = "exchanges"
)]
fn route_gql_root_metadata_exchanges() {}

/// Implements `graphql::query::root_metadata::calendar` (field `calendar`).
#[allow(dead_code)]
#[soothfast::route(
    spec = "schema.graphql",
    operation = "calendar",
    method = "QUERY",
    path = "calendar"
)]
fn route_gql_root_metadata_calendar() {}

/// Implements `graphql::query::root_metadata::fred_series` (field `fredSeries`).
#[allow(dead_code)]
#[soothfast::route(
    spec = "schema.graphql",
    operation = "fredSeries",
    method = "QUERY",
    path = "fredSeries"
)]
fn route_gql_root_metadata_fred_series() {}

/// Implements `graphql::query::root_metadata::treasury_yields` (field `treasuryYields`).
#[allow(dead_code)]
#[soothfast::route(
    spec = "schema.graphql",
    operation = "treasuryYields",
    method = "QUERY",
    path = "treasuryYields"
)]
fn route_gql_root_metadata_treasury_yields() {}

/// Implements `graphql::query::root_metadata::edgar_search` (field `edgarSearch`).
#[allow(dead_code)]
#[soothfast::route(
    spec = "schema.graphql",
    operation = "edgarSearch",
    method = "QUERY",
    path = "edgarSearch"
)]
fn route_gql_root_metadata_edgar_search() {}

/// Implements `graphql::query::root_metadata::gdelt_news` (field `gdeltNews`).
#[allow(dead_code)]
#[soothfast::route(
    spec = "schema.graphql",
    operation = "gdeltNews",
    method = "QUERY",
    path = "gdeltNews"
)]
fn route_gql_root_metadata_gdelt_news() {}

/// Implements `graphql::query::root_metadata::commitments_of_traders` (field `commitmentsOfTraders`).
#[allow(dead_code)]
#[soothfast::route(
    spec = "schema.graphql",
    operation = "commitmentsOfTraders",
    method = "QUERY",
    path = "commitmentsOfTraders"
)]
fn route_gql_root_metadata_commitments_of_traders() {}

/// Implements `graphql::query::root_metadata::crypto_trending` (field `cryptoTrending`).
#[allow(dead_code)]
#[soothfast::route(
    spec = "schema.graphql",
    operation = "cryptoTrending",
    method = "QUERY",
    path = "cryptoTrending"
)]
fn route_gql_root_metadata_crypto_trending() {}

/// Implements `graphql::query::root_metadata::crypto_search` (field `cryptoSearch`).
#[allow(dead_code)]
#[soothfast::route(
    spec = "schema.graphql",
    operation = "cryptoSearch",
    method = "QUERY",
    path = "cryptoSearch"
)]
fn route_gql_root_metadata_crypto_search() {}

/// Implements `graphql::query::root_metadata::crypto_global` (field `cryptoGlobal`).
#[allow(dead_code)]
#[soothfast::route(
    spec = "schema.graphql",
    operation = "cryptoGlobal",
    method = "QUERY",
    path = "cryptoGlobal"
)]
fn route_gql_root_metadata_crypto_global() {}

/// Implements `graphql::query::root_metadata::crypto_news` (field `cryptoNews`).
#[allow(dead_code)]
#[soothfast::route(
    spec = "schema.graphql",
    operation = "cryptoNews",
    method = "QUERY",
    path = "cryptoNews"
)]
fn route_gql_root_metadata_crypto_news() {}

/// Implements `graphql::query::root_metadata::forex_news` (field `forexNews`).
#[allow(dead_code)]
#[soothfast::route(
    spec = "schema.graphql",
    operation = "forexNews",
    method = "QUERY",
    path = "forexNews"
)]
fn route_gql_root_metadata_forex_news() {}

/// Implements `graphql::subscription::price_stream` (field `priceStream`).
#[allow(dead_code)]
#[soothfast::route(
    spec = "schema.graphql",
    operation = "priceStream",
    method = "SUBSCRIPTION",
    path = "priceStream"
)]
fn route_gql_subscription_price_stream() {}

/// Implements `graphql::subscription::feed_stream` (field `feedStream`).
#[allow(dead_code)]
#[soothfast::route(
    spec = "schema.graphql",
    operation = "feedStream",
    method = "SUBSCRIPTION",
    path = "feedStream"
)]
fn route_gql_subscription_feed_stream() {}

/// Implements `graphql::subscription::price_alerts` (field `priceAlerts`).
#[allow(dead_code)]
#[soothfast::route(
    spec = "schema.graphql",
    operation = "priceAlerts",
    method = "SUBSCRIPTION",
    path = "priceAlerts"
)]
fn route_gql_subscription_price_alerts() {}
