//! GraphQL → MCP bridge: field-selection building and error mapping.
//!
//! Mechanism A: `build_selection` validates caller-requested field names against a
//! typed allow-list, then joins them into a GraphQL selection-set string. Field
//! names are spliced directly into the query (they only pass through after exact
//! matching against `VALID_FIELDS`, never as raw caller input), while scalar
//! arguments always go through `async_graphql::Variables`.
//!
//! Mechanism B: `gql_errors_to_mcp` maps `async_graphql::Response::errors` to
//! `McpError`, using the `status` extension when present (same taxonomy as the
//! REST error mapper).

use async_graphql::Response;
pub use finance_query_server::graphql::fields::{
    CALENDAR_EVENT_UNION_SELECTION, DIVIDENDS_COMPOSITE_FIELDS, EDGAR_FACTS_COMPOSITE_FIELDS,
    GQL_CALENDAR_VALID_FIELDS, GQL_CANDLE_VALID_FIELDS, GQL_CHART_META_VALID_FIELDS,
    GQL_CHART_VALID_FIELDS, GQL_COIN_VALID_FIELDS, GQL_DIVIDENDS_VALID_FIELDS,
    GQL_EARNINGS_ESTIMATE_COMPOSITE, GQL_EARNINGS_ESTIMATE_VALID_FIELDS,
    GQL_EARNINGS_HISTORY_COMPOSITE, GQL_EARNINGS_HISTORY_VALID_FIELDS,
    GQL_EDGAR_FACTS_VALID_FIELDS, GQL_FEAR_AND_GREED_VALID_FIELDS, GQL_GRADING_HISTORY_COMPOSITE,
    GQL_GRADING_HISTORY_VALID_FIELDS, GQL_INDICATORS_VALID_FIELDS, GQL_INDUSTRY_VALID_FIELDS,
    GQL_INSIDER_PURCHASES_VALID_FIELDS, GQL_INSIDER_ROSTER_COMPOSITE,
    GQL_INSIDER_ROSTER_VALID_FIELDS, GQL_INSIDER_TRANSACTIONS_COMPOSITE,
    GQL_INSIDER_TRANSACTIONS_VALID_FIELDS, GQL_INSTITUTIONAL_HOLDERS_VALID_FIELDS,
    GQL_LOOKUP_RESULTS_VALID_FIELDS, GQL_MACRO_SERIES_VALID_FIELDS, GQL_MAJOR_HOLDERS_VALID_FIELDS,
    GQL_MARKET_HOURS_VALID_FIELDS, GQL_MARKET_SUMMARY_VALID_FIELDS,
    GQL_MUTUAL_FUND_HOLDERS_VALID_FIELDS, GQL_NEWS_VALID_FIELDS, GQL_OPTIONS_VALID_FIELDS,
    GQL_OWNER_FIELDS, GQL_QUOTE_VALID_FIELDS, GQL_RECOMMENDATION_TREND_COMPOSITE,
    GQL_RECOMMENDATION_TREND_VALID_FIELDS, GQL_RECOMMENDATION_VALID_FIELDS,
    GQL_SCREENER_RESULTS_VALID_FIELDS, GQL_SEARCH_RESULTS_VALID_FIELDS, GQL_SECTOR_VALID_FIELDS,
    GQL_SPARK_VALID_FIELDS, GQL_SPLIT_VALID_FIELDS, GQL_TRANSCRIPT_VALID_FIELDS,
    GQL_TREASURY_YIELD_VALID_FIELDS, GQL_TRENDING_VALID_FIELDS, INDICATOR_COMPOSITE_FIELDS,
    INDUSTRY_COMPOSITE_FIELDS, LOOKUP_RESULTS_COMPOSITE_FIELDS, MACRO_SERIES_COMPOSITE_FIELDS,
    MARKET_HOURS_COMPOSITE_FIELDS, OPTIONS_COMPOSITE_FIELDS, RECOMMENDATION_COMPOSITE_FIELDS,
    SCREENER_RESULTS_COMPOSITE_FIELDS, SEARCH_RESULTS_COMPOSITE_FIELDS, SECTOR_COMPOSITE_FIELDS,
    TRANSCRIPT_COMPOSITE_FIELDS, escape_gql_string, gql_string_list_literal, unwrap_field,
    unwrap_ticker_field,
};
use rmcp::ErrorData as McpError;

use crate::error::invalid_params;

// ── Valid fields per typed GraphQL object ────────────────────────────────────
//
// The `*_VALID_FIELDS`/`*_COMPOSITE_FIELDS` schema-fact data (and
// `escape_gql_string`/`gql_string_list_literal`/`unwrap_ticker_field`/
// `unwrap_field`) live in `finance_query_server::graphql::fields`, shared with
// the REST layer (`server/src/main.rs`) so the two transports can't drift on
// what a GraphQL type's valid field names are. Only `*_DEFAULT_FIELDS` below —
// MCP's curated "when `fields` is omitted" policy, deliberately smaller than
// REST's (which defaults to everything) to keep responses small for an LLM
// context window — stays local to this file.

/// Curated default field set for MCP `get_chart` when `fields` is omitted.
pub const GQL_CHART_DEFAULT_FIELDS: &[&str] = &["symbol", "meta", "candles"];

/// Full field set for `GqlChartMeta` (used as default sub-selection).
pub const GQL_CHART_META_DEFAULT_FIELDS: &[&str] = &[
    "symbol",
    "currency",
    "regularMarketPrice",
    "previousClose",
    "fiftyTwoWeekHigh",
    "fiftyTwoWeekLow",
    "regularMarketDayHigh",
    "regularMarketDayLow",
    "regularMarketVolume",
    "exchangeName",
    "dataGranularity",
    "range",
];

/// Minimal default field set for candles (price + volume only).
pub const GQL_CANDLE_DEFAULT_FIELDS: &[&str] =
    &["timestamp", "open", "high", "low", "close", "volume"];

/// Curated default field set for MCP `get_news` when `fields` is omitted.
pub const GQL_NEWS_DEFAULT_FIELDS: &[&str] = &["title", "link", "source", "time"];

/// Default field set for MCP `get_trending` — all 4 fields, since the type is
/// already tiny (unlike quote/chart there's no bloat to curate away).
pub const GQL_TRENDING_DEFAULT_FIELDS: &[&str] = GQL_TRENDING_VALID_FIELDS;

/// Curated default field set for MCP `get_spark` — close prices + timestamps
/// only; `meta` is the whole non-sparkline payload, dropped by default.
pub const GQL_SPARK_DEFAULT_FIELDS: &[&str] = &["symbol", "timestamps", "closes"];

/// Default fields for MCP `get_market_summary`/`get_fear_and_greed` — both
/// tiny types, no bloat to curate away.
pub const GQL_MARKET_SUMMARY_DEFAULT_FIELDS: &[&str] = GQL_MARKET_SUMMARY_VALID_FIELDS;
pub const GQL_FEAR_AND_GREED_DEFAULT_FIELDS: &[&str] = GQL_FEAR_AND_GREED_VALID_FIELDS;

/// Default fields for MCP `search` — quotes/news are the primary payload;
/// count/totalTime metadata is dropped by default to keep responses small.
pub const GQL_SEARCH_RESULTS_DEFAULT_FIELDS: &[&str] = &["quotes", "news"];

/// Default fields for MCP `lookup` — the quote list is the whole point.
pub const GQL_LOOKUP_RESULTS_DEFAULT_FIELDS: &[&str] = &["quotes"];

/// Default fields for MCP `screener` — the quote list; per-quote-item
/// fields are always fully expanded (no sub-selection below `quotes`, same
/// as REST) since `ScreenerQuote` items are the whole point of the call.
pub const GQL_SCREENER_RESULTS_DEFAULT_FIELDS: &[&str] = &["quotes"];

/// Default fields for MCP `get_dividends`.
pub const GQL_DIVIDENDS_DEFAULT_FIELDS: &[&str] = &["dividends", "analytics"];

/// Default fields for MCP `get_splits` (all fields; splits are small).
pub const GQL_SPLIT_DEFAULT_FIELDS: &[&str] = &["timestamp", "numerator", "denominator", "ratio"];

/// Default fields for MCP `get_options` (all fields; the whole point of a
/// single expiration's chain is usually to see both sides).
pub const GQL_OPTIONS_DEFAULT_FIELDS: &[&str] = GQL_OPTIONS_VALID_FIELDS;

/// Valid fields for `GqlRiskSummary`.
pub const GQL_RISK_VALID_FIELDS: &[&str] = &[
    "var95",
    "var99",
    "parametricVar95",
    "sharpe",
    "sortino",
    "calmar",
    "beta",
    "maxDrawdown",
    "maxDrawdownRecoveryPeriods",
];

/// Default fields for MCP `get_risk` (all fields; risk summary is tiny).
pub const GQL_RISK_DEFAULT_FIELDS: &[&str] = GQL_RISK_VALID_FIELDS;

// ── Holders (6 GraphQL fields, each a structurally distinct type) ───────────
//
// Every holders type's top-level VALID_FIELDS, plus (where a top-level field
// is itself composite — a list of objects) its required nested sub-selection.
// `holder_type` (analysis.rs) selects WHICH of these 6 to query; `fields`
// selects among that type's own top-level fields.

pub const GQL_MAJOR_HOLDERS_DEFAULT_FIELDS: &[&str] = GQL_MAJOR_HOLDERS_VALID_FIELDS;
pub const GQL_INSTITUTIONAL_HOLDERS_DEFAULT_FIELDS: &[&str] =
    GQL_INSTITUTIONAL_HOLDERS_VALID_FIELDS;
pub const GQL_MUTUAL_FUND_HOLDERS_DEFAULT_FIELDS: &[&str] = GQL_MUTUAL_FUND_HOLDERS_VALID_FIELDS;
pub const GQL_INSIDER_TRANSACTIONS_DEFAULT_FIELDS: &[&str] = GQL_INSIDER_TRANSACTIONS_VALID_FIELDS;
pub const GQL_INSIDER_PURCHASES_DEFAULT_FIELDS: &[&str] = GQL_INSIDER_PURCHASES_VALID_FIELDS;
pub const GQL_INSIDER_ROSTER_DEFAULT_FIELDS: &[&str] = GQL_INSIDER_ROSTER_VALID_FIELDS;

/// (GraphQL field name, VALID fields, DEFAULT fields, composite sub-field map).
pub type TypeSpec = (
    &'static str,
    &'static [&'static str],
    &'static [&'static str],
    &'static [(&'static str, &'static str)],
);

/// Per-holder-type spec, keyed by GraphQL field name.
pub const HOLDER_TYPE_SPECS: &[TypeSpec] = &[
    (
        "majorHolders",
        GQL_MAJOR_HOLDERS_VALID_FIELDS,
        GQL_MAJOR_HOLDERS_DEFAULT_FIELDS,
        &[],
    ),
    (
        "institutionalHolders",
        GQL_INSTITUTIONAL_HOLDERS_VALID_FIELDS,
        GQL_INSTITUTIONAL_HOLDERS_DEFAULT_FIELDS,
        &[("ownershipList", GQL_OWNER_FIELDS)],
    ),
    (
        "mutualFundHolders",
        GQL_MUTUAL_FUND_HOLDERS_VALID_FIELDS,
        GQL_MUTUAL_FUND_HOLDERS_DEFAULT_FIELDS,
        &[("ownershipList", GQL_OWNER_FIELDS)],
    ),
    (
        "insiderTransactions",
        GQL_INSIDER_TRANSACTIONS_VALID_FIELDS,
        GQL_INSIDER_TRANSACTIONS_DEFAULT_FIELDS,
        &[("transactions", GQL_INSIDER_TRANSACTIONS_COMPOSITE)],
    ),
    (
        "insiderPurchases",
        GQL_INSIDER_PURCHASES_VALID_FIELDS,
        GQL_INSIDER_PURCHASES_DEFAULT_FIELDS,
        &[],
    ),
    (
        "insiderRoster",
        GQL_INSIDER_ROSTER_VALID_FIELDS,
        GQL_INSIDER_ROSTER_DEFAULT_FIELDS,
        &[("holders", GQL_INSIDER_ROSTER_COMPOSITE)],
    ),
];

// ── Analysis (4 GraphQL fields, each a structurally distinct type) ─────────

pub const GQL_RECOMMENDATION_TREND_DEFAULT_FIELDS: &[&str] = GQL_RECOMMENDATION_TREND_VALID_FIELDS;
pub const GQL_GRADING_HISTORY_DEFAULT_FIELDS: &[&str] = GQL_GRADING_HISTORY_VALID_FIELDS;
pub const GQL_EARNINGS_ESTIMATE_DEFAULT_FIELDS: &[&str] = GQL_EARNINGS_ESTIMATE_VALID_FIELDS;
pub const GQL_EARNINGS_HISTORY_DEFAULT_FIELDS: &[&str] = GQL_EARNINGS_HISTORY_VALID_FIELDS;

/// Per-analysis-type (top-level field name -> (VALID, DEFAULT, composite sub-field map)).
pub const ANALYSIS_TYPE_SPECS: &[TypeSpec] = &[
    (
        "recommendationTrend",
        GQL_RECOMMENDATION_TREND_VALID_FIELDS,
        GQL_RECOMMENDATION_TREND_DEFAULT_FIELDS,
        &[("trend", GQL_RECOMMENDATION_TREND_COMPOSITE)],
    ),
    (
        "gradingHistory",
        GQL_GRADING_HISTORY_VALID_FIELDS,
        GQL_GRADING_HISTORY_DEFAULT_FIELDS,
        &[("history", GQL_GRADING_HISTORY_COMPOSITE)],
    ),
    (
        "earningsEstimate",
        GQL_EARNINGS_ESTIMATE_VALID_FIELDS,
        GQL_EARNINGS_ESTIMATE_DEFAULT_FIELDS,
        &[("trend", GQL_EARNINGS_ESTIMATE_COMPOSITE)],
    ),
    (
        "earningsHistory",
        GQL_EARNINGS_HISTORY_VALID_FIELDS,
        GQL_EARNINGS_HISTORY_DEFAULT_FIELDS,
        &[("history", GQL_EARNINGS_HISTORY_COMPOSITE)],
    ),
];

/// Build a `<gqlField> { ... }` selection set for a holders/analysis type,
/// expanding any composite top-level field with its required nested
/// sub-selection (bare composite fields are invalid GraphQL).
pub fn build_type_spec_selection(
    fields: Option<&[String]>,
    valid_fields: &[&str],
    default_fields: &[&str],
    composite_fields: &[(&str, &str)],
) -> String {
    let mut chosen: Vec<&str> = match fields {
        Some(fs) if !fs.is_empty() => fs
            .iter()
            .map(|f| f.trim())
            .filter(|f| valid_fields.contains(f))
            .collect(),
        _ => default_fields.to_vec(),
    };
    // Every requested name was unknown — fall back to the curated defaults
    // rather than emitting an empty (syntactically invalid) GraphQL selection.
    if chosen.is_empty() {
        chosen = default_fields.to_vec();
    }
    let mut selected = String::from("{ ");
    for f in chosen {
        selected.push_str(f);
        if let Some((_, nested)) = composite_fields.iter().find(|(n, _)| *n == f) {
            selected.push(' ');
            selected.push_str(nested);
        }
        selected.push(' ');
    }
    selected.push('}');
    selected
}

/// Default fields for MCP `get_indicators` (all fields; indicators are flat).
pub const GQL_INDICATORS_DEFAULT_FIELDS: &[&str] = GQL_INDICATORS_VALID_FIELDS;

/// Curated default field set for MCP `get_quote` when `fields` is omitted.
///
/// Target: comfortably under ~10 K tokens for a typical quote response,
/// covering the most commonly useful fields an AI agent will want.
pub const GQL_QUOTE_DEFAULT_FIELDS: &[&str] = &[
    "symbol",
    "regularMarketPrice",
    "regularMarketChange",
    "regularMarketChangePercent",
    "regularMarketTime",
    "shortName",
    "longName",
    "exchange",
    "quoteType",
    "currency",
    "marketState",
    "marketCap",
    "regularMarketVolume",
    "regularMarketDayHigh",
    "regularMarketDayLow",
    "regularMarketOpen",
    "regularMarketPreviousClose",
    "fiftyTwoWeekHigh",
    "fiftyTwoWeekLow",
    "forwardPe",
    "trailingPe",
    "beta",
    "dividendYield",
    "sector",
    "industry",
];

// ── EDGAR company facts (GqlFactConcept sub-field selection) ───────────────

pub const GQL_EDGAR_FACTS_DEFAULT_FIELDS: &[&str] = &["concept", "unit", "dataPoints"];

// ── Transcripts ──────────────────────────────────────────────────────────────

/// Default fields: excludes nothing at the top level (the bloat-avoidance
/// happens inside the `transcript` composite sub-selection, which stops at
/// paragraph-text level and excludes sentence/word-level timing data).
pub const GQL_TRANSCRIPT_DEFAULT_FIELDS: &[&str] = GQL_TRANSCRIPT_VALID_FIELDS;

// ── Sector / Industry (market-wide, top-level QueryRoot fields) ────────────
//
// Both are large composite objects (overview/performance/benchmark/several
// top-N lists), so every composite top-level field is always expanded with
// ALL of its own sub-fields when selected (no deep configurability below the
// first composite boundary, consistent with every other domain in this file).

pub const GQL_SECTOR_DEFAULT_FIELDS: &[&str] = GQL_SECTOR_VALID_FIELDS;
pub const GQL_INDUSTRY_DEFAULT_FIELDS: &[&str] = GQL_INDUSTRY_VALID_FIELDS;

// ── `fields`/`metrics`/`concepts` CSV param parsing ──────────────────────────

/// Parse a comma-separated tool param (`fields`, `metrics`, `concepts`, ...)
/// into a trimmed, non-empty list — `None` stays `None` (caller didn't ask).
pub fn parse_fields(csv: Option<String>) -> Option<Vec<String>> {
    csv.map(|f| {
        f.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    })
}

// ── Selection-set building (Mechanism A) ─────────────────────────────────────

/// Build a GraphQL selection-set string from a list of requested field names.
///
/// Each name is validated against `valid_fields`. Unknown fields are silently
/// skipped (the caller gets exactly what was asked for and validated). The
/// result is always wrapped in `{ ... }`.
///
/// Because we only interpolate strings that passed exact-match against our own
/// compile-time `&[&str]` allow-list, this is safe against GraphQL injection.
pub fn build_selection(fields: &[String], valid_fields: &[&str]) -> String {
    let mut selected = String::from("{ ");
    for f in fields {
        let f = f.trim();
        if !f.is_empty() && valid_fields.contains(&f) {
            selected.push_str(f);
            selected.push(' ');
        }
    }
    selected.push('}');
    selected
}

/// Build a selection set from explicit fields, falling back to `default_fields`
/// when `fields` is `None`, empty, or matches no `valid_fields` entry (an
/// empty selection set is invalid GraphQL syntax, so a caller typo must fall
/// back rather than produce a hard parse error).
pub fn build_selection_or_default(
    fields: Option<&[String]>,
    valid_fields: &[&str],
    default_fields: &[&str],
) -> String {
    let selection = fields
        .filter(|fs| !fs.is_empty())
        .map(|fs| build_selection(fs, valid_fields));
    match selection {
        Some(sel) if sel != "{ }" => sel,
        _ => build_selection_all(default_fields),
    }
}

fn build_selection_all(fields: &[&str]) -> String {
    let mut selected = String::from("{ ");
    for f in fields {
        selected.push_str(f);
        selected.push(' ');
    }
    selected.push('}');
    selected
}

// ── GraphQL → MCP error mapping (Mechanism B) ────────────────────────────────

/// Map `async_graphql::Response::errors` to an `McpError`.
///
/// When *any* error in the list carries a `status` extension (set by
/// `finance_error_to_gql`), we map it to `McpError::internal_error`
/// (message passthrough), matching today's `finance_err` behaviour.
///
/// Errors without a `status` extension (pure GraphQL-layer: parse failure,
/// unknown field, depth/complexity limit exceeded) are mapped to
/// `McpError::invalid_params`.
pub fn gql_errors_to_mcp(errors: &[async_graphql::ServerError]) -> McpError {
    // Check if any error has a `status` extension (library-level error).
    let has_status = errors.iter().any(|e| {
        e.extensions
            .as_ref()
            .and_then(|ext| ext.get("status"))
            .is_some()
    });

    if has_status {
        // Passthrough: join all error messages.
        let msg: String = errors
            .iter()
            .map(|e| e.message.as_str())
            .collect::<Vec<_>>()
            .join("; ");
        McpError::internal_error(msg, None)
    } else {
        // Pure GraphQL-layer error (parse / validation / limits).
        let msg: String = errors
            .iter()
            .map(|e| e.message.as_str())
            .collect::<Vec<_>>()
            .join("; ");
        invalid_params(msg)
    }
}

/// Execute a schema query and unwrap the top-level `data` key.
///
/// On error, maps via `gql_errors_to_mcp`. On success but with empty data
/// (unexpected), returns an internal error.
pub async fn execute_query(
    schema: &async_graphql::Schema<
        finance_query_server::graphql::query::QueryRoot,
        async_graphql::EmptyMutation,
        finance_query_server::graphql::subscription::SubscriptionRoot,
    >,
    query: &str,
    variables: async_graphql::Variables,
) -> Result<serde_json::Value, McpError> {
    let response: Response = schema
        .execute(async_graphql::Request::new(query).variables(variables))
        .await;

    if !response.errors.is_empty() {
        return Err(gql_errors_to_mcp(&response.errors));
    }

    response.data.into_json().map_err(|e| {
        McpError::internal_error(format!("Failed to serialize GraphQL response: {e}"), None)
    })
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn fields(names: &[&str]) -> Vec<String> {
        names.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn build_selection_includes_exactly_the_requested_valid_fields() {
        let requested = fields(&["symbol", "shortName"]);
        let selection = build_selection(&requested, GQL_TRENDING_VALID_FIELDS);

        assert!(selection.contains("symbol"));
        assert!(selection.contains("shortName"));
        assert!(!selection.contains("regularMarketPrice"));
        assert!(!selection.contains("regularMarketChangePercent"));
    }

    #[test]
    fn build_selection_drops_unknown_field_names() {
        let requested = fields(&["symbol", "bogusField"]);
        let selection = build_selection(&requested, GQL_TRENDING_VALID_FIELDS);

        assert!(selection.contains("symbol"));
        assert!(!selection.contains("bogusField"));
    }

    #[test]
    fn build_selection_drops_injection_shaped_strings() {
        // Neither string exact-matches an entry in VALID_FIELDS, so both are
        // dropped rather than spliced into the query text verbatim.
        let requested = fields(&["\") { __schema", "price __typename"]);
        let selection = build_selection(&requested, GQL_TRENDING_VALID_FIELDS);

        assert_eq!(selection, "{ }");
        assert!(!selection.contains("__schema"));
        assert!(!selection.contains("__typename"));
    }

    #[test]
    fn build_selection_or_default_uses_valid_requested_fields() {
        let requested = fields(&["symbol"]);
        let selection = build_selection_or_default(
            Some(&requested),
            GQL_TRENDING_VALID_FIELDS,
            GQL_TRENDING_DEFAULT_FIELDS,
        );

        assert_eq!(selection, "{ symbol }");
    }

    #[test]
    fn build_selection_or_default_falls_back_when_fields_is_none() {
        let selection = build_selection_or_default(
            None,
            GQL_TRENDING_VALID_FIELDS,
            GQL_TRENDING_DEFAULT_FIELDS,
        );

        for f in GQL_TRENDING_DEFAULT_FIELDS {
            assert!(selection.contains(f));
        }
    }

    #[test]
    fn build_selection_or_default_falls_back_when_fields_is_empty() {
        let empty: Vec<String> = Vec::new();
        let selection = build_selection_or_default(
            Some(&empty),
            GQL_TRENDING_VALID_FIELDS,
            GQL_TRENDING_DEFAULT_FIELDS,
        );

        for f in GQL_TRENDING_DEFAULT_FIELDS {
            assert!(selection.contains(f));
        }
    }

    #[test]
    fn build_selection_or_default_drops_unknown_fields_from_requested_list() {
        let requested = fields(&["symbol", "__schema"]);
        let selection = build_selection_or_default(
            Some(&requested),
            GQL_QUOTE_VALID_FIELDS,
            GQL_QUOTE_DEFAULT_FIELDS,
        );

        assert_eq!(selection, "{ symbol }");
    }

    #[test]
    fn build_selection_or_default_falls_back_when_every_requested_field_is_unknown() {
        // Regression: previously produced a bare "{ }", which is invalid
        // GraphQL syntax and surfaced as a confusing parser error to callers.
        let requested = fields(&["bogus1", "bogus2"]);
        let selection = build_selection_or_default(
            Some(&requested),
            GQL_QUOTE_VALID_FIELDS,
            GQL_QUOTE_DEFAULT_FIELDS,
        );

        assert_ne!(selection, "{ }");
        for f in GQL_QUOTE_DEFAULT_FIELDS {
            assert!(selection.contains(f));
        }
    }

    #[test]
    fn build_type_spec_selection_falls_back_when_every_requested_field_is_unknown() {
        let requested = fields(&["bogus1", "bogus2"]);
        let selection = build_type_spec_selection(
            Some(&requested),
            GQL_RECOMMENDATION_TREND_VALID_FIELDS,
            GQL_RECOMMENDATION_TREND_DEFAULT_FIELDS,
            &[("trend", GQL_RECOMMENDATION_TREND_COMPOSITE)],
        );

        assert_ne!(selection, "{ }");
        for f in GQL_RECOMMENDATION_TREND_DEFAULT_FIELDS {
            assert!(selection.contains(f));
        }
    }
}
