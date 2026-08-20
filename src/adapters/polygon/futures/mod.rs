//! Futures market data endpoints.

// Polygon publishes no grouped-daily (all-tickers-for-one-date) endpoint
// for futures, unlike stocks/crypto/forex; the URL 404s. Per-symbol
// aggregates already route through the generic CHART capability path
// (`fetch_chart_response`), so this module stays unrouted.
#[allow(dead_code)]
pub mod aggregates;
pub mod reference;
pub mod snapshots;
#[allow(dead_code)] // unrouted: tick-level trades land with #250
pub mod trades;
