//! GDELT DOC 2.0 — keyless global news search and monitoring.
//!
//! Requires the **`gdelt`** feature flag.
//!
//! News previously came from Yahoo (a scraper, fragile to page changes) or
//! Alpha Vantage (keyed, 25 req/day). GDELT indexes worldwide online news
//! across 65 languages, updated roughly every 15 minutes, entirely keyless —
//! closing that gap for the news slice of `Capability::CORPORATE`.
//!
//! # Query derivation
//!
//! GDELT has no ticker vocabulary of its own, so [`fetch_news_response`]
//! (via [`corporate::build_query`]) searches GDELT for the ticker symbol
//! itself, quoted for an exact phrase match. See that function's doc comment
//! for the precision/recall tradeoff this implies.
//!
//! GDELT has no concept of a corporate calendar (earnings/dividends/splits),
//! so the provider bridge reports [`crate::error::FinanceError::NotSupported`]
//! for that operation rather than implementing it here.

pub(crate) mod client;
pub(crate) mod corporate;
pub(crate) mod models;

use std::time::Duration;

use crate::adapters::singleton::keyless_limiter;
use crate::error::Result;
use client::GdeltClient;

/// Self-imposed pacing: GDELT asks callers to keep requests to roughly one
/// every 5 seconds.
const GDELT_RATE_PER_SEC: f64 = 0.2;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

keyless_limiter!(rate = GDELT_RATE_PER_SEC);

/// Build a client against the live API, reusing the shared token bucket.
fn client() -> Result<GdeltClient> {
    GdeltClient::new(DEFAULT_TIMEOUT, shared_limiter(), client::GDELT_BASE)
}

pub(crate) use corporate::fetch_news_response;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::FinanceError;
    use crate::rate_limiter::RateLimiter;
    use std::sync::Arc;

    fn test_client(base_url: &str) -> GdeltClient {
        GdeltClient::new(
            Duration::from_secs(5),
            Arc::new(RateLimiter::new(100.0)),
            base_url,
        )
        .unwrap()
    }

    /// Verbatim shape from `api.gdeltproject.org/api/v2/doc/doc`, trimmed to
    /// two articles.
    fn articles_payload() -> String {
        serde_json::json!({
            "articles": [
                {
                    "url": "https://example.com/news/aapl-earnings",
                    "url_mobile": "",
                    "title": "Apple (NASDAQ: AAPL) beats earnings estimates",
                    "seendate": "20260805T113000Z",
                    "socialimage": "https://example.com/img.jpg",
                    "domain": "example.com",
                    "language": "English",
                    "sourcecountry": "United States"
                },
                {
                    "url": "https://example.fr/actu/aapl",
                    "title": "Apple annonce des resultats records",
                    "seendate": "20260805T090000Z",
                    "domain": "example.fr",
                    "language": "French",
                    "sourcecountry": "France"
                }
            ]
        })
        .to_string()
    }

    #[tokio::test]
    async fn articles_map_to_canonical_news() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(articles_payload())
            .create_async()
            .await;

        let response = test_client(&server.url())
            .article_search("\"AAPL\"", "2w", 50)
            .await
            .unwrap();

        assert_eq!(response.articles.len(), 2);
        let news: Vec<_> = response
            .articles
            .into_iter()
            .map(super::corporate::to_news)
            .collect();
        assert_eq!(news[0].link, "https://example.com/news/aapl-earnings");
        assert_eq!(news[0].source, "example.com");
        assert_eq!(news[0].img, "https://example.com/img.jpg");
        // No socialimage on the second article — must not become "null".
        assert_eq!(news[1].img, "");
    }

    #[tokio::test]
    async fn throttled_response_maps_to_rate_limited() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/")
            .match_query(mockito::Matcher::Any)
            .with_status(429)
            .create_async()
            .await;

        let err = test_client(&server.url())
            .article_search("\"AAPL\"", "2w", 50)
            .await
            .unwrap_err();
        assert!(matches!(err, FinanceError::RateLimited { .. }), "{err:?}");
    }

    #[tokio::test]
    async fn plain_text_throttle_message_is_not_treated_as_valid_json() {
        // GDELT answers over-quota requests with a 200 and a plain-text
        // message rather than JSON — must not silently parse as zero articles.
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_body("Please limit requests to one every 5 seconds")
            .create_async()
            .await;

        let err = test_client(&server.url())
            .article_search("\"AAPL\"", "2w", 50)
            .await
            .unwrap_err();
        match err {
            FinanceError::ResponseStructureError { context, .. } => {
                assert!(context.contains("limit requests"), "got {context}");
            }
            other => panic!("expected ResponseStructureError, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn http_error_maps_to_external_api_error() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/")
            .match_query(mockito::Matcher::Any)
            .with_status(503)
            .create_async()
            .await;

        let err = test_client(&server.url())
            .article_search("\"AAPL\"", "2w", 50)
            .await
            .unwrap_err();
        assert!(
            matches!(err, FinanceError::ExternalApiError { status: 503, .. }),
            "{err:?}"
        );
    }

    #[tokio::test]
    async fn empty_articles_array_maps_to_an_empty_vec() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(serde_json::json!({"articles": []}).to_string())
            .create_async()
            .await;

        let response = test_client(&server.url())
            .article_search("\"NOSUCHTICKER\"", "2w", 50)
            .await
            .unwrap();
        assert!(response.articles.is_empty());
    }
}
