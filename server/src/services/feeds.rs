use crate::cache::{self, Cache};
use finance_query::feeds::FeedSource;

use super::{ServiceError, ServiceResult};

/// Canonical slug identifiers accepted by the `sources` param on the `feeds`
/// GraphQL field (and, transitively, REST's `/v2/feeds` and MCP's
/// `get_feeds`), shared so all three surfaces validate against — and report
/// errors listing — the exact same set.
const FEED_SOURCE_SLUGS: &[&str] = &[
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

fn normalize_slug(slug: &str) -> String {
    slug.trim().to_lowercase().replace(['-', '_', ' '], "")
}

/// Parse one slug into its `FeedSource`, normalizing away `-`/`_`/space so
/// `"sec-filings"`, `"sec_filings"`, and `"secfilings"` are all accepted.
fn parse_source_slug(slug: &str, form_type: Option<&str>) -> Option<FeedSource> {
    match normalize_slug(slug).as_str() {
        "federalreserve" => Some(FeedSource::FederalReserve),
        "sec" => Some(FeedSource::SecPressReleases),
        "secfilings" => Some(FeedSource::SecFilings(
            form_type.unwrap_or("10-K").to_string(),
        )),
        "marketwatch" => Some(FeedSource::MarketWatch),
        "cnbc" => Some(FeedSource::Cnbc),
        "bloomberg" => Some(FeedSource::Bloomberg),
        "ft" | "financialtimes" => Some(FeedSource::FinancialTimes),
        "nyt" | "nytbusiness" => Some(FeedSource::NytBusiness),
        "guardian" | "guardianbusiness" => Some(FeedSource::GuardianBusiness),
        "investing" | "investingcom" => Some(FeedSource::Investing),
        "bea" => Some(FeedSource::Bea),
        "ecb" => Some(FeedSource::Ecb),
        "cfpb" => Some(FeedSource::Cfpb),
        "wsj" | "wsjmarkets" => Some(FeedSource::WsjMarkets),
        "fortune" => Some(FeedSource::Fortune),
        "businesswire" => Some(FeedSource::BusinessWire),
        "coindesk" => Some(FeedSource::CoinDesk),
        "cointelegraph" => Some(FeedSource::CoinTelegraph),
        "techcrunch" => Some(FeedSource::TechCrunch),
        "hackernews" | "hn" => Some(FeedSource::HackerNews),
        "oilprice" => Some(FeedSource::OilPrice),
        "calculatedrisk" => Some(FeedSource::CalculatedRisk),
        "scmp" | "southchinamorningpost" => Some(FeedSource::Scmp),
        "nikkei" | "nikkeiasia" => Some(FeedSource::NikkeiAsia),
        "boe" | "bankofengland" => Some(FeedSource::BankOfEngland),
        "venturebeat" => Some(FeedSource::VentureBeat),
        "yc" | "ycombinator" => Some(FeedSource::YCombinator),
        "economist" | "theeconomist" => Some(FeedSource::TheEconomist),
        "financialpost" => Some(FeedSource::FinancialPost),
        "ftlex" | "lex" => Some(FeedSource::FtLex),
        "ritholtz" | "bigpicture" => Some(FeedSource::RitholtzBigPicture),
        _ => None,
    }
}

/// Parse feed-source slugs into `FeedSource`s, shared by the `feeds` GraphQL
/// field (and thus REST and MCP, both of which bridge through it).
///
/// `None`/empty → 4 built-in defaults (Fed, SEC, MarketWatch, Bloomberg).
/// Returns `Err` listing every valid slug if any requested one is unrecognized.
pub fn parse_sources(
    slugs: Option<&[String]>,
    form_type: Option<&str>,
) -> Result<Vec<FeedSource>, String> {
    if form_type.is_some()
        && !slugs.is_some_and(|s| s.iter().any(|s| normalize_slug(s) == "secfilings"))
    {
        return Err("formType only applies to the sec-filings source".to_string());
    }

    let default_sources = || {
        vec![
            FeedSource::FederalReserve,
            FeedSource::SecPressReleases,
            FeedSource::MarketWatch,
            FeedSource::Bloomberg,
        ]
    };

    let Some(slugs) = slugs else {
        return Ok(default_sources());
    };

    let mut parsed = Vec::new();
    for slug in slugs.iter().map(|s| s.trim()).filter(|s| !s.is_empty()) {
        match parse_source_slug(slug, form_type) {
            Some(source) => parsed.push(source),
            None => {
                return Err(format!(
                    "Unknown feed source: '{slug}'. Valid sources: {}",
                    FEED_SOURCE_SLUGS.join(", ")
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

pub async fn get_feeds(
    cache: &Cache,
    sources: &[FeedSource],
    source_key: &str,
    form_type_key: &str,
) -> ServiceResult {
    let cache_key = Cache::key("feeds", &[source_key, form_type_key]);
    let sources = sources.to_vec();

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::GENERAL_NEWS,
            cache::is_market_open(),
            || async move {
                let entries = finance_query::feeds::fetch_all(sources).await?;
                serde_json::to_value(&entries).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slugs(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn form_type_with_sec_filings_is_applied() {
        let parsed = parse_sources(Some(&slugs(&["sec-filings"])), Some("8-K")).unwrap();
        assert!(matches!(parsed.as_slice(), [FeedSource::SecFilings(f)] if f == "8-K"));
    }

    #[test]
    fn form_type_without_sec_filings_is_rejected() {
        let err = parse_sources(Some(&slugs(&["sec"])), Some("8-K")).unwrap_err();
        assert!(err.contains("sec-filings"), "{err}");
        assert!(parse_sources(None, Some("8-K")).is_err());
    }

    #[test]
    fn omitted_form_type_leaves_sources_unchanged() {
        assert!(parse_sources(Some(&slugs(&["sec"])), None).is_ok());
    }
}
