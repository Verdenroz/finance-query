//! News endpoints: stock news, FMP articles, press releases, crypto news, forex news.

use serde::{Deserialize, Serialize};

use crate::error::Result;

use super::build_client;

// ============================================================================
// Response types
// ============================================================================

/// Stock news article.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct StockNews {
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

/// FMP article from their own editorial.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FmpArticle {
    /// Article title.
    pub title: Option<String>,
    /// Article date.
    pub date: Option<String>,
    /// Article content.
    pub content: Option<String>,
    /// Tickers mentioned.
    pub tickers: Option<String>,
    /// Article image URL.
    pub image: Option<String>,
    /// Article link.
    pub link: Option<String>,
    /// Author.
    pub author: Option<String>,
    /// Site name.
    pub site: Option<String>,
}

/// FMP articles response wrapper (paginated).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FmpArticlesResponse {
    /// Content articles.
    pub content: Option<Vec<FmpArticle>>,
    /// Page number.
    pub page: Option<u32>,
    /// Page size.
    pub size: Option<u32>,
}

/// Press release.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PressRelease {
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
// Public API
// ============================================================================

/// Fetch stock news articles.
///
/// * `tickers` - Comma-separated ticker symbols (e.g., `"AAPL,MSFT"`)
/// * `limit` - Number of results
pub async fn stock_news(tickers: &str, limit: u32) -> Result<Vec<StockNews>> {
    let client = build_client()?;
    let limit_str = limit.to_string();
    client
        .get(
            "/api/v3/stock_news",
            &[("tickers", tickers), ("limit", &limit_str)],
        )
        .await
}

/// Fetch FMP editorial articles.
///
/// * `page` - Page number (0-indexed)
/// * `size` - Page size
pub async fn fmp_articles(page: u32, size: u32) -> Result<FmpArticlesResponse> {
    let client = build_client()?;
    let page_str = page.to_string();
    let size_str = size.to_string();
    client
        .get(
            "/api/v3/fmp/articles",
            &[("page", &*page_str), ("size", &*size_str)],
        )
        .await
}

/// Fetch press releases for a symbol.
pub async fn press_releases(symbol: &str, limit: u32) -> Result<Vec<PressRelease>> {
    let client = build_client()?;
    let path = format!("/api/v3/press-releases/{symbol}");
    let limit_str = limit.to_string();
    client.get(&path, &[("limit", &*limit_str)]).await
}

/// Fetch crypto news.
pub async fn crypto_news(limit: u32) -> Result<Vec<StockNews>> {
    let client = build_client()?;
    let size_str = limit.to_string();
    client
        .get("/api/v4/crypto_news", &[("page", "0"), ("size", &size_str)])
        .await
}

/// Fetch forex news.
pub async fn forex_news(limit: u32) -> Result<Vec<StockNews>> {
    let client = build_client()?;
    let size_str = limit.to_string();
    client
        .get("/api/v4/forex_news", &[("page", "0"), ("size", &size_str)])
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_stock_news_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/api/v3/stock_news")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apikey".into(), "test-key".into()),
                mockito::Matcher::UrlEncoded("tickers".into(), "AAPL".into()),
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

        let client = super::super::build_test_client(&server.url()).unwrap();
        let resp: Vec<StockNews> = client
            .get(
                "/api/v3/stock_news",
                &[("tickers", "AAPL"), ("limit", "5")],
            )
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
            .mock("GET", "/api/v3/press-releases/AAPL")
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

        let client = super::super::build_test_client(&server.url()).unwrap();
        let resp: Vec<PressRelease> = client
            .get("/api/v3/press-releases/AAPL", &[("limit", "10")])
            .await
            .unwrap();
        assert_eq!(resp.len(), 1);
        assert_eq!(resp[0].title.as_deref(), Some("Apple Announces New Product"));
    }
}
