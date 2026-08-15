//! Compile-time API-surface check for the market-wide factory handles
//! documented in docs/library/providers/index.md ("Three handles are
//! market-wide..."). Ported from the retired `tests/doc_providers.rs`; it
//! asserts nothing at runtime — only that the `Providers` factories and the
//! `Discovery` / `MarketCalendar` / `Market` handle types still exist with
//! the documented signatures, so a rename breaks this build, not the docs.

/// `Discovery` and `MarketCalendar` need a keyed-provider feature (see the
/// re-export gates in `src/lib.rs`); the keyed set here satisfies both.
#[cfg(any(feature = "fmp", feature = "polygon", feature = "alphavantage"))]
#[allow(dead_code)]
fn _verify_market_wide_handles(providers: &finance_query::Providers) {
    let _disco: finance_query::Discovery = providers.discovery();
    let _cal: finance_query::MarketCalendar = providers.calendar();
    let _mkt: finance_query::Market = providers.market();
}

/// `Market` is unconditional — verify it without any keyed-provider feature.
#[allow(dead_code)]
fn _verify_market_handle(providers: &finance_query::Providers) {
    let _mkt: finance_query::Market = providers.market();
}
