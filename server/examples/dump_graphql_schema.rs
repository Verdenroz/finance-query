//! Prints the current GraphQL SDL to stdout.
//!
//! Regenerate the checked-in snapshot with:
//! ```text
//! cargo run --example dump_graphql_schema -p finance-query-server > server/schema.graphql
//! ```
//! `tests/graphql_schema.rs` fails CI if the checked-in file drifts from
//! this, and `cargo soothfast spec check` reconciles it against every
//! `#[soothfast::route(spec = "schema.graphql", ...)]` in `benches/soothfast.rs`.

use finance_query_server::{AppState, FeedHub, StreamHub, cache::Cache, graphql};

#[tokio::main]
async fn main() {
    let state = AppState {
        cache: Cache::new(None).await,
        stream_hub: StreamHub::new(),
        feed_hub: FeedHub::new(),
    };
    print!("{}", graphql::build_schema(state).sdl());
}
