//! News endpoints: stock news, FMP articles, press releases, crypto news, forex news.

use serde::{Deserialize, Serialize};

use crate::error::Result;

use crate::adapters::fmp::build_client;

// ============================================================================
// Response types
// ============================================================================

/// Stock news article.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct StockNewsDTO {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Published date.
    #[serde(rename = "publishedDate")]
    pub published_date: Option<String>,
    /// Article title.
    pub title: Option<String>,
    /// Article image URL.
    pub image: Option<String>,
    /// News site name.
    pub site: Option<String>,
    /// Article text / summary.
    pub text: Option<String>,
    /// Article URL.
    pub url: Option<String>,
}

/// Press release.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PressReleaseDTO {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Date.
    pub date: Option<String>,
    /// Title.
    pub title: Option<String>,
    /// Full text.
    pub text: Option<String>,
}

// ============================================================================
// Canonical conversion functions
// ============================================================================

/// Convert stock news DTOs into canonical News items.
fn stock_news_to_canonical(
    articles: Vec<StockNewsDTO>,
) -> Vec<crate::models::corporate::news::News> {
    articles
        .into_iter()
        .map(|a| crate::models::corporate::news::News {
            title: a.title.unwrap_or_default(),
            link: a.url.unwrap_or_default(),
            source: a.site.unwrap_or_default(),
            img: String::new(),
            time: a.published_date.unwrap_or_default(),
            provider_id: Some(crate::providers::Provider::Fmp),
            #[cfg(feature = "sentiment")]
            sentiment: None,
        })
        .collect()
}

/// Fetch canonical news for a symbol.
pub async fn fetch_canonical_news(
    symbol: &str,
    limit: u32,
) -> Result<Vec<crate::models::corporate::news::News>> {
    let articles = stock_news(symbol, limit).await?;
    Ok(stock_news_to_canonical(articles))
}

// ============================================================================
// Public API
// ============================================================================

/// Fetch stock news articles.
///
/// * `tickers` - Comma-separated ticker symbols (e.g., `"AAPL,MSFT"`)
/// * `limit` - Number of results
pub async fn stock_news(tickers: &str, limit: u32) -> Result<Vec<StockNewsDTO>> {
    let client = build_client()?;
    let limit_str = limit.to_string();
    client
        .get(
            "/stable/news/stock",
            &[("symbols", tickers), ("limit", &limit_str)],
        )
        .await
}

/// Fetch press releases for a symbol.
pub async fn press_releases(symbol: &str, limit: u32) -> Result<Vec<PressReleaseDTO>> {
    let client = build_client()?;
    let limit_str = limit.to_string();
    client
        .get(
            "/stable/news/press-releases",
            &[("symbols", symbol), ("limit", &limit_str)],
        )
        .await
}

/// Fetch crypto news.
#[allow(dead_code)] // unrouted: category news deliberately deferred (#300 optional)
pub async fn crypto_news(limit: u32) -> Result<Vec<StockNewsDTO>> {
    let client = build_client()?;
    let limit_str = limit.to_string();
    client
        .get(
            "/stable/news/crypto-latest",
            &[("page", "0"), ("limit", &limit_str)],
        )
        .await
}

/// Fetch forex news.
#[allow(dead_code)] // unrouted: category news deliberately deferred (#300 optional)
pub async fn forex_news(limit: u32) -> Result<Vec<StockNewsDTO>> {
    let client = build_client()?;
    let limit_str = limit.to_string();
    client
        .get(
            "/stable/news/forex-latest",
            &[("page", "0"), ("limit", &limit_str)],
        )
        .await
}

/// Fetch canonical press releases for a symbol.
pub async fn fetch_press_releases_response(
    symbol: &str,
    limit: u32,
) -> Result<Vec<crate::models::corporate::press_release::PressRelease>> {
    let dtos = press_releases(symbol, limit).await?;
    Ok(dtos
        .into_iter()
        .map(|d| crate::models::corporate::press_release::PressRelease {
            symbol: d.symbol.or_else(|| Some(symbol.to_string())),
            date: d.date,
            title: d.title,
            text: d.text,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_stock_news_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/stable/news/stock")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apikey".into(), "test-key".into()),
                mockito::Matcher::UrlEncoded("symbols".into(), "AAPL".into()),
                mockito::Matcher::UrlEncoded("limit".into(), "5".into()),
            ]))
            .with_status(200)
            .with_body(
                serde_json::json!([
                    {
                        "symbol": "AAPL",
                        "publishedDate": "2024-01-15 12:00:00",
                        "title": "Apple Reports Record Quarter",
                        "image": "https://example.com/image.jpg",
                        "site": "Reuters",
                        "text": "Apple Inc. reported record quarterly earnings...",
                        "url": "https://example.com/article"
                    }
                ])
                .to_string(),
            )
            .create_async()
            .await;

        let client = crate::adapters::fmp::build_test_client(&server.url()).unwrap();
        let resp: Vec<StockNewsDTO> = client
            .get("/stable/news/stock", &[("symbols", "AAPL"), ("limit", "5")])
            .await
            .unwrap();
        assert_eq!(resp.len(), 1);
        assert_eq!(resp[0].symbol.as_deref(), Some("AAPL"));
        assert_eq!(resp[0].site.as_deref(), Some("Reuters"));
    }

    #[tokio::test]
    async fn test_press_releases_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/stable/news/press-releases")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apikey".into(), "test-key".into()),
                mockito::Matcher::UrlEncoded("limit".into(), "10".into()),
            ]))
            .with_status(200)
            .with_body(
                serde_json::json!([
                    {
                        "symbol": "AAPL",
                        "date": "2024-01-15",
                        "title": "Apple Announces New Product",
                        "text": "Cupertino, CA -- Apple today announced..."
                    }
                ])
                .to_string(),
            )
            .create_async()
            .await;

        let client = crate::adapters::fmp::build_test_client(&server.url()).unwrap();
        let resp: Vec<PressReleaseDTO> = client
            .get("/stable/news/press-releases", &[("limit", "10")])
            .await
            .unwrap();
        assert_eq!(resp.len(), 1);
        assert_eq!(
            resp[0].title.as_deref(),
            Some("Apple Announces New Product")
        );
    }
}
