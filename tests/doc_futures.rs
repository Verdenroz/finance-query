//! Compile tests for docs/library/futures.md
//!
//! Run: cargo test --test doc_futures --features polygon
//!
//! Runtime behavior of the futures quote path is covered without network
//! access by the mock + unit tests in `src/adapters/polygon/futures/snapshots.rs`
//! (mocked HTTP → DTO → canonical `FuturesQuote`).

#![cfg(feature = "polygon")]

use finance_query::FuturesQuote;

// ---------------------------------------------------------------------------
// FuturesQuote — compile-time field verification
// ---------------------------------------------------------------------------

/// Verifies all FuturesQuote fields documented in futures.md exist with correct
/// types. Never called; exists only for the compiler to type-check.
#[allow(dead_code)]
fn _verify_futures_quote_fields(q: FuturesQuote) {
    let _: String = q.symbol;
    let _: Option<String> = q.name;
    let _: Option<String> = q.underlying;
    let _: Option<String> = q.exchange;
    let _: Option<String> = q.expiration_date;
    let _: Option<f64> = q.price;
    let _: Option<f64> = q.change;
    let _: Option<f64> = q.change_percent;
    let _: Option<u64> = q.open_interest;
    let _: Option<u64> = q.volume;
    let _: Option<i64> = q.timestamp;
}

// ---------------------------------------------------------------------------
// Compile-time: handle API — mirrors the "Getting a Handle" block in futures.md
// ---------------------------------------------------------------------------

/// Verifies the documented `FuturesContract` flow type-checks: routing setup,
/// `quote()`, `chart()`, `history()`.
/// Never called; exists only for the compiler to type-check. Runtime behavior
/// is covered by the mock + unit tests in
/// `src/adapters/polygon/futures/snapshots.rs`.
#[allow(dead_code)]
async fn _verify_futures_contract_api() -> finance_query::Result<()> {
    use finance_query::{Capability, Interval, Provider, Providers, TimeRange};

    let providers = Providers::builder()
        .route(Capability::FUTURES, &[Provider::Polygon])
        .build()
        .await?;

    let contract = providers.futures("ES");
    let _quote: FuturesQuote = contract.quote().await?;
    let _chart: finance_query::Chart = contract
        .chart(Interval::OneDay, TimeRange::OneMonth)
        .await?;
    let _history: finance_query::Chart = contract.history(TimeRange::OneMonth).await?;
    Ok(())
}
