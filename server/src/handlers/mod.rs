//! REST route handlers, one module per domain — mirrors the domain split
//! already used by `finance_query_server::services` and by
//! `finance-query-mcp/src/tools/`. `support`/`gql_bridge` hold helpers shared
//! across 2+ domain modules; anything used by only one domain lives there
//! instead.

mod analysis;
mod calendar;
mod chart;
mod crypto;
mod edgar;
mod events;
mod feeds;
mod feeds_stream;
mod financials;
mod fred;
mod gql_bridge;
mod holders;
mod indicators;
mod market;
mod metadata;
mod news;
mod options;
mod quote;
mod risk;
mod screener;
mod search;
mod sector;
mod stream;
mod support;
mod system;
mod transcripts;

pub(crate) use system::metrics_middleware;

use axum::{
    Router,
    routing::{get, post},
};

/// API routes
pub(crate) fn api_routes() -> Router {
    Router::new()
        // Routes are sorted alphabetically by path.
        // GET /v2/analysis/{symbol}/{analysis_type}
        .route(
            "/analysis/{symbol}/{analysis_type}",
            get(analysis::get_analysis),
        )
        // GET /v2/capital-gains/{symbol}?range=<str>
        .route("/capital-gains/{symbol}", get(events::get_capital_gains))
        // GET /v2/capital-gains?symbols=<csv>&range=<str>
        .route("/capital-gains", get(events::get_batch_capital_gains))
        .route("/calendar", get(calendar::get_calendar))
        // GET /v2/chart/{symbol}?interval=<str>&range=<str>&events=<bool>&patterns=<bool>
        .route("/chart/{symbol}", get(chart::get_chart))
        // GET /v2/charts?symbols=<csv>&interval=<str>&range=<str>&patterns=<bool>
        .route("/charts", get(chart::get_batch_charts))
        // GET /v2/crypto/coins?vs_currency=<str>&count=<u32>
        .route("/crypto/coins", get(crypto::get_crypto_coins))
        // GET /v2/crypto/coins/{id}?vs_currency=<str>
        .route("/crypto/coins/{id}", get(crypto::get_crypto_coin))
        // GET /v2/currencies
        .route("/currencies", get(metadata::get_currencies))
        // GET /v2/dividends/{symbol}?range=<str>
        .route("/dividends/{symbol}", get(events::get_dividends))
        // GET /v2/dividends?symbols=<csv>&range=<str>
        .route("/dividends", get(events::get_batch_dividends))
        // GET /v2/edgar/cik/{symbol}
        .route("/edgar/cik/{symbol}", get(edgar::get_edgar_cik))
        // GET /v2/edgar/facts/{symbol}
        .route("/edgar/facts/{symbol}", get(edgar::get_edgar_facts))
        // GET /v2/edgar/search?q=<string>&forms=<csv>&start_date=<date>&end_date=<date>
        .route("/edgar/search", get(edgar::get_edgar_search))
        // GET /v2/edgar/submissions/{symbol}
        .route(
            "/edgar/submissions/{symbol}",
            get(edgar::get_edgar_submissions),
        )
        // GET /v2/exchanges
        .route("/exchanges", get(metadata::get_exchanges))
        // GET /v2/fear-and-greed
        .route("/fear-and-greed", get(market::get_fear_and_greed))
        // GET /v2/feeds?sources=<csv>&form_type=<str>
        .route("/feeds", get(feeds::get_feeds))
        // GET /v2/feeds/stream - WebSocket continuous RSS/Atom feed entries
        .route("/feeds/stream", get(feeds_stream::ws_feeds_stream_handler))
        // GET /v2/financials/{symbol}/{statement}?frequency=<annual|quarterly>
        .route(
            "/financials/{symbol}/{statement}",
            get(financials::get_financials),
        )
        // GET /v2/financials?symbols=<csv>&statement=<str>&frequency=<str>
        .route("/financials", get(financials::get_batch_financials))
        // GET /v2/fred/series/{id}
        .route("/fred/series/{id}", get(fred::get_fred_series))
        // GET /v2/fred/treasury-yields?year=<u32>
        .route("/fred/treasury-yields", get(fred::get_fred_treasury_yields))
        // GET /v2/health - version-prefixed health check
        .route("/health", get(system::health_check))
        // GET /v2/holders/{symbol}/{holder_type}
        .route("/holders/{symbol}/{holder_type}", get(holders::get_holders))
        // GET /v2/hours
        .route("/hours", get(metadata::get_hours))
        // GET /v2/indicators/{symbol}?interval=<str>&range=<str>
        .route("/indicators/{symbol}", get(indicators::get_indicators))
        // GET /v2/indicators?symbols=<csv>&interval=<str>&range=<str>
        .route("/indicators", get(indicators::get_batch_indicators))
        // GET /v2/indices?format=<raw|pretty|both>
        .route("/indices", get(market::get_indices))
        // GET /v2/industries/{industry}
        .route("/industries/{industry}", get(sector::get_industry))
        // GET /v2/lookup?q=<string>&type=<string>&count=<u32>&logo=<bool>
        .route("/lookup", get(search::lookup))
        // GET /v2/market-summary
        .route("/market-summary", get(market::get_market_summary))
        // GET /v2/news?count=<u32>
        .route("/news", get(news::get_general_news))
        // GET /v2/news/{symbol}?count=<u32>
        .route("/news/{symbol}", get(news::get_news))
        // GET /v2/options/{symbol}?date=<i64>
        .route("/options/{symbol}", get(options::get_options))
        // GET /v2/options?symbols=<csv>&date=<i64>
        .route("/options", get(options::get_batch_options))
        // GET /v2/ping - version-prefixed ping
        .route("/ping", get(system::ping))
        // GET /v2/quote/{symbol}?logo=<bool>
        .route("/quote/{symbol}", get(quote::get_quote))
        // GET /v2/quote-type/{symbol}
        .route("/quote-type/{symbol}", get(metadata::get_quote_type))
        // GET /v2/quotes?symbols=<csv>&logo=<bool>
        .route("/quotes", get(quote::get_quotes))
        // GET /v2/recommendations/{symbol}?limit=<u32>
        .route(
            "/recommendations/{symbol}",
            get(analysis::get_recommendations),
        )
        // GET /v2/recommendations?symbols=<csv>&limit=<u32>
        .route("/recommendations", get(analysis::get_batch_recommendations))
        // GET /v2/risk/{symbol}?interval=<str>&range=<str>&benchmark=<str>
        .route("/risk/{symbol}", get(risk::get_risk))
        // GET /v2/screeners/{screener}?count=<u32>
        .route("/screeners/{screener}", get(screener::get_screeners))
        // POST /v2/screeners/custom
        .route("/screeners/custom", post(screener::post_custom_screener))
        // GET /v2/search?q=<string>&hits=<u32>
        .route("/search", get(search::search))
        // GET /v2/sectors/{sector}
        .route("/sectors/{sector}", get(sector::get_sector))
        // GET /v2/spark?symbols=<csv>&interval=<str>&range=<str>
        .route("/spark", get(chart::get_spark))
        // GET /v2/splits/{symbol}?range=<str>
        .route("/splits/{symbol}", get(events::get_splits))
        // GET /v2/splits?symbols=<csv>&range=<str>
        .route("/splits", get(events::get_batch_splits))
        // GET /v2/stream - WebSocket real-time price streaming
        .route("/stream", get(stream::ws_stream_handler))
        // GET /v2/transcripts/{symbol}?quarter=<str>&year=<i32>
        .route("/transcripts/{symbol}", get(transcripts::get_transcript))
        // GET /v2/transcripts/{symbol}/all?limit=<usize>
        .route(
            "/transcripts/{symbol}/all",
            get(transcripts::get_transcripts),
        )
        // GET /v2/trending?region=<str>
        .route("/trending", get(market::get_trending))
        // GET /v2/metrics - Prometheus metrics endpoint
        .route("/metrics", get(system::get_metrics))
}
