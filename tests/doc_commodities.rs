//! Compile tests for docs/library/commodities.md
//!
//! Requires the `fmp` feature flag:
//!   cargo test --test doc_commodities --features fmp
//!
//! Runtime behavior of the commodity quote path is covered without network
//! access by the mock + unit tests in `src/adapters/fmp/commodities/mod.rs` and
//! `src/adapters/alphavantage/commodities/mod.rs` (mocked HTTP → DTO →
//! canonical `CommodityQuote`).

#![cfg(feature = "fmp")]

use finance_query::CommodityQuote;

// ---------------------------------------------------------------------------
// CommodityQuote — compile-time field verification
// ---------------------------------------------------------------------------

/// Verifies all CommodityQuote fields documented in commodities.md exist with
/// the correct types. Never called; exists only for the compiler to type-check.
#[allow(dead_code)]
fn _verify_commodity_quote_fields(q: CommodityQuote) {
    let _: String = q.symbol;
    let _: Option<String> = q.name;
    let _: Option<String> = q.unit;
    let _: Option<f64> = q.price;
    let _: Option<f64> = q.change;
    let _: Option<f64> = q.change_percent;
    let _: Option<i64> = q.timestamp;
}

// ---------------------------------------------------------------------------
// Compile-time: handle API — mirrors the main handle block from commodities.md
// ---------------------------------------------------------------------------

/// Verifies the documented `Commodity` flow type-checks:
///   providers.commodity(symbol) → quote(), chart(), history()
/// Never called; exists only for the compiler to type-check. Runtime behavior
/// is covered by the mock + unit tests in `src/adapters/fmp/commodities/mod.rs`.
#[allow(dead_code)]
async fn _verify_commodity_api() -> finance_query::Result<()> {
    use finance_query::{Capability, Interval, Provider, Providers, TimeRange};

    let providers = Providers::builder()
        .route(Capability::COMMODITIES, [Provider::Fmp])
        .build()
        .await?;

    let gold = providers.commodity("GCUSD");
    let _quote: CommodityQuote = gold.quote().await?;
    let _chart: finance_query::Chart = gold.chart(Interval::OneDay, TimeRange::OneMonth).await?;
    let _history: finance_query::Chart = gold.history(TimeRange::OneMonth).await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Compile-time: Indicators & Risk section of commodities.md
// ---------------------------------------------------------------------------

/// Verifies the documented `Commodity::indicators`/`indicator`/`risk` flow
/// type-checks. Never called; exists only for the compiler to type-check.
#[allow(dead_code)]
#[cfg(all(feature = "indicators", feature = "risk"))]
async fn _verify_commodity_indicators_and_risk() -> finance_query::Result<()> {
    use finance_query::indicators::Indicator;
    use finance_query::{Capability, Interval, Provider, Providers, TimeRange};

    let providers = Providers::builder()
        .route(Capability::COMMODITIES, [Provider::Fmp])
        .build()
        .await?;
    let gold = providers.commodity("GCUSD");

    let summary = gold
        .indicators(Interval::OneDay, TimeRange::ThreeMonths)
        .await?;
    let _: Option<f64> = summary.rsi_14;

    let _rsi_21 = gold
        .indicator(Indicator::Rsi(21), Interval::OneDay, TimeRange::ThreeMonths)
        .await?;

    let risk = gold.risk(Interval::OneDay, TimeRange::OneYear).await?;
    let _: f64 = risk.var_95;
    let _: f64 = risk.max_drawdown;

    Ok(())
}
