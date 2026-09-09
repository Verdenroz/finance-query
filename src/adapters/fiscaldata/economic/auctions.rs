//! US Treasury securities auctions, on the `ECONOMIC` capability.
//!
//! Two datasets share the FiscalData query grammar: `auctions_query` holds
//! every auction from announcement through settlement, and
//! `upcoming_auctions` holds the forward schedule.

use crate::error::Result;
use crate::models::economic::{TreasuryAuction, TreasuryAuctionQuery, UpcomingAuction};

use super::super::client::{DATE_FIELD, RowQuery};
use super::super::models::FiscalRow;
use super::parse_value;

const AUCTIONS_DATASET: &str = "v1/accounting/od/auctions_query";
const UPCOMING_DATASET: &str = "v1/accounting/od/upcoming_auctions";

/// The curated slice of `auctions_query`'s ~110 columns: identity, terms, and
/// the bidder breakdown, dropping the settlement plumbing (CUSIP conversion
/// factors, PDF filenames, STRIPS minimums).
const AUCTION_FIELDS: &str = "record_date,cusip,security_type,security_term,auction_date,\
    issue_date,maturity_date,reopening,auction_format,int_rate,offering_amt,total_tendered,\
    total_accepted,bid_to_cover_ratio,high_yield,high_discnt_rate,high_investment_rate,\
    high_price,primary_dealer_accepted,direct_bidder_accepted,indirect_bidder_accepted,\
    comp_accepted,noncomp_accepted,soma_accepted";

/// `announcemt_date` is Treasury's own spelling of the column.
const UPCOMING_FIELDS: &str = "record_date,security_type,security_term,reopening,cusip,\
    offering_amt,announcemt_date,auction_date,issue_date";

/// Auctions returned when the caller names no limit.
const DEFAULT_LIMIT: u32 = 100;

/// Ceiling on `limit`, which maps straight onto FiscalData's page size.
const MAX_LIMIT: u32 = 1_000;

/// The whole forward schedule is under a hundred rows, so one page covers it.
const UPCOMING_PAGE_SIZE: u32 = 1_000;

/// Compose the FiscalData row filter for `query`.
fn build_filter(query: &TreasuryAuctionQuery) -> Option<String> {
    let mut clauses: Vec<String> = Vec::new();
    if let Some(security_type) = &query.security_type {
        clauses.push(format!("security_type:eq:{security_type}"));
    }
    if let Some(security_term) = &query.security_term {
        clauses.push(format!("security_term:eq:{security_term}"));
    }
    if let Some(from) = &query.from {
        clauses.push(format!("auction_date:gte:{from}"));
    }
    if let Some(to) = &query.to {
        clauses.push(format!("auction_date:lte:{to}"));
    }
    (!clauses.is_empty()).then(|| clauses.join(","))
}

/// Read a column that carries text, treating FiscalData's `"null"` sentinel as
/// absent.
fn text(row: &FiscalRow, field: &str) -> Option<String> {
    let raw = row.get(field)?.trim();
    (!raw.is_empty() && !raw.eq_ignore_ascii_case("null")).then(|| raw.to_string())
}

fn number(row: &FiscalRow, field: &str) -> Option<f64> {
    parse_value(row.get(field)?)
}

/// Read one of the `"Yes"`/`"No"` columns.
fn flag(row: &FiscalRow, field: &str) -> Option<bool> {
    match text(row, field)?.as_str() {
        "Yes" => Some(true),
        "No" => Some(false),
        _ => None,
    }
}

fn to_auction(row: &FiscalRow) -> Option<TreasuryAuction> {
    Some(TreasuryAuction {
        record_date: text(row, DATE_FIELD)?,
        cusip: text(row, "cusip")?,
        security_type: text(row, "security_type")?,
        security_term: text(row, "security_term")?,
        auction_date: text(row, "auction_date")?,
        issue_date: text(row, "issue_date")?,
        maturity_date: text(row, "maturity_date")?,
        reopening: flag(row, "reopening"),
        auction_format: text(row, "auction_format"),
        int_rate: number(row, "int_rate"),
        offering_amt: number(row, "offering_amt"),
        total_tendered: number(row, "total_tendered"),
        total_accepted: number(row, "total_accepted"),
        bid_to_cover_ratio: number(row, "bid_to_cover_ratio"),
        high_yield: number(row, "high_yield"),
        high_discnt_rate: number(row, "high_discnt_rate"),
        high_investment_rate: number(row, "high_investment_rate"),
        high_price: number(row, "high_price"),
        primary_dealer_accepted: number(row, "primary_dealer_accepted"),
        direct_bidder_accepted: number(row, "direct_bidder_accepted"),
        indirect_bidder_accepted: number(row, "indirect_bidder_accepted"),
        comp_accepted: number(row, "comp_accepted"),
        noncomp_accepted: number(row, "noncomp_accepted"),
        soma_accepted: number(row, "soma_accepted"),
    })
}

fn to_upcoming(row: &FiscalRow) -> Option<UpcomingAuction> {
    Some(UpcomingAuction {
        record_date: text(row, DATE_FIELD)?,
        security_type: text(row, "security_type")?,
        security_term: text(row, "security_term")?,
        cusip: text(row, "cusip")?,
        reopening: flag(row, "reopening"),
        offering_amt: number(row, "offering_amt"),
        announcement_date: text(row, "announcemt_date")?,
        auction_date: text(row, "auction_date")?,
        issue_date: text(row, "issue_date")?,
    })
}

/// Fetch auctions matching `query`, most recent auction date first.
pub(crate) async fn fetch_treasury_auctions_response(
    query: &TreasuryAuctionQuery,
) -> Result<Vec<TreasuryAuction>> {
    let limit = query.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let filter = build_filter(query);
    let (rows, _) = super::super::client()?
        .rows(
            &RowQuery {
                dataset: AUCTIONS_DATASET,
                fields: AUCTION_FIELDS,
                sort: "-auction_date",
                filter: filter.as_deref(),
                page_size: limit,
            },
            1,
        )
        .await?;

    Ok(rows.iter().filter_map(to_auction).collect())
}

/// Fetch the auctions Treasury has scheduled but not yet held, soonest first.
pub(crate) async fn fetch_upcoming_auctions_response() -> Result<Vec<UpcomingAuction>> {
    let (rows, _) = super::super::client()?
        .rows(
            &RowQuery {
                dataset: UPCOMING_DATASET,
                fields: UPCOMING_FIELDS,
                sort: "-record_date",
                filter: None,
                page_size: UPCOMING_PAGE_SIZE,
            },
            1,
        )
        .await?;

    Ok(newest_schedule(&rows))
}

/// Keep the newest published schedule out of `rows`, soonest auction first.
///
/// The dataset appends each week's schedule without retiring the last, so rows
/// below the newest record_date describe auctions already held. Expects `rows`
/// ordered by descending record_date.
fn newest_schedule(rows: &[FiscalRow]) -> Vec<UpcomingAuction> {
    let Some(newest) = rows.first().and_then(|row| text(row, DATE_FIELD)) else {
        return Vec::new();
    };
    let mut upcoming: Vec<UpcomingAuction> = rows
        .iter()
        .take_while(|row| text(row, DATE_FIELD).as_deref() == Some(newest.as_str()))
        .filter_map(to_upcoming)
        .collect();
    upcoming.sort_by(|a, b| a.auction_date.cmp(&b.auction_date));
    upcoming
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::FinanceError;
    use crate::rate_limiter::RateLimiter;
    use std::sync::Arc;
    use std::time::Duration;

    use super::super::super::client::FiscalDataClient;

    fn test_client(base_url: &str) -> FiscalDataClient {
        FiscalDataClient::new(
            Duration::from_secs(5),
            Arc::new(RateLimiter::new(100.0)),
            base_url,
        )
        .unwrap()
    }

    fn row(pairs: &[(&str, &str)]) -> FiscalRow {
        pairs
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect()
    }

    fn bill_row() -> FiscalRow {
        row(&[
            ("record_date", "2026-09-01"),
            ("cusip", "912797UK1"),
            ("security_type", "Bill"),
            ("security_term", "6-Week"),
            ("auction_date", "2026-09-01"),
            ("issue_date", "2026-09-03"),
            ("maturity_date", "2026-10-15"),
            ("reopening", "Yes"),
            ("auction_format", "Single-Price"),
            ("int_rate", "null"),
            ("offering_amt", "85000000000"),
            ("total_tendered", "247263363500"),
            ("total_accepted", "90124268500"),
            ("bid_to_cover_ratio", "2.850000"),
            ("high_yield", "null"),
            ("high_discnt_rate", "3.735000"),
            ("high_investment_rate", "3.803000"),
            ("high_price", "99.564250"),
            ("soma_accepted", "5124268500"),
        ])
    }

    fn schedule_row(record_date: &str, cusip: &str, auction_date: &str) -> FiscalRow {
        row(&[
            ("record_date", record_date),
            ("security_type", "Bill"),
            ("security_term", "13-Week"),
            ("cusip", cusip),
            ("reopening", "No"),
            ("offering_amt", "null"),
            ("announcemt_date", "2026-09-10"),
            ("auction_date", auction_date),
            ("issue_date", "2026-09-17"),
        ])
    }

    #[test]
    fn empty_query_sends_no_filter() {
        assert_eq!(build_filter(&TreasuryAuctionQuery::new()), None);
    }

    #[test]
    fn query_fields_become_filter_clauses() {
        let query = TreasuryAuctionQuery::new()
            .security_type("Bill")
            .security_term("13-Week")
            .dates(Some("2026-01-01"), Some("2026-06-30"));
        assert_eq!(
            build_filter(&query).as_deref(),
            Some(
                "security_type:eq:Bill,security_term:eq:13-Week,\
                 auction_date:gte:2026-01-01,auction_date:lte:2026-06-30"
            )
        );
    }

    #[test]
    fn bill_maps_discount_rate_and_leaves_yield_unset() {
        let auction = to_auction(&bill_row()).unwrap();
        assert_eq!(auction.cusip, "912797UK1");
        assert_eq!(auction.security_type, "Bill");
        assert_eq!(auction.maturity_date, "2026-10-15");
        assert_eq!(auction.reopening, Some(true));
        assert_eq!(auction.bid_to_cover_ratio, Some(2.85));
        assert_eq!(auction.high_discnt_rate, Some(3.735));
        assert_eq!(auction.high_investment_rate, Some(3.803));
        assert_eq!(auction.high_yield, None);
        assert_eq!(auction.int_rate, None);
        // Columns absent from the response, not merely null.
        assert_eq!(auction.comp_accepted, None);
    }

    #[test]
    fn row_missing_an_identity_column_is_dropped() {
        let mut incomplete = bill_row();
        incomplete.remove("cusip");
        assert!(to_auction(&incomplete).is_none());
    }

    #[test]
    fn schedule_keeps_only_the_newest_record_date() {
        let rows = vec![
            schedule_row("2026-09-04", "912797VH7", "2026-09-14"),
            schedule_row("2026-09-04", "912797VL8", "2026-09-10"),
            schedule_row("2024-03-08", "912797JW8", "2024-03-14"),
        ];
        let schedule = newest_schedule(&rows);
        assert_eq!(schedule.len(), 2);
        assert_eq!(
            schedule
                .iter()
                .map(|a| a.auction_date.as_str())
                .collect::<Vec<_>>(),
            ["2026-09-10", "2026-09-14"]
        );
        assert_eq!(schedule[0].announcement_date, "2026-09-10");
        assert_eq!(schedule[0].reopening, Some(false));
        assert_eq!(schedule[0].offering_amt, None);
    }

    #[test]
    fn schedule_of_no_rows_is_empty() {
        assert!(newest_schedule(&[]).is_empty());
    }

    #[tokio::test]
    async fn limit_and_filter_reach_the_wire() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/v1/accounting/od/auctions_query")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("page[size]".into(), "25".into()),
                mockito::Matcher::UrlEncoded("sort".into(), "-auction_date".into()),
                mockito::Matcher::UrlEncoded(
                    "filter".into(),
                    "security_type:eq:Bill,auction_date:gte:2026-01-01".into(),
                ),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({ "data": [bill_row()], "meta": { "total-pages": 1 } })
                    .to_string(),
            )
            .create_async()
            .await;

        let query = TreasuryAuctionQuery::new()
            .security_type("Bill")
            .dates(Some("2026-01-01"), None)
            .limit(25);
        let (rows, _) = test_client(&server.url())
            .rows(
                &RowQuery {
                    dataset: AUCTIONS_DATASET,
                    fields: AUCTION_FIELDS,
                    sort: "-auction_date",
                    filter: build_filter(&query).as_deref(),
                    page_size: query.limit.unwrap(),
                },
                1,
            )
            .await
            .unwrap();
        assert_eq!(rows.len(), 1);
    }

    #[tokio::test]
    async fn paging_stops_at_the_page_cap() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/v1/accounting/od/auctions_query")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({ "data": [bill_row()], "meta": { "total-pages": 9 } })
                    .to_string(),
            )
            .expect(1)
            .create_async()
            .await;

        let (rows, _) = test_client(&server.url())
            .rows(
                &RowQuery {
                    dataset: AUCTIONS_DATASET,
                    fields: AUCTION_FIELDS,
                    sort: "-auction_date",
                    filter: None,
                    page_size: 100,
                },
                1,
            )
            .await
            .unwrap();
        assert_eq!(rows.len(), 1);
    }

    #[tokio::test]
    async fn no_matching_auctions_is_an_empty_list() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/v1/accounting/od/auctions_query")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(serde_json::json!({ "data": [], "meta": { "total-pages": 0 } }).to_string())
            .create_async()
            .await;

        let (rows, _) = test_client(&server.url())
            .rows(
                &RowQuery {
                    dataset: AUCTIONS_DATASET,
                    fields: AUCTION_FIELDS,
                    sort: "-auction_date",
                    filter: None,
                    page_size: 100,
                },
                1,
            )
            .await
            .unwrap();
        assert!(rows.iter().filter_map(to_auction).next().is_none());
    }

    #[tokio::test]
    async fn rejected_query_surfaces_the_api_message() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/v1/accounting/od/auctions_query")
            .match_query(mockito::Matcher::Any)
            .with_status(400)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "error": "Invalid Query Param",
                    "message": "Field 'nope' does not exist."
                })
                .to_string(),
            )
            .create_async()
            .await;

        let err = test_client(&server.url())
            .rows(
                &RowQuery {
                    dataset: AUCTIONS_DATASET,
                    fields: "nope",
                    sort: "-auction_date",
                    filter: None,
                    page_size: 100,
                },
                1,
            )
            .await
            .unwrap_err();
        match err {
            FinanceError::MacroDataError { provider, context } => {
                assert_eq!(provider, "US Treasury FiscalData");
                assert!(context.contains("does not exist"), "got {context}");
            }
            other => panic!("expected MacroDataError, got {other:?}"),
        }
    }
}
