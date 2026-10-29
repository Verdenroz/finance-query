#![cfg(feature = "fmp")]
use finance_query::{Capability, Provider, Providers};
use mockito::{Matcher, Server};

#[tokio::test]
async fn a_different_page_containing_only_seen_records_is_not_progress() {
    let mut server = Server::new_async().await;
    let first = server
        .mock("GET", "/stable/delisted-companies")
        .match_query(Matcher::UrlEncoded("page".into(), "0".into()))
        .with_body(r#"[{"symbol":"A"},{"symbol":"B"}]"#)
        .create_async()
        .await;
    let repeated = server
        .mock("GET", "/stable/delisted-companies")
        .match_query(Matcher::UrlEncoded("page".into(), "1".into()))
        .with_body(r#"[{"symbol":"B"}]"#)
        .create_async()
        .await;
    let client = Providers::builder()
        .providers([Provider::Fmp])
        .api_key(Provider::Fmp, "fixture-progress")
        .endpoint(Provider::Fmp, server.url())
        .requests_per_minute(Provider::Fmp, 600000)
        .route(Capability::DISCOVERY, [Provider::Fmp])
        .build()
        .await
        .unwrap();
    let page = client
        .discovery()
        .delisted_stocks_page(100, None)
        .await
        .unwrap();
    assert!(
        client
            .discovery()
            .delisted_stocks_page(100, page.next.as_ref())
            .await
            .is_err()
    );
    first.assert_async().await;
    repeated.assert_async().await;
}
