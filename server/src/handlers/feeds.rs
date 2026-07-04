use axum::{
    extract::{Extension, Query},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use finance_query::{ValueFormat, feeds::FeedSource};
use finance_query_server::AppState;
use finance_query_server::services;
use serde::Deserialize;
use tracing::{error, info};

use super::support::{apply_transforms, into_error_response, parse_fields};

/// Query parameters for /v2/feeds
#[derive(Deserialize)]
pub(crate) struct FeedsQuery {
    /// Comma-separated source slugs (see `FeedSourceName` for valid values)
    sources: Option<String>,
    /// SEC form type for sec-filings source (e.g., "10-K", "8-K", default: "10-K")
    form_type: Option<String>,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

/// Canonical slug identifiers accepted by the `/v2/feeds` `sources` query parameter.
///
/// Multiple aliases are accepted (e.g. `"ft"`, `"financial-times"`, `"financialtimes"`), but the
/// primary slug listed here is what appears in OpenAPI documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FeedSourceName {
    FederalReserve,
    Sec,
    SecFilings,
    MarketWatch,
    Cnbc,
    Bloomberg,
    FinancialTimes,
    Nyt,
    Guardian,
    Investing,
    Bea,
    Ecb,
    Cfpb,
    Wsj,
    Fortune,
    BusinessWire,
    CoinDesk,
    CoinTelegraph,
    TechCrunch,
    HackerNews,
    OilPrice,
    CalculatedRisk,
    Scmp,
    NikkeiAsia,
    BankOfEngland,
    VentureBeat,
    YCombinator,
    TheEconomist,
    FinancialPost,
    FtLex,
    RitholtzBigPicture,
}

impl FeedSourceName {
    fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "federal-reserve" | "federalreserve" => Some(Self::FederalReserve),
            "sec" => Some(Self::Sec),
            "sec-filings" | "secfilings" => Some(Self::SecFilings),
            "marketwatch" => Some(Self::MarketWatch),
            "cnbc" => Some(Self::Cnbc),
            "bloomberg" => Some(Self::Bloomberg),
            "ft" | "financial-times" | "financialtimes" => Some(Self::FinancialTimes),
            "nyt" | "nyt-business" | "nytbusiness" => Some(Self::Nyt),
            "guardian" | "guardian-business" | "guardianbusiness" => Some(Self::Guardian),
            "investing" | "investing-com" | "investingcom" => Some(Self::Investing),
            "bea" => Some(Self::Bea),
            "ecb" => Some(Self::Ecb),
            "cfpb" => Some(Self::Cfpb),
            "wsj" | "wsj-markets" | "wsjmarkets" => Some(Self::Wsj),
            "fortune" => Some(Self::Fortune),
            "businesswire" | "business-wire" => Some(Self::BusinessWire),
            "coindesk" | "coin-desk" => Some(Self::CoinDesk),
            "cointelegraph" | "coin-telegraph" => Some(Self::CoinTelegraph),
            "techcrunch" | "tech-crunch" => Some(Self::TechCrunch),
            "hackernews" | "hacker-news" | "hn" => Some(Self::HackerNews),
            "oilprice" | "oil-price" => Some(Self::OilPrice),
            "calculated-risk" | "calculatedrisk" => Some(Self::CalculatedRisk),
            "scmp" | "south-china-morning-post" => Some(Self::Scmp),
            "nikkei" | "nikkei-asia" | "nikkeiasia" => Some(Self::NikkeiAsia),
            "boe" | "bank-of-england" | "bankofengland" => Some(Self::BankOfEngland),
            "venturebeat" | "venture-beat" => Some(Self::VentureBeat),
            "yc" | "ycombinator" | "y-combinator" => Some(Self::YCombinator),
            "economist" | "the-economist" => Some(Self::TheEconomist),
            "financial-post" | "financialpost" => Some(Self::FinancialPost),
            "ft-lex" | "ftlex" | "lex" => Some(Self::FtLex),
            "ritholtz" | "big-picture" | "bigpicture" => Some(Self::RitholtzBigPicture),
            _ => None,
        }
    }

    fn into_feed_source(self, form_type: Option<&str>) -> FeedSource {
        match self {
            Self::FederalReserve => FeedSource::FederalReserve,
            Self::Sec => FeedSource::SecPressReleases,
            Self::SecFilings => FeedSource::SecFilings(form_type.unwrap_or("10-K").to_string()),
            Self::MarketWatch => FeedSource::MarketWatch,
            Self::Cnbc => FeedSource::Cnbc,
            Self::Bloomberg => FeedSource::Bloomberg,
            Self::FinancialTimes => FeedSource::FinancialTimes,
            Self::Nyt => FeedSource::NytBusiness,
            Self::Guardian => FeedSource::GuardianBusiness,
            Self::Investing => FeedSource::Investing,
            Self::Bea => FeedSource::Bea,
            Self::Ecb => FeedSource::Ecb,
            Self::Cfpb => FeedSource::Cfpb,
            Self::Wsj => FeedSource::WsjMarkets,
            Self::Fortune => FeedSource::Fortune,
            Self::BusinessWire => FeedSource::BusinessWire,
            Self::CoinDesk => FeedSource::CoinDesk,
            Self::CoinTelegraph => FeedSource::CoinTelegraph,
            Self::TechCrunch => FeedSource::TechCrunch,
            Self::HackerNews => FeedSource::HackerNews,
            Self::OilPrice => FeedSource::OilPrice,
            Self::CalculatedRisk => FeedSource::CalculatedRisk,
            Self::Scmp => FeedSource::Scmp,
            Self::NikkeiAsia => FeedSource::NikkeiAsia,
            Self::BankOfEngland => FeedSource::BankOfEngland,
            Self::VentureBeat => FeedSource::VentureBeat,
            Self::YCombinator => FeedSource::YCombinator,
            Self::TheEconomist => FeedSource::TheEconomist,
            Self::FinancialPost => FeedSource::FinancialPost,
            Self::FtLex => FeedSource::FtLex,
            Self::RitholtzBigPicture => FeedSource::RitholtzBigPicture,
        }
    }

    const ALL_SLUGS: &'static [&'static str] = &[
        "federal-reserve",
        "sec",
        "sec-filings",
        "marketwatch",
        "cnbc",
        "bloomberg",
        "ft",
        "nyt",
        "guardian",
        "investing",
        "bea",
        "ecb",
        "cfpb",
        "wsj",
        "fortune",
        "businesswire",
        "coindesk",
        "cointelegraph",
        "techcrunch",
        "hackernews",
        "oilprice",
        "calculated-risk",
        "scmp",
        "nikkei",
        "boe",
        "venturebeat",
        "yc",
        "economist",
        "financial-post",
        "ft-lex",
        "ritholtz",
    ];
}

/// GET /v2/feeds
///
/// Query: `sources` (csv, default: all built-in), `form_type` (str, for sec-filings source)
pub(crate) async fn get_feeds(
    Extension(state): Extension<AppState>,
    Query(params): Query<FeedsQuery>,
) -> impl IntoResponse {
    let fields = parse_fields(params.fields.as_deref());
    let source_list = params.sources.as_deref().unwrap_or("all");
    info!("Fetching feeds (sources={})", source_list);

    let sources = match parse_feed_sources(params.sources.as_deref(), params.form_type.as_deref()) {
        Ok(s) => s,
        Err(msg) => {
            let error = serde_json::json!({ "error": msg, "status": 400 });
            return (StatusCode::BAD_REQUEST, Json(error)).into_response();
        }
    };

    match services::feeds::get_feeds(
        &state.cache,
        &sources,
        source_list,
        params.form_type.as_deref().unwrap_or(""),
    )
    .await
    {
        Ok(json) => {
            let response = apply_transforms(json, ValueFormat::default(), fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch feeds: {}", e);
            into_error_response(e)
        }
    }
}

/// Parse comma-separated source slugs into a `Vec<FeedSource>`.
///
/// Returns `Err` with a descriptive message if any slug is unrecognized (caller should 400).
/// Falls back to default sources when `sources` is `None`.
fn parse_feed_sources(
    sources: Option<&str>,
    form_type: Option<&str>,
) -> Result<Vec<FeedSource>, String> {
    let default_sources = || {
        vec![
            FeedSource::FederalReserve,
            FeedSource::SecPressReleases,
            FeedSource::MarketWatch,
            FeedSource::Bloomberg,
        ]
    };

    let Some(sources_str) = sources else {
        return Ok(default_sources());
    };

    let mut parsed = Vec::new();
    for slug in sources_str
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        match FeedSourceName::parse(slug) {
            Some(name) => parsed.push(name.into_feed_source(form_type)),
            None => {
                return Err(format!(
                    "Unknown feed source: '{}'. Valid sources: {}",
                    slug,
                    FeedSourceName::ALL_SLUGS.join(", ")
                ));
            }
        }
    }

    if parsed.is_empty() {
        Ok(default_sources())
    } else {
        Ok(parsed)
    }
}
