//! Index market data endpoints.

// #245 investigated adding grouped-daily (all-tickers-for-one-date) aggregates
// here to match stocks/crypto/forex, but Polygon has no such endpoint for
// indices (no `/v2/aggs/grouped/locale/.../market/indices/{date}` — verified
// against the current API docs, 404). Per-symbol aggregates already route
// through the generic CHART capability path (`fetch_chart_response`), so this
// module stays unrouted.
#[allow(dead_code)]
pub mod aggregates;
pub mod snapshots;
