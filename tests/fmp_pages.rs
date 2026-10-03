#![cfg(feature = "fmp")]
use finance_query::{Capability, Provider, Providers};
use mockito::{Matcher, Server};

async fn client(server: &Server) -> Providers {
    Providers::builder()
        .providers([Provider::Fmp])
        .api_key(Provider::Fmp, "page-fixture")
        .endpoint(Provider::Fmp, server.url())
        .route(Capability::DISCOVERY, [Provider::Fmp])
        .requests_per_minute(Provider::Fmp, 600000)
        .build()
        .await
        .unwrap()
}

#[tokio::test]
async fn delisted_page_resumes_and_rejects_mismatched_or_repeated_continuations() {
    let mut server = Server::new_async().await;
    let first = server
        .mock("GET", "/stable/delisted-companies")
        .match_query(Matcher::UrlEncoded("page".into(), "0".into()))
        .with_body(r#"[{"symbol":"OLD","ipoDate":"1980-12-12","delistedDate":"2003-01-02"}]"#)
        .expect(1)
        .create_async()
        .await;
    let second = server
        .mock("GET", "/stable/delisted-companies")
        .match_query(Matcher::UrlEncoded("page".into(), "1".into()))
        .with_body("[]")
        .expect(1)
        .create_async()
        .await;
    let page = client(&server)
        .await
        .discovery()
        .delisted_stocks_page(100, None)
        .await
        .unwrap();
    assert_eq!(page.items[0].ipo_date.as_deref(), Some("1980-12-12"));
    let cursor = serde_json::to_string(&page.next.unwrap()).unwrap();
    assert!(!cursor.contains("fixture"));
    let cursor = serde_json::from_str(&cursor).unwrap();
    let rebuilt = client(&server).await;
    assert!(
        rebuilt
            .discovery()
            .delisted_stocks_page(99, Some(&cursor))
            .await
            .is_err()
    );
    let end = rebuilt
        .discovery()
        .delisted_stocks_page(100, Some(&cursor))
        .await
        .unwrap();
    assert!(end.items.is_empty() && end.next.is_none());
    first.assert_async().await;
    second.assert_async().await;
}

#[tokio::test]
async fn numbered_delisted_pages_are_independent_and_validate_bounds() {
    let mut server = Server::new_async().await;
    let second = server
        .mock("GET", "/stable/delisted-companies")
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("page".into(), "2".into()),
            Matcher::UrlEncoded("limit".into(), "100".into()),
        ]))
        .with_body(r#"[{"symbol":"OLD","ipoDate":"1980-12-12","delistedDate":"2003-01-02"}]"#)
        .expect(1)
        .create_async()
        .await;
    let first = server
        .mock("GET", "/stable/delisted-companies")
        .match_query(Matcher::UrlEncoded("page".into(), "0".into()))
        .with_body("[]")
        .expect(1)
        .create_async()
        .await;
    let client = client(&server).await;
    let discovery = client.discovery();
    let (later, earlier) = tokio::join!(
        discovery.delisted_stocks_page_at(2, 100),
        discovery.delisted_stocks_page_at(0, 100)
    );
    let later = later.unwrap();
    assert_eq!(later[0].symbol, "OLD");
    assert_eq!(later[0].ipo_date.as_deref(), Some("1980-12-12"));
    assert_eq!(later[0].delisted_date.as_deref(), Some("2003-01-02"));
    assert!(earlier.unwrap().is_empty());
    for (page, limit) in [(10000, 100), (0, 0), (0, 101)] {
        assert!(
            discovery
                .delisted_stocks_page_at(page, limit)
                .await
                .is_err()
        );
    }
    first.assert_async().await;
    second.assert_async().await;
}
