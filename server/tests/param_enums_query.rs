//! Wire contract for the typed REST *query* and *body* parameters.
//!
//! `format` moved from `Option<String>` to `Option<ValueFormat>` so the generated
//! spec regains its `raw | pretty | both` constraint; these pin that the move
//! changed nothing a client can observe. The rest cover enum spellings that an
//! earlier `String` → enum refactor silently dropped (`60m`, `1wk`, `q`, `asc`).

mod common;

use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use finance_query::{Frequency, IndicesRegion, Interval, StatementType, TimeRange, ValueFormat};
use finance_query_server::params::{
    BatchFinancialsQuery, ChartQuery, CustomScreenerRequest, IndicesQuery, MarketSummaryQuery,
    QuoteQuery, QuotesQuery, ScreenersQuery, SectorQuery,
};

use common::{from_query, get_status, post_status, query_route};

// -- format ----------------------------------------------------------------

#[tokio::test]
async fn format_accepts_every_spelling_it_accepted_as_a_string() {
    // `parse_format` went through `ValueFormat::from_str`: case-insensitive,
    // with `fmt`/`full` aliases. All of it must survive the typing.
    for (spelling, want) in [
        ("raw", ValueFormat::Raw),
        ("pretty", ValueFormat::Pretty),
        ("both", ValueFormat::Both),
        ("fmt", ValueFormat::Pretty),
        ("full", ValueFormat::Both),
        ("RAW", ValueFormat::Raw),
        ("Pretty", ValueFormat::Pretty),
        ("BOTH", ValueFormat::Both),
        ("FMT", ValueFormat::Pretty),
        ("Full", ValueFormat::Both),
    ] {
        let q: QuoteQuery = from_query(&format!("format={spelling}"))
            .await
            .unwrap_or_else(|e| panic!("format={spelling}: {e}"));
        assert_eq!(q.format, Some(want), "format={spelling}");
    }
}

#[tokio::test]
async fn an_unrecognized_format_still_falls_back_instead_of_rejecting() {
    // Before typing, an unknown value parsed to `None` and the handler used the
    // default — a 200, not a 400. Turning that into a rejection would break clients.
    let (status, _) = get_status(
        query_route::<QuoteQuery>("/v2/quote/{symbol}"),
        "/v2/quote/AAPL?format=bogus",
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let q: QuoteQuery = from_query("format=bogus")
        .await
        .expect("an unknown format is tolerated");
    assert_eq!(q.format, None, "and resolves to the default");
}

#[tokio::test]
async fn an_omitted_format_is_absent_rather_than_an_error() {
    let q: QuoteQuery = from_query("").await.expect("format is optional");
    assert_eq!(q.format, None);
}

#[tokio::test]
async fn format_is_typed_on_every_struct_that_carries_it() {
    // One case per params struct declaring `format`, so a revert to `String`
    // fails here rather than silently shrinking the generated spec.
    let q: QuoteQuery = from_query("format=pretty").await.expect("QuoteQuery");
    assert_eq!(q.format, Some(ValueFormat::Pretty));
    let q: QuotesQuery = from_query("symbols=AAPL&format=pretty")
        .await
        .expect("QuotesQuery");
    assert_eq!(q.format, Some(ValueFormat::Pretty));
    let q: IndicesQuery = from_query("format=pretty").await.expect("IndicesQuery");
    assert_eq!(q.format, Some(ValueFormat::Pretty));
    let q: MarketSummaryQuery = from_query("format=pretty")
        .await
        .expect("MarketSummaryQuery");
    assert_eq!(q.format, Some(ValueFormat::Pretty));
    let q: ScreenersQuery = from_query("format=pretty").await.expect("ScreenersQuery");
    assert_eq!(q.format, Some(ValueFormat::Pretty));
    let q: SectorQuery = from_query("format=pretty").await.expect("SectorQuery");
    assert_eq!(q.format, Some(ValueFormat::Pretty));
}

// -- interval / range ------------------------------------------------------

#[tokio::test]
async fn sixty_m_is_still_accepted_as_an_hourly_interval() {
    let hour: ChartQuery = from_query("interval=1h").await.expect("interval=1h");
    let sixty: ChartQuery = from_query("interval=60m").await.expect("interval=60m");
    assert_eq!(sixty.interval, Interval::OneHour);
    assert_eq!(sixty.interval, hour.interval, "60m behaves exactly like 1h");
    assert_eq!(sixty.range, hour.range);
}

#[tokio::test]
async fn sixty_m_reaches_the_handler_through_the_real_extractor() {
    let (status, _) = get_status(
        query_route::<ChartQuery>("/v2/chart/{symbol}"),
        "/v2/chart/AAPL?interval=60m",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn one_wk_is_still_accepted_as_a_five_day_range() {
    let wk: ChartQuery = from_query("range=1wk").await.expect("range=1wk");
    assert_eq!(wk.range, TimeRange::FiveDays);
}

#[tokio::test]
async fn an_unknown_interval_is_still_a_400() {
    let (status, _) = get_status(
        query_route::<ChartQuery>("/v2/chart/{symbol}"),
        "/v2/chart/AAPL?interval=7y",
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// -- frequency / statement -------------------------------------------------

#[tokio::test]
async fn frequency_shorthands_the_old_parser_took_are_still_accepted() {
    for (spelling, want) in [
        ("annual", Frequency::Annual),
        ("yearly", Frequency::Annual),
        ("year", Frequency::Annual),
        ("quarterly", Frequency::Quarterly),
        ("quarter", Frequency::Quarterly),
        ("q", Frequency::Quarterly),
    ] {
        let q: BatchFinancialsQuery = from_query(&format!(
            "symbols=AAPL&statement=income&frequency={spelling}"
        ))
        .await
        .unwrap_or_else(|e| panic!("frequency={spelling}: {e}"));
        assert_eq!(q.frequency, want, "frequency={spelling}");
    }
}

#[tokio::test]
async fn the_statement_query_param_takes_the_same_spellings_as_the_path_one() {
    for (spelling, want) in [
        ("income", StatementType::Income),
        ("income-statement", StatementType::Income),
        ("balance", StatementType::Balance),
        ("balance-sheet", StatementType::Balance),
        ("cashflow", StatementType::CashFlow),
        ("cash", StatementType::CashFlow),
        ("cash-flow", StatementType::CashFlow),
    ] {
        let q: BatchFinancialsQuery = from_query(&format!("symbols=AAPL&statement={spelling}"))
            .await
            .unwrap_or_else(|e| panic!("statement={spelling}: {e}"));
        assert_eq!(q.statement, want, "statement={spelling}");
    }
}

// -- indices region --------------------------------------------------------

#[tokio::test]
async fn indices_region_shorthands_the_old_parser_took_are_still_accepted() {
    for (spelling, want) in [
        ("americas", IndicesRegion::Americas),
        ("america", IndicesRegion::Americas),
        ("am", IndicesRegion::Americas),
        ("europe", IndicesRegion::Europe),
        ("eu", IndicesRegion::Europe),
        ("asia-pacific", IndicesRegion::AsiaPacific),
        ("asia_pacific", IndicesRegion::AsiaPacific),
        ("asia", IndicesRegion::AsiaPacific),
        ("apac", IndicesRegion::AsiaPacific),
        ("middle-east-africa", IndicesRegion::MiddleEastAfrica),
        ("mea", IndicesRegion::MiddleEastAfrica),
        ("emea", IndicesRegion::MiddleEastAfrica),
        ("currencies", IndicesRegion::Currencies),
        ("currency", IndicesRegion::Currencies),
        ("fx", IndicesRegion::Currencies),
    ] {
        let q: IndicesQuery = from_query(&format!("region={spelling}"))
            .await
            .unwrap_or_else(|e| panic!("region={spelling}: {e}"));
        assert_eq!(q.region, Some(want), "region={spelling}");
    }
}

// -- custom screener body --------------------------------------------------

fn custom_screener_route() -> Router {
    Router::new().route(
        "/v2/screeners/custom",
        post(|Json(b): Json<CustomScreenerRequest>| async move {
            format!("{:?}/{:?}/{:?}", b.sort_type, b.quote_type, b.format)
        }),
    )
}

#[tokio::test]
async fn the_custom_screener_body_takes_the_spellings_it_always_did() {
    // The pre-typed handler lowercased `sortType` itself and handed `quoteType`
    // to the GraphQL layer, which parsed it with `FromStr`.
    for (body, want) in [
        (
            r#"{"sortType":"DESC","quoteType":"EQUITY","format":"raw"}"#,
            "Some(Desc)/Some(Equity)/Some(Raw)",
        ),
        (
            r#"{"sortType":"asc","quoteType":"equity","format":"fmt"}"#,
            "Some(Asc)/Some(Equity)/Some(Pretty)",
        ),
        (
            r#"{"sortType":"ascending","quoteType":"fund","format":"full"}"#,
            "Some(Asc)/Some(MutualFund)/Some(Both)",
        ),
        (
            r#"{"sortType":"desc","quoteType":"mutualfund"}"#,
            "Some(Desc)/Some(MutualFund)/None",
        ),
        (
            r#"{"sortType":"descending","quoteType":"stocks"}"#,
            "Some(Desc)/Some(Equity)/None",
        ),
    ] {
        let (status, got) =
            post_status(custom_screener_route(), "/v2/screeners/custom", body).await;
        assert_eq!(status, StatusCode::OK, "body {body}");
        assert_eq!(got, want, "body {body}");
    }
}
