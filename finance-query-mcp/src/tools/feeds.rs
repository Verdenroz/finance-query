use finance_query::feeds::{self, FeedSource};
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::{finance_err, ser_err};

fn parse_source(s: &str) -> FeedSource {
    match s.to_lowercase().replace(['-', '_', ' '], "").as_str() {
        "federalreserve" | "fed" => FeedSource::FederalReserve,
        "secpressreleases" | "sec" => FeedSource::SecPressReleases,
        "marketwatch" => FeedSource::MarketWatch,
        "cnbc" => FeedSource::Cnbc,
        "bloomberg" => FeedSource::Bloomberg,
        "financialtimes" | "ft" => FeedSource::FinancialTimes,
        "nytbusiness" | "nyt" => FeedSource::NytBusiness,
        "guardianbusiness" | "guardian" => FeedSource::GuardianBusiness,
        "investing" => FeedSource::Investing,
        "bea" => FeedSource::Bea,
        "ecb" => FeedSource::Ecb,
        "cfpb" => FeedSource::Cfpb,
        "wsjmarkets" | "wsj" => FeedSource::WsjMarkets,
        "fortune" => FeedSource::Fortune,
        "businesswire" => FeedSource::BusinessWire,
        "coindesk" => FeedSource::CoinDesk,
        "cointelegraph" => FeedSource::CoinTelegraph,
        "techcrunch" => FeedSource::TechCrunch,
        "hackernews" | "hn" => FeedSource::HackerNews,
        "oilprice" => FeedSource::OilPrice,
        "calculatedrisk" => FeedSource::CalculatedRisk,
        "scmp" => FeedSource::Scmp,
        "nikkeiasia" | "nikkei" => FeedSource::NikkeiAsia,
        "bankofengland" | "boe" => FeedSource::BankOfEngland,
        "venturebeat" => FeedSource::VentureBeat,
        "ycombinator" | "yc" => FeedSource::YCombinator,
        "theeconomist" | "economist" => FeedSource::TheEconomist,
        "financialpost" => FeedSource::FinancialPost,
        "ftlex" => FeedSource::FtLex,
        "ritholtz" | "bigpicture" => FeedSource::RitholtzBigPicture,
        other => FeedSource::Custom(other.to_string()),
    }
}

pub async fn get_feeds(sources: Option<String>) -> Result<CallToolResult, McpError> {
    let feed_sources: Vec<FeedSource> = match sources {
        Some(s) => s.split(',').map(str::trim).map(parse_source).collect(),
        None => vec![
            FeedSource::MarketWatch,
            FeedSource::Bloomberg,
            FeedSource::WsjMarkets,
            FeedSource::Fortune,
        ],
    };
    let entries = feeds::fetch_all(&feed_sources).await.map_err(finance_err)?;
    let json = serde_json::to_string(&entries).map_err(ser_err)?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(json)]))
}
