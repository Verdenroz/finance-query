use crate::Provider;
/// News adapter for Yahoo Finance provider.
///
/// Wraps the StockAnalysis news scraper and sets Yahoo provider_id.
use crate::error::Result;
use crate::models::corporate::news::News;

/// Fetch news for a symbol using the StockAnalysis scraper.
///
/// Scrapes stockanalysis.com for news articles and annotates
/// each article with the Yahoo Finance provider identifier.
pub(crate) async fn fetch_news(symbol: &str) -> Result<Vec<News>> {
    let news = crate::scrapers::stockanalysis::scrape_symbol_news(symbol).await?;
    Ok(news
        .into_iter()
        .map(|n| News {
            title: n.title,
            link: n.link,
            source: n.source,
            img: String::new(),
            time: n.time,
            provider_id: Some(Provider::Yahoo),
        })
        .collect())
}
