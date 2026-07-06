//! Compile tests for docs/library/forex.md
//!
//! Run: cargo test --test doc_forex --features alphavantage
//!
//! Runtime behavior of the forex quote path is covered without network access
//! by the mock + unit tests in `src/adapters/alphavantage/forex/mod.rs` and
//! `src/adapters/fmp/forex/mod.rs` (mocked HTTP → DTO → canonical `ForexQuote`),
//! and the chart symbol mapping in `src/domains/forex.rs`.
#![cfg(feature = "alphavantage")]

// ---------------------------------------------------------------------------
// Compile-time: ForexQuote field verification
// ---------------------------------------------------------------------------

/// Verifies all ForexQuote fields documented in forex.md exist with correct types.
/// Never called; exists only for the compiler to type-check.
#[allow(dead_code)]
fn _verify_forex_quote_fields(q: finance_query::ForexQuote) {
    let _: String = q.symbol;
    let _: Option<String> = q.base_currency;
    let _: Option<String> = q.quote_currency;
    let _: Option<f64> = q.bid;
    let _: Option<f64> = q.ask;
    let _: Option<f64> = q.price;
    let _: Option<f64> = q.change;
    let _: Option<f64> = q.change_percent;
    let _: Option<i64> = q.timestamp;
}

// ---------------------------------------------------------------------------
// Compile-time: handle API used by the forex.md examples
// ---------------------------------------------------------------------------

/// Verifies the documented `ForexPair` flow type-checks: routing setup,
/// `quote()`, `chart()`, `history()`, and `.cache()`.
/// Never called; exists only for the compiler to type-check.
#[allow(dead_code)]
async fn _verify_forex_pair_api() -> finance_query::Result<()> {
    use finance_query::{Capability, Interval, Provider, Providers, TimeRange};
    use std::time::Duration;

    let providers = Providers::builder()
        .route(Capability::FOREX, [Provider::AlphaVantage])
        .build()
        .await?;

    let pair = providers.forex("EUR", "USD");
    let _quote: finance_query::ForexQuote = pair.quote().await?;
    let _chart: finance_query::Chart = pair.chart(Interval::OneDay, TimeRange::OneMonth).await?;
    let _history: finance_query::Chart = pair.history(TimeRange::OneMonth).await?;

    let _cached = providers.forex("EUR", "USD").cache(Duration::from_secs(60));
    Ok(())
}

// ---------------------------------------------------------------------------
// Compile-time: Indicators & Risk section of forex.md
// ---------------------------------------------------------------------------

/// Verifies the documented `ForexPair::indicators`/`indicator`/`risk` flow
/// type-checks. Never called; exists only for the compiler to type-check.
#[allow(dead_code)]
#[cfg(all(feature = "indicators", feature = "risk"))]
async fn _verify_forex_pair_indicators_and_risk() -> finance_query::Result<()> {
    use finance_query::indicators::Indicator;
    use finance_query::{Capability, Interval, Provider, Providers, TimeRange};

    let providers = Providers::builder()
        .route(Capability::FOREX, [Provider::AlphaVantage])
        .build()
        .await?;
    let pair = providers.forex("EUR", "USD");

    let summary = pair
        .indicators(Interval::OneDay, TimeRange::ThreeMonths)
        .await?;
    let _: Option<f64> = summary.rsi_14;

    let _rsi_21 = pair
        .indicator(Indicator::Rsi(21), Interval::OneDay, TimeRange::ThreeMonths)
        .await?;

    let risk = pair.risk(Interval::OneDay, TimeRange::OneYear).await?;
    let _: f64 = risk.var_95;
    let _: f64 = risk.max_drawdown;

    Ok(())
}
