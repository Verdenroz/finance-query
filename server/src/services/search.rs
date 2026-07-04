use crate::cache::{self, Cache};
use finance_query::{Region, Tickers, finance};

use super::{ServiceError, ServiceResult, lang_key};

/// Boolean feature flags for a search request.
#[derive(Clone, Copy, Default)]
pub struct SearchFlags {
    pub fuzzy: bool,
    pub logo: bool,
    pub research: bool,
    pub cultural: bool,
}

pub async fn search(
    cache: &Cache,
    query: &str,
    quotes: u32,
    news: u32,
    flags: SearchFlags,
    region: Option<Region>,
    lang: Option<&str>,
) -> ServiceResult {
    let cache_key = Cache::key(
        "search",
        &[
            &query.to_lowercase(),
            &quotes.to_string(),
            &news.to_string(),
            if flags.logo { "1" } else { "0" },
            lang_key(lang),
        ],
    );
    let query = query.to_string();
    let lang = lang.map(str::to_string);

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::SEARCH,
            cache::is_market_open(),
            || async move {
                let mut options = finance::SearchOptions::new()
                    .quotes_count(quotes)
                    .news_count(news)
                    .enable_fuzzy_query(flags.fuzzy)
                    .enable_logo_url(flags.logo)
                    .enable_research_reports(flags.research)
                    .enable_cultural_assets(flags.cultural);

                if let Some(r) = region {
                    options = options.region(r);
                }

                let mut result = finance::search(&query, &options).await?;
                super::translate(&mut result, lang.as_deref()).await?;
                serde_json::to_value(&result).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

pub async fn lookup(
    cache: &Cache,
    query: &str,
    lookup_type: finance::LookupType,
    count: u32,
    logo: bool,
    region: Option<Region>,
    lang: Option<&str>,
) -> ServiceResult {
    let type_str = format!("{:?}", lookup_type).to_lowercase();
    let cache_key = Cache::key(
        "lookup",
        &[
            &query.to_lowercase(),
            &type_str,
            &count.to_string(),
            if logo { "1" } else { "0" },
            lang_key(lang),
        ],
    );
    let query = query.to_string();
    let lang = lang.map(str::to_string);

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::SEARCH,
            cache::is_market_open(),
            || async move {
                let mut options = finance::LookupOptions::new()
                    .lookup_type(lookup_type)
                    .count(count)
                    .include_logo(logo);

                if let Some(r) = region {
                    options = options.region(r);
                }

                let mut result = finance::lookup(&query, &options).await?;
                super::translate(&mut result, lang.as_deref()).await?;

                // Yahoo's lookup endpoint only returns a lightweight document
                // shape (symbol/shortName/exchange/quoteType/price) — enrich
                // each with a batch quote lookup so longName/changePercent/
                // previousClose aren't null. `sector`/`industry` stay null even
                // after enrichment: they live in Yahoo's assetProfile
                // quoteSummary module, which the lightweight batch quotes
                // endpoint doesn't include — populating them would need a full
                // per-symbol quoteSummary fetch per result, defeating the
                // point of `lookup` being a fast, cheap discovery call.
                let symbols: Vec<&str> = result.quotes.iter().map(|q| q.symbol.as_str()).collect();
                let quotes = if symbols.is_empty() {
                    std::collections::HashMap::new()
                } else {
                    Tickers::builder(symbols)
                        .build()
                        .await?
                        .quotes()
                        .await?
                        .quotes
                };
                // `Quote`'s numeric fields are format-generic (`F::Value<f64>`)
                // and may serialize as `{"raw": .., "fmt": ..}` rather than a
                // bare number — `LookupQuote`'s fields are plain `f64`/`String`,
                // so normalize each enrichment value to the scalar shape
                // lookup expects instead of copying it through as-is.
                fn as_scalar(v: &serde_json::Value) -> Option<serde_json::Value> {
                    match v {
                        serde_json::Value::Object(o) => o.get("raw").cloned(),
                        serde_json::Value::Null => None,
                        other => Some(other.clone()),
                    }
                }

                let enriched_quotes: Vec<serde_json::Value> = result
                    .quotes
                    .iter()
                    .map(|lq| {
                        let mut v = serde_json::to_value(lq).unwrap_or_default();
                        if let Some(q) = quotes.get(&lq.symbol)
                            && let Ok(qv) = serde_json::to_value(q)
                            && let Some(obj) = v.as_object_mut()
                        {
                            for field in [
                                "longName",
                                "regularMarketChangePercent",
                                "regularMarketPreviousClose",
                            ] {
                                if obj.get(field).map(|f| f.is_null()).unwrap_or(true)
                                    && let Some(val) = qv.get(field).and_then(as_scalar)
                                {
                                    obj.insert(field.to_string(), val);
                                }
                            }
                        }
                        v
                    })
                    .collect();

                let mut result_json =
                    serde_json::to_value(&result).map_err(|e| Box::new(e) as ServiceError)?;
                if let Some(obj) = result_json.as_object_mut() {
                    obj.insert(
                        "quotes".to_string(),
                        serde_json::to_value(enriched_quotes)
                            .map_err(|e| Box::new(e) as ServiceError)?,
                    );
                }
                Ok(result_json)
            },
        )
        .await
}
