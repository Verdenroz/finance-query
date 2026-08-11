//! Fails if the checked-in `server/schema.graphql` snapshot has drifted from
//! the live schema. Regenerate with:
//! `cargo run --example dump_graphql_schema -p finance-query-server > server/schema.graphql`

use finance_query_server::{AppState, FeedHub, StreamHub, cache::Cache, graphql};

#[tokio::test]
async fn schema_graphql_is_up_to_date() {
    let state = AppState {
        cache: Cache::new(None).await,
        stream_hub: StreamHub::new(),
        feed_hub: FeedHub::new(),
    };
    let live_sdl = graphql::build_schema(state).sdl();
    let checked_in =
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/schema.graphql"))
            .expect("server/schema.graphql should exist — see examples/dump_graphql_schema.rs");
    assert_eq!(
        live_sdl.trim(),
        checked_in.trim(),
        "server/schema.graphql is stale — regenerate with:\n\
         cargo run --example dump_graphql_schema -p finance-query-server > server/schema.graphql"
    );
}
