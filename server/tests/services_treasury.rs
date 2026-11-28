use finance_query::{Capability, Provider, Providers, TreasuryAuctionQuery};
use finance_query_server::cache::Cache;
use finance_query_server::services::treasury;
use mockito::{Matcher, Server};
use serde_json::json;
use std::sync::Arc;

const AUCTIONS: &str = "/services/api/fiscal_service/v1/accounting/od/auctions_query";
const UPCOMING: &str = "/services/api/fiscal_service/v1/accounting/od/upcoming_auctions";

// One test: the FiscalData endpoint override is process-global, so cases share a single server.
#[tokio::test]
async fn treasury_services_shape_fiscaldata_responses_and_map_errors() {
    let mut server = Server::new_async().await;
    let auctions = server
        .mock("GET", AUCTIONS)
        .match_query(Matcher::Any)
        .with_body(
            json!({
                "data": [{
                    "record_date": "2026-09-02", "cusip": "912797UK1",
                    "security_type": "Bill", "security_term": "13-Week",
                    "auction_date": "2026-09-01", "issue_date": "2026-09-04",
                    "maturity_date": "2026-12-03", "reopening": "No",
                    "int_rate": "null", "high_yield": "4.125",
                    "bid_to_cover_ratio": "2.9", "offering_amt": "70000000000"
                }],
                "meta": {"total-pages": 1}
            })
            .to_string(),
        )
        .create_async()
        .await;
    let upcoming = server
        .mock("GET", UPCOMING)
        .match_query(Matcher::Any)
        .with_body(
            json!({
                "data": [
                    {"record_date": "2026-09-05", "security_type": "Note",
                     "security_term": "10-Year", "reopening": "Yes", "cusip": "91282CNN7",
                     "offering_amt": "39000000000", "announcemt_date": "2026-09-08",
                     "auction_date": "2026-09-10", "issue_date": "2026-09-15"},
                    {"record_date": "2026-09-05", "security_type": "Bill",
                     "security_term": "4-Week", "reopening": "No", "cusip": "912797UL9",
                     "offering_amt": "80000000000", "announcemt_date": "2026-09-02",
                     "auction_date": "2026-09-04", "issue_date": "2026-09-09"}
                ],
                "meta": {"total-pages": 1}
            })
            .to_string(),
        )
        .create_async()
        .await;

    let providers = Arc::new(
        Providers::builder()
            .providers([Provider::FiscalData])
            .endpoint(Provider::FiscalData, server.url())
            .route(Capability::ECONOMIC, [Provider::FiscalData])
            .build()
            .await
            .unwrap(),
    );
    let cache = Cache::new(None).await;

    let value = treasury::get_treasury_auctions(&cache, &providers, TreasuryAuctionQuery::new())
        .await
        .unwrap();
    let rows = value.as_array().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["cusip"], "912797UK1");
    assert_eq!(rows[0]["high_yield"], 4.125);
    assert!(rows[0]["int_rate"].is_null());
    auctions.assert_async().await;

    let value = treasury::get_upcoming_auctions(&cache, &providers)
        .await
        .unwrap();
    let rows = value.as_array().unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(
        rows[0]["security_term"], "4-Week",
        "soonest auction comes first"
    );
    upcoming.assert_async().await;

    server.reset();
    let failing = server
        .mock("GET", AUCTIONS)
        .match_query(Matcher::Any)
        .with_status(500)
        .create_async()
        .await;
    let query = TreasuryAuctionQuery::new().security_type("Note");
    let err = treasury::get_treasury_auctions(&cache, &providers, query)
        .await
        .unwrap_err();
    assert!(!err.to_string().is_empty());
    failing.assert_async().await;
}
