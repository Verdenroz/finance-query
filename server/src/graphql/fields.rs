//! GraphQL field-selection primitives shared by REST (`main.rs`) and MCP
//! (`finance-query-mcp/src/tools/gql.rs`) — the single source of truth for
//! valid GraphQL field names, composite (nested-object) sub-selections, and
//! the escaping/selection-building helpers both transports need to safely
//! splice caller-requested field names into query text.
//!
//! Each consumer still owns its own "default fields when omitted" policy —
//! REST defaults to the full `*_VALID_FIELDS` set (backward-compatible, no
//! curation), MCP defaults to a smaller curated set to keep responses small
//! for an LLM context window — so `*_DEFAULT_FIELDS` constants stay local to
//! each consumer, built from the shared `*_VALID_FIELDS` here.

// ── Safe string-literal interpolation ────────────────────────────────────────

/// Escape a string for safe use as a GraphQL string literal's contents
/// (caller still wraps the result in `"..."`).
pub fn escape_gql_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Build a `"a", "b", "c"` GraphQL list literal from a list of raw strings,
/// escaping each element. Generic over `&[String]` (MCP, from parsed CSV) and
/// `&[&str]` (REST, split in place) alike.
pub fn gql_string_list_literal<S: AsRef<str>>(items: &[S]) -> String {
    items
        .iter()
        .map(|s| format!("\"{}\"", escape_gql_string(s.as_ref())))
        .collect::<Vec<_>>()
        .join(", ")
}

// ── GraphQL response unwrapping ──────────────────────────────────────────────

/// Pull a symbol-scoped field out of `{ ticker: { <field>: ... } }`.
pub fn unwrap_ticker_field(mut data: serde_json::Value, field: &str) -> serde_json::Value {
    data.get_mut("ticker")
        .and_then(|v| v.as_object_mut())
        .and_then(|obj| obj.remove(field))
        .unwrap_or(serde_json::Value::Null)
}

/// Pull a top-level root field out of `{ <field>: ... }`.
pub fn unwrap_field(mut data: serde_json::Value, field: &str) -> serde_json::Value {
    data.as_object_mut()
        .and_then(|obj| obj.remove(field))
        .unwrap_or(serde_json::Value::Null)
}

// ── Valid fields per typed GraphQL object ────────────────────────────────────
//
// Selection-set *building* stays consumer-local (`build_rest_composite_selection`
// in `server/src/main.rs`, `build_type_spec_selection` in
// `finance-query-mcp/src/tools/gql.rs`) since REST and MCP take field lists in
// different shapes (raw CSV `&str` vs. pre-split `Vec<String>`) and have
// different "fields omitted" defaults (REST: everything; MCP: a curated
// subset). Only the field-name/composite-sub-selection *data* below —
// objective facts about the GraphQL schema — is shared.

/// Valid GraphQL field names for `GqlQuote`.
///
/// These are the camelCase GraphQL names (produced by `#[graphql(rename_fields =
/// "camelCase")]`), not Rust field names. Kept in sync with
/// `server/src/graphql/types/quote.rs`.
pub const GQL_QUOTE_VALID_FIELDS: &[&str] = &[
    // Identity & metadata
    "symbol",
    "logoUrl",
    "companyLogoUrl",
    "shortName",
    "longName",
    "exchange",
    "exchangeName",
    "quoteType",
    "currency",
    "currencySymbol",
    "underlyingSymbol",
    "fromCurrency",
    "toCurrency",
    // Real-time price data
    "regularMarketPrice",
    "regularMarketChange",
    "regularMarketChangePercent",
    "regularMarketTime",
    "regularMarketDayHigh",
    "regularMarketDayLow",
    "regularMarketOpen",
    "regularMarketPreviousClose",
    "regularMarketVolume",
    "marketState",
    // Convenience aliases
    "dayHigh",
    "dayLow",
    "open",
    "previousClose",
    "volume",
    "allTimeHigh",
    "allTimeLow",
    // Pre/post market
    "preMarketPrice",
    "preMarketChange",
    "preMarketChangePercent",
    "preMarketTime",
    "postMarketPrice",
    "postMarketChange",
    "postMarketChangePercent",
    "postMarketTime",
    // Volume & market cap
    "averageVolume",
    "marketCap",
    "enterpriseValue",
    "enterpriseToRevenue",
    "enterpriseToEbitda",
    "priceToBook",
    // Valuation ratios
    "forwardPe",
    "trailingPe",
    "beta",
    // 52-week range & moving averages
    "fiftyTwoWeekHigh",
    "fiftyTwoWeekLow",
    "fiftyDayAverage",
    "twoHundredDayAverage",
    // Dividends
    "dividendRate",
    "dividendYield",
    "trailingAnnualDividendRate",
    "trailingAnnualDividendYield",
    "fiveYearAvgDividendYield",
    "exDividendDate",
    "payoutRatio",
    "lastDividendValue",
    "lastDividendDate",
    // Bid / ask
    "bid",
    "bidSize",
    "ask",
    "askSize",
    // Shares & ownership
    "sharesOutstanding",
    "floatShares",
    "impliedSharesOutstanding",
    "heldPercentInsiders",
    "heldPercentInstitutions",
    "sharesShort",
    "sharesShortPriorMonth",
    "shortRatio",
    "shortPercentOfFloat",
    "sharesPercentSharesOut",
    "dateShortInterest",
    // Analyst targets
    "currentPrice",
    "targetHighPrice",
    "targetLowPrice",
    "targetMeanPrice",
    "targetMedianPrice",
    "recommendationMean",
    "numberOfAnalystOpinions",
    "recommendationKey",
    // Financials (key metrics)
    "totalDebt",
    "totalRevenue",
    "netIncomeToCommon",
    "debtToEquity",
    "revenuePerShare",
    "returnOnAssets",
    "returnOnEquity",
    "freeCashflow",
    "operatingCashflow",
    "profitMargins",
    "grossMargins",
    "ebitdaMargins",
    "operatingMargins",
    "grossProfits",
    "earningsGrowth",
    "revenueGrowth",
    "earningsQuarterlyGrowth",
    "currentRatio",
    "quickRatio",
    "trailingEps",
    "forwardEps",
    "bookValue",
    // Company profile
    "sector",
    "sectorKey",
    "sectorDisp",
    "industry",
    "industryKey",
    "industryDisp",
    "longBusinessSummary",
    "website",
    "irWebsite",
    "city",
    "state",
    "zip",
    "country",
    "phone",
    "fullTimeEmployees",
    // Fund-specific
    "category",
    "fundFamily",
    "navPrice",
    "totalAssets",
    "yieldValue",
    // Governance
    "auditRisk",
    "boardRisk",
    "compensationRisk",
    "shareholderRightsRisk",
    "overallRisk",
    // Exchange metadata
    "timeZoneFullName",
    "timeZoneShortName",
    "gmtOffSetMilliseconds",
    "firstTradeDateEpochUtc",
    "exchangeDataDelayedBy",
    "financialCurrency",
    "tradeable",
    "priceHint",
    // Dates
    "lastSplitDate",
    "lastSplitFactor",
    "lastFiscalYearEnd",
    "nextFiscalYearEnd",
    "mostRecentQuarter",
    // Complex nested objects (opaque JSON)
    "earnings",
    "calendarEvents",
    "recommendationTrend",
    "upgradeDowngradeHistory",
    "earningsHistory",
    "earningsTrend",
    "insiderHolders",
    "insiderTransactions",
    "institutionOwnership",
    "fundOwnership",
    "majorHoldersBreakdown",
    "netSharePurchaseActivity",
    "secFilings",
    "balanceSheetHistory",
    "balanceSheetHistoryQuarterly",
    "cashflowStatementHistory",
    "cashflowStatementHistoryQuarterly",
    "incomeStatementHistory",
    "incomeStatementHistoryQuarterly",
    "equityPerformance",
    "indexTrend",
    "industryTrend",
    "sectorTrend",
    "fundProfile",
    "fundPerformance",
    "topHoldings",
    "companyOfficers",
];

/// Valid GraphQL field names for `GqlChart` (top-level only).
pub const GQL_CHART_VALID_FIELDS: &[&str] = &["symbol", "meta", "candles"];

/// Valid GraphQL field names for `GqlChartMeta` (sub-fields of `meta`).
pub const GQL_CHART_META_VALID_FIELDS: &[&str] = &[
    "symbol",
    "currency",
    "exchangeName",
    "fullExchangeName",
    "instrumentType",
    "firstTradeDate",
    "regularMarketTime",
    "hasPrePostMarketData",
    "gmtOffset",
    "timezone",
    "exchangeTimezoneName",
    "regularMarketPrice",
    "fiftyTwoWeekHigh",
    "fiftyTwoWeekLow",
    "regularMarketDayHigh",
    "regularMarketDayLow",
    "regularMarketVolume",
    "chartPreviousClose",
    "previousClose",
    "priceHint",
    "dataGranularity",
    "range",
];

/// Valid GraphQL field names for `GqlCandle`.
pub const GQL_CANDLE_VALID_FIELDS: &[&str] = &[
    "timestamp",
    "open",
    "high",
    "low",
    "close",
    "volume",
    "adjClose",
];

/// Valid top-level fields for `GqlSpark` (used by the batch `spark` root field).
pub const GQL_SPARK_VALID_FIELDS: &[&str] = &[
    "symbol",
    "meta",
    "timestamps",
    "closes",
    "interval",
    "range",
];

/// Valid GraphQL field names for `GqlNews`.
pub const GQL_NEWS_VALID_FIELDS: &[&str] = &["title", "link", "source", "img", "time"];

/// Valid GraphQL field names for `GqlFeedEntry` (top-level `feeds` root field).
pub const GQL_FEEDS_VALID_FIELDS: &[&str] = &["title", "url", "published", "summary", "source"];

/// Valid GraphQL field names for `GqlTrendingQuote` (top-level `trending` root field).
pub const GQL_TRENDING_VALID_FIELDS: &[&str] = &[
    "symbol",
    "shortName",
    "regularMarketPrice",
    "regularMarketChangePercent",
];

/// Valid fields for `GqlDividends` (top-level: dividends, analytics).
pub const GQL_DIVIDENDS_VALID_FIELDS: &[&str] = &["dividends", "analytics"];

/// `dividends`/`analytics` are both composite (`GqlDividend`/`GqlDividendAnalytics`)
/// and require a nested sub-selection — bare field names are invalid GraphQL.
/// `analytics.lastPayment`/`firstPayment` are themselves `GqlDividend`, hence
/// the doubly-nested selection.
pub const DIVIDENDS_COMPOSITE_FIELDS: &[(&str, &str)] = &[
    ("dividends", "{ timestamp amount }"),
    (
        "analytics",
        "{ totalPaid paymentCount averagePayment cagr lastPayment { timestamp amount } firstPayment { timestamp amount } }",
    ),
];

/// Valid fields for `GqlSplit`.
pub const GQL_SPLIT_VALID_FIELDS: &[&str] = &["timestamp", "numerator", "denominator", "ratio"];

/// Valid top-level fields for `GqlOptions`.
pub const GQL_OPTIONS_VALID_FIELDS: &[&str] = &["expirationDates", "strikes", "calls", "puts"];

/// `calls`/`puts` are `[GqlOptionContract]` (composite) and require a nested
/// sub-selection; `expirationDates`/`strikes` are scalar lists (no braces).
pub const OPTIONS_COMPOSITE_FIELDS: &[(&str, &str)] = &[
    (
        "calls",
        "{ contractSymbol strike currency lastPrice change percentChange volume openInterest bid ask contractSize expiration lastTradeDate impliedVolatility inTheMoney }",
    ),
    (
        "puts",
        "{ contractSymbol strike currency lastPrice change percentChange volume openInterest bid ask contractSize expiration lastTradeDate impliedVolatility inTheMoney }",
    ),
];

// ── Calendar ─────────────────────────────────────────────────────────────────

/// Valid top-level fields for `GqlCalendarEvent`. `event` is a GraphQL union
/// and — unlike every other composite field in this file — is always
/// expanded with inline fragments for all 6 variants rather than a flat
/// nested-object sub-selection (`build_rest_calendar_selection` in
/// `handlers/calendar.rs` / MCP's calendar tool own that expansion).
pub const GQL_CALENDAR_VALID_FIELDS: &[&str] = &["timestamp", "date", "symbol", "event"];

/// Full inline-fragment selection for the `GqlEventKind` union — spliced in
/// whenever `event` is selected.
pub const CALENDAR_EVENT_UNION_SELECTION: &str = "{ \
    __typename \
    ... on GqlEarningsEvent { epsEstimateLow epsEstimateAvg epsEstimateHigh revenueEstimateAvg isEstimate } \
    ... on GqlExDividendEvent { amount } \
    ... on GqlDividendPaymentEvent { amount } \
    ... on GqlOptionsExpirationEvent { contractCount } \
    ... on GqlEconomicReleaseEvent { name seriesId } \
    ... on GqlUnknownEvent { raw } \
}";

// ── Market-wide (indices reuse GQL_QUOTE_VALID_FIELDS) ──────────────────────

/// Valid fields for `GqlMarketSummaryQuote`.
pub const GQL_MARKET_SUMMARY_VALID_FIELDS: &[&str] = &[
    "symbol",
    "shortName",
    "fullExchangeName",
    "regularMarketPrice",
    "regularMarketChange",
    "regularMarketChangePercent",
];

/// Valid fields for `GqlFearAndGreed`.
pub const GQL_FEAR_AND_GREED_VALID_FIELDS: &[&str] = &["value", "classification", "timestamp"];

// ── Crypto (CoinGecko) ───────────────────────────────────────────────────────

pub const GQL_COIN_VALID_FIELDS: &[&str] = &[
    "id",
    "symbol",
    "name",
    "currentPrice",
    "marketCap",
    "priceChangePercentage24H",
    "totalVolume",
    "circulatingSupply",
    "image",
    "marketCapRank",
];

// ── FRED (economic series + Treasury yields) ────────────────────────────────

pub const GQL_MACRO_SERIES_VALID_FIELDS: &[&str] = &["id", "observations"];
pub const MACRO_SERIES_COMPOSITE_FIELDS: &[(&str, &str)] = &[("observations", "{ date value }")];

pub const GQL_TREASURY_YIELD_VALID_FIELDS: &[&str] = &[
    "date", "y1M", "y2M", "y3M", "y4M", "y6M", "y1", "y2", "y3", "y5", "y7", "y10", "y20", "y30",
];

// ── Market metadata (hours, quote type, currencies, exchanges) ─────────────

pub const GQL_MARKET_HOURS_VALID_FIELDS: &[&str] = &["markets"];
pub const MARKET_HOURS_COMPOSITE_FIELDS: &[(&str, &str)] = &[(
    "markets",
    "{ id name status message open close time timezone timezoneShort gmtOffset dst }",
)];

pub const GQL_QUOTE_TYPE_VALID_FIELDS: &[&str] = &[
    "exchange",
    "firstTradeDateEpochUtc",
    "gmtOffSetMilliseconds",
    "longName",
    "maxAge",
    "messageBoardId",
    "quoteType",
    "shortName",
    "symbol",
    "timeZoneFullName",
    "timeZoneShortName",
    "underlyingSymbol",
    "uuid",
];

pub const GQL_CURRENCY_VALID_FIELDS: &[&str] =
    &["shortName", "longName", "symbol", "localLongName"];

pub const GQL_EXCHANGE_VALID_FIELDS: &[&str] =
    &["country", "market", "suffix", "delay", "dataProvider"];

// ── Screeners (predefined + custom) ─────────────────────────────────────────

pub const GQL_SCREENER_RESULTS_VALID_FIELDS: &[&str] = &[
    "quotes",
    "type",
    "description",
    "lastUpdated",
    "total",
    "pageInfo",
];

/// Full field set for a single `GqlScreenerQuote` — always expanded whole
/// when `quotes` is selected (consistent with sector/industry/search: no
/// deep configurability below the first composite boundary).
pub const GQL_SCREENER_QUOTE_FIELDS: &str = "{ \
    symbol shortName longName displayName quoteType exchange \
    regularMarketPrice regularMarketChange regularMarketChangePercent \
    regularMarketOpen regularMarketDayHigh regularMarketDayLow regularMarketPreviousClose regularMarketTime \
    regularMarketVolume averageDailyVolume3Month averageDailyVolume10Day marketCap sharesOutstanding \
    fiftyTwoWeekHigh fiftyTwoWeekLow fiftyTwoWeekChange fiftyTwoWeekChangePercent \
    fiftyDayAverage fiftyDayAverageChange fiftyDayAverageChangePercent \
    twoHundredDayAverage twoHundredDayAverageChange twoHundredDayAverageChangePercent \
    averageAnalystRating trailingPE forwardPE priceToBook bookValue \
    epsTrailingTwelveMonths epsForward epsCurrentYear priceEpsCurrentYear \
    dividendYield dividendRate dividendDate trailingAnnualDividendRate trailingAnnualDividendYield \
    bid bidSize ask askSize \
    postMarketPrice postMarketChange postMarketChangePercent postMarketTime \
    preMarketPrice preMarketChange preMarketChangePercent preMarketTime \
    earningsTimestamp earningsTimestampStart earningsTimestampEnd currency \
}";

pub const SCREENER_RESULTS_COMPOSITE_FIELDS: &[(&str, &str)] = &[
    ("quotes", GQL_SCREENER_QUOTE_FIELDS),
    (
        "pageInfo",
        "{ hasNextPage hasPreviousPage startCursor endCursor }",
    ),
];

// ── Search / Lookup ──────────────────────────────────────────────────────────

pub const GQL_SEARCH_RESULTS_VALID_FIELDS: &[&str] =
    &["count", "quotes", "news", "researchReports", "totalTime"];

pub const SEARCH_RESULTS_COMPOSITE_FIELDS: &[(&str, &str)] = &[
    (
        "quotes",
        "{ symbol shortName longName quoteType exchange exchDisp typeDisp industry industryDisp sector sectorDisp isYahooFinance dispSecIndFlag logoUrl score }",
    ),
    (
        "news",
        "{ uuid title publisher link providerPublishTime type thumbnail relatedTickers }",
    ),
    (
        "researchReports",
        "{ reportHeadline author reportDate id provider }",
    ),
];

pub const GQL_LOOKUP_RESULTS_VALID_FIELDS: &[&str] = &["quotes", "start", "count"];

pub const LOOKUP_RESULTS_COMPOSITE_FIELDS: &[(&str, &str)] = &[(
    "quotes",
    "{ symbol shortName longName quoteType exchange exchDisp typeDisp industry sector regularMarketPrice regularMarketChange regularMarketChangePercent regularMarketPreviousClose logoUrl companyLogoUrl }",
)];

// ── Recommendations ──────────────────────────────────────────────────────────

pub const GQL_RECOMMENDATION_VALID_FIELDS: &[&str] = &["symbol", "recommendations", "providerId"];
pub const RECOMMENDATION_COMPOSITE_FIELDS: &[(&str, &str)] =
    &[("recommendations", "{ symbol score }")];

// ── Financials ───────────────────────────────────────────────────────────

/// Valid fields for `GqlFinancialLineItem` (`{ metric values }`).
pub const GQL_FINANCIAL_LINE_ITEM_VALID_FIELDS: &[&str] = &["metric", "values"];
/// `values` (`[GqlFinancialDataPoint]`) is composite and needs its own nested
/// sub-selection.
pub const FINANCIAL_LINE_ITEM_COMPOSITE_FIELDS: &[(&str, &str)] = &[("values", "{ date value }")];

// ── Holders (6 GraphQL fields, each a structurally distinct type) ───────────

/// `majorHolders` (`GqlMajorHoldersBreakdown`) — all scalar, no nesting needed.
pub const GQL_MAJOR_HOLDERS_VALID_FIELDS: &[&str] = &[
    "insidersPercentHeld",
    "institutionsCount",
    "institutionsFloatPercentHeld",
    "institutionsPercentHeld",
    "maxAge",
];

/// Shared sub-selection for `GqlInstitutionOwner`/`GqlFundOwner` (same shape).
pub const GQL_OWNER_FIELDS: &str =
    "{ maxAge organization pctHeld position value pctChange reportDate }";

/// `institutionalHolders` (`GqlInstitutionOwnership`) — `ownershipList` is
/// `[GqlInstitutionOwner]` (composite).
pub const GQL_INSTITUTIONAL_HOLDERS_VALID_FIELDS: &[&str] = &["maxAge", "ownershipList"];

/// `mutualFundHolders` (`GqlFundOwnership`) — `ownershipList` is
/// `[GqlFundOwner]` (composite, same shape as `GqlInstitutionOwner`).
pub const GQL_MUTUAL_FUND_HOLDERS_VALID_FIELDS: &[&str] = &["maxAge", "ownershipList"];

/// `insiderTransactions` (`GqlInsiderTransactions`) — `transactions` is
/// `[GqlInsiderTransaction]` (composite).
pub const GQL_INSIDER_TRANSACTIONS_VALID_FIELDS: &[&str] = &["maxAge", "transactions"];
pub const GQL_INSIDER_TRANSACTIONS_COMPOSITE: &str = "{ maxAge shares value filerName filerRelation filerUrl moneyText startDate ownership transactionText }";

/// `insiderPurchases` (`GqlNetSharePurchaseActivity`) — all scalar.
pub const GQL_INSIDER_PURCHASES_VALID_FIELDS: &[&str] = &[
    "period",
    "buyInfoCount",
    "buyInfoShares",
    "buyPercentInsiderShares",
    "sellInfoCount",
    "sellInfoShares",
    "sellPercentInsiderShares",
    "netInfoCount",
    "netInfoShares",
    "netPercentInsiderShares",
    "totalInsiderShares",
    "maxAge",
];

/// `insiderRoster` (`GqlInsiderHolders`) — `holders` is `[GqlInsiderHolder]`
/// (composite).
pub const GQL_INSIDER_ROSTER_VALID_FIELDS: &[&str] = &["holders"];
pub const GQL_INSIDER_ROSTER_COMPOSITE: &str = "{ maxAge name relation url transactionDescription latestTransDate positionDirect positionDirectDate positionIndirect positionIndirectDate }";

// ── Analysis (4 GraphQL fields, each a structurally distinct type) ─────────

/// `recommendationTrend` (`GqlRecommendationTrend`) — `trend` is
/// `[GqlRecommendationPeriod]` (composite).
pub const GQL_RECOMMENDATION_TREND_VALID_FIELDS: &[&str] = &["trend", "maxAge"];
pub const GQL_RECOMMENDATION_TREND_COMPOSITE: &str =
    "{ period strongBuy buy hold sell strongSell }";

/// `gradingHistory` (`GqlUpgradeDowngradeHistory`) — `history` is
/// `[GqlGradeChange]` (composite).
pub const GQL_GRADING_HISTORY_VALID_FIELDS: &[&str] = &["history", "maxAge"];
pub const GQL_GRADING_HISTORY_COMPOSITE: &str = "{ epochGradeDate firm fromGrade toGrade action priorPriceTarget currentPriceTarget priceTargetAction }";

/// `earningsEstimate` (`GqlEarningsTrend`) — `trend` is
/// `[GqlEarningsTrendPeriod]` (composite, itself containing further nested
/// composites: earningsEstimate/revenueEstimate/epsTrend/epsRevisions).
pub const GQL_EARNINGS_ESTIMATE_VALID_FIELDS: &[&str] = &["defaultMethodology", "maxAge", "trend"];
pub const GQL_EARNINGS_ESTIMATE_COMPOSITE: &str = "{ endDate earningsEstimate { avg low high yearAgoEps numberOfAnalysts growth earningsCurrency } revenueEstimate { avg low high numberOfAnalysts yearAgoRevenue growth } epsTrend { current sevenDaysAgo thirtyDaysAgo sixtyDaysAgo ninetyDaysAgo } epsRevisions { upLast7Days upLast30Days downLast7Days downLast30Days downLast90Days epsRevisionsCurrency } }";

/// `earningsHistory` (`GqlEarningsHistory`) — `history` is
/// `[GqlEarningsHistoryEntry]` (composite).
pub const GQL_EARNINGS_HISTORY_VALID_FIELDS: &[&str] = &["defaultMethodology", "history"];
pub const GQL_EARNINGS_HISTORY_COMPOSITE: &str =
    "{ maxAge quarter period currency epsActual epsEstimate epsDifference surprisePercent }";

// ── Technical indicators ─────────────────────────────────────────────────────

/// Valid fields for `GqlIndicatorsSummary` — camelCase GraphQL field names
/// (matching `#[graphql(rename_fields = "camelCase")]`, NOT the Rust/serde
/// snake_case names). A handful of these are composite types (stochastic,
/// macd, aroon, etc.) and require their own nested sub-selection — see
/// `INDICATOR_COMPOSITE_FIELDS`.
pub const GQL_INDICATORS_VALID_FIELDS: &[&str] = &[
    "sma10",
    "sma20",
    "sma50",
    "sma100",
    "sma200",
    "ema10",
    "ema20",
    "ema50",
    "ema100",
    "ema200",
    "wma10",
    "wma20",
    "wma50",
    "wma100",
    "wma200",
    "dema20",
    "tema20",
    "hma20",
    "vwma20",
    "alma9",
    "mcginleyDynamic20",
    "rsi14",
    "stochastic",
    "cci20",
    "williamsR14",
    "stochasticRsi",
    "roc12",
    "momentum10",
    "cmo14",
    "awesomeOscillator",
    "coppockCurve",
    "macd",
    "adx14",
    "aroon",
    "supertrend",
    "ichimoku",
    "parabolicSar",
    "bullBearPower",
    "elderRayIndex",
    "bollingerBands",
    "atr14",
    "keltnerChannels",
    "donchianChannels",
    "trueRange",
    "choppinessIndex14",
    "obv",
    "mfi14",
    "cmf20",
    "chaikinOscillator",
    "accumulationDistribution",
    "vwap",
    "balanceOfPower",
];

/// Indicator fields whose GraphQL type is a composite object (not a scalar),
/// paired with their full nested sub-selection. `stochastic`/`stochasticRsi`
/// both use `GqlStochasticData` (`{ k d }`); the rest are 1:1 with their own
/// `Gql*Data` type in `server/src/graphql/types/indicators.rs`.
pub const INDICATOR_COMPOSITE_FIELDS: &[(&str, &str)] = &[
    ("stochastic", "{ k d }"),
    ("stochasticRsi", "{ k d }"),
    ("macd", "{ macd signal histogram }"),
    ("aroon", "{ aroonUp aroonDown }"),
    ("supertrend", "{ value trend }"),
    (
        "ichimoku",
        "{ conversionLine baseLine leadingSpanA leadingSpanB laggingSpan }",
    ),
    ("bollingerBands", "{ upper middle lower }"),
    ("keltnerChannels", "{ upper middle lower }"),
    ("donchianChannels", "{ upper middle lower }"),
    ("bullBearPower", "{ bullPower bearPower }"),
    ("elderRayIndex", "{ bullPower bearPower }"),
];

// ── EDGAR company facts (GqlFactConcept sub-field selection) ───────────────

/// Valid fields for `GqlFactConcept` (each returned XBRL line item).
pub const GQL_EDGAR_FACTS_VALID_FIELDS: &[&str] = &[
    "concept",
    "label",
    "description",
    "taxonomy",
    "unit",
    "dataPoints",
];
pub const EDGAR_FACTS_COMPOSITE_FIELDS: &[(&str, &str)] =
    &[("dataPoints", "{ end val fy fp form }")];

/// Valid GraphQL field names for `GqlEdgarSubmissions` (top-level `edgarSubmissions` field).
pub const GQL_EDGAR_SUBMISSIONS_VALID_FIELDS: &[&str] = &[
    "cik",
    "name",
    "tickers",
    "exchanges",
    "sic",
    "sicDescription",
    "fiscalYearEnd",
    "category",
    "filings",
];
/// `filings` is composite (`GqlEdgarFiling`) and needs its own nested sub-selection.
pub const EDGAR_SUBMISSIONS_COMPOSITE_FIELDS: &[(&str, &str)] = &[(
    "filings",
    "{ accessionNumber filingDate reportDate form size primaryDocument primaryDocDescription }",
)];

// ── Transcripts ──────────────────────────────────────────────────────────────

/// Valid fields for `GqlTranscriptWithMeta`.
pub const GQL_TRANSCRIPT_VALID_FIELDS: &[&str] =
    &["eventId", "quarter", "year", "title", "url", "transcript"];
/// `transcript` (`GqlTranscript`) is composite and needs its own nested
/// sub-selection; stops at whole-transcript `text`, excluding
/// `paragraphs`/`sentences`/`words` (word-level timing data would be huge).
pub const TRANSCRIPT_COMPOSITE_FIELDS: &[(&str, &str)] = &[(
    "transcript",
    "{ transcriptContent { companyId eventId version speakerMapping { speaker speakerData { company name role } } transcript { numberOfSpeakers text } } transcriptMetadata { date eventId eventType fiscalPeriod fiscalYear isLatest s3Url title transcriptId transcriptType updated } }",
)];

// ── Sector / Industry (market-wide, top-level QueryRoot fields) ────────────
//
// Both are large composite objects (overview/performance/benchmark/several
// top-N lists), so every composite top-level field is always expanded with
// ALL of its own sub-fields when selected (no deep configurability below the
// first composite boundary, consistent with every other domain here).

pub const GQL_SECTOR_VALID_FIELDS: &[&str] = &[
    "name",
    "symbol",
    "key",
    "overview",
    "performance",
    "benchmark",
    "benchmarkName",
    "topCompanies",
    "topEtfs",
    "topMutualFunds",
    "industries",
    "researchReports",
];

const GQL_SECTOR_PERFORMANCE_FIELDS: &str = "{ ytdChangePercent dayChangePercent oneYearChangePercent threeYearChangePercent fiveYearChangePercent }";

pub const SECTOR_COMPOSITE_FIELDS: &[(&str, &str)] = &[
    (
        "overview",
        "{ companiesCount marketCap description industriesCount marketWeight employeeCount }",
    ),
    ("performance", GQL_SECTOR_PERFORMANCE_FIELDS),
    ("benchmark", GQL_SECTOR_PERFORMANCE_FIELDS),
    (
        "topCompanies",
        "{ symbol name marketCap marketWeight lastPrice targetPrice dayChangePercent ytdReturn rating }",
    ),
    (
        "topEtfs",
        "{ symbol name netAssets expenseRatio lastPrice ytdReturn }",
    ),
    (
        "topMutualFunds",
        "{ symbol name netAssets expenseRatio lastPrice ytdReturn }",
    ),
    (
        "industries",
        "{ symbol key name marketWeight dayChangePercent ytdReturn }",
    ),
    (
        "researchReports",
        "{ id headline provider reportDate reportTitle reportType targetPrice targetPriceStatus investmentRating }",
    ),
];

pub const GQL_INDUSTRY_VALID_FIELDS: &[&str] = &[
    "name",
    "key",
    "symbol",
    "sectorName",
    "sectorKey",
    "overview",
    "performance",
    "benchmark",
    "topCompanies",
    "topPerformingCompanies",
    "topGrowthCompanies",
    "researchReports",
];

pub const INDUSTRY_COMPOSITE_FIELDS: &[(&str, &str)] = &[
    (
        "overview",
        "{ description companiesCount marketCap marketWeight employeeCount }",
    ),
    (
        "performance",
        "{ dayChangePercent ytdChangePercent oneYearChangePercent threeYearChangePercent fiveYearChangePercent }",
    ),
    (
        "benchmark",
        "{ name dayChangePercent ytdChangePercent oneYearChangePercent threeYearChangePercent fiveYearChangePercent }",
    ),
    (
        "topCompanies",
        "{ symbol name lastPrice marketCap marketWeight dayChangePercent ytdReturn rating targetPrice }",
    ),
    (
        "topPerformingCompanies",
        "{ symbol name lastPrice ytdReturn targetPrice }",
    ),
    (
        "topGrowthCompanies",
        "{ symbol name lastPrice ytdReturn growthEstimate }",
    ),
    (
        "researchReports",
        "{ id title provider reportDate reportType investmentRating targetPrice targetPriceStatus }",
    ),
];
