//! Shared Relay-style cursor pagination primitive for GraphQL list fields.
//!
//! Cursor = `async_graphql::connection::OpaqueCursor<usize>` — a base64-encoded
//! offset. Upstream sources here (Yahoo, EDGAR, FRED, RSS) have no real keyset
//! APIs, so offset-in-a-cursor is the only backing data available; wrapping it
//! in `OpaqueCursor` gives callers a proper opaque Relay token instead of a raw
//! integer, entirely via the crate's built-in machinery (no custom encode/decode
//! code needed).
//!
//! Forward-only: every paginated field exposes `first`/`after` only (no
//! `last`/`before`) — nothing in this codebase has a backward-pagination use
//! case, so callers of the two builders below always pass `None` for them.

use async_graphql::connection::{Connection, CursorType, Edge, OpaqueCursor, query};
use async_graphql::{OutputType, Result};
use std::future::Future;

/// The `Connection` type every paginated GraphQL field returns.
pub type Page<T> = Connection<OpaqueCursor<usize>, T>;

/// Paginate a slice already held in memory — the common case (feeds, holders,
/// dividends, chart candles, news, search quotes, batch results, FRED
/// observations, EDGAR fact data points): all fetched in full from upstream
/// today, with Rust-side slicing as the only lever available.
///
/// Takes `&[T]` rather than an owned `Vec<T>` so callers don't have to clone
/// the entire backing list just to slice out one page — only the returned
/// page's items are cloned.
pub async fn paginate<T: OutputType + Clone>(
    items: &[T],
    first: Option<i32>,
    after: Option<String>,
) -> Result<Page<T>> {
    let total = items.len();
    query(
        after,
        None,
        first,
        None,
        |after: Option<OpaqueCursor<usize>>, _before, first, _last| async move {
            let start = after.map(|OpaqueCursor(o)| o + 1).unwrap_or(0).min(total);
            let end = first.map(|f| (start + f).min(total)).unwrap_or(total);
            let mut connection = Connection::new(start > 0, end < total);
            connection.edges.extend(
                items[start..end]
                    .iter()
                    .cloned()
                    .enumerate()
                    .map(|(i, node)| Edge::new(OpaqueCursor(start + i), node)),
            );
            Ok::<_, async_graphql::Error>(connection)
        },
    )
    .await
}

/// Paginate via an upstream fetcher that itself supports offset/limit and
/// reports a known total (screener, EDGAR search, lookup) — avoids fetching
/// every item just to slice it in Rust.
pub async fn paginate_with_fetcher<T, F, Fut>(
    total: usize,
    first: Option<i32>,
    after: Option<String>,
    fetch_page: F,
) -> Result<Page<T>>
where
    T: OutputType,
    F: FnOnce(usize, usize) -> Fut,
    Fut: Future<Output = Result<Vec<T>>>,
{
    query(
        after,
        None,
        first,
        None,
        |after: Option<OpaqueCursor<usize>>, _before, first, _last| async move {
            let start = after.map(|OpaqueCursor(o)| o + 1).unwrap_or(0);
            let limit = first.unwrap_or_else(|| total.saturating_sub(start));
            let end = (start + limit).min(total);
            let page_items = fetch_page(start, end.saturating_sub(start)).await?;
            let mut connection = Connection::new(start > 0, end < total);
            connection.edges.extend(
                page_items
                    .into_iter()
                    .enumerate()
                    .map(|(i, node)| Edge::new(OpaqueCursor(start + i), node)),
            );
            Ok::<_, async_graphql::Error>(connection)
        },
    )
    .await
}

/// Pagination metadata for fields whose pagination genuinely happens at the
/// *input* level (an `offset`/`size`-style arg the upstream API already
/// consumes to fetch the requested slice directly — screener's `customScreener`,
/// EDGAR's `edgarSearch`) rather than by fetching everything and slicing in
/// Rust. These fields keep returning a plain `Vec<T>` (no `edges`/`node`
/// wrapping — that would just be redundant pagination vocabulary layered on
/// top of the offset/size that already did the real work) and add this as a
/// `pageInfo` sibling field alongside their existing `total`/`totalHits`.
///
/// A distinct type from `async_graphql::connection::PageInfo` (which is
/// `#[graphql(internal)]` and doesn't derive `Clone`/`Debug`, so it isn't
/// meant for reuse in ordinary struct fields) — same shape, usable directly.
#[derive(async_graphql::SimpleObject, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlPageInfo {
    pub has_previous_page: bool,
    pub has_next_page: bool,
    pub start_cursor: Option<String>,
    pub end_cursor: Option<String>,
}

pub fn offset_page_info(offset: usize, size: usize, total: Option<i64>) -> GqlPageInfo {
    let end = offset + size;
    let has_next_page = total.map(|t| (end as i64) < t).unwrap_or(false);
    GqlPageInfo {
        has_previous_page: offset > 0,
        has_next_page,
        start_cursor: Some(OpaqueCursor(offset).encode_cursor()),
        end_cursor: Some(OpaqueCursor(end.saturating_sub(1).max(offset)).encode_cursor()),
    }
}

/// Build the outer `{ edges { node <inner> } pageInfo { hasNextPage endCursor } }`
/// selection wrapping an already-built inner (per-item) selection, for REST/MCP
/// bridges that query a paginated field. Shared since it's identical string
/// shaping regardless of consumer default policy.
pub fn build_connection_selection(inner: &str) -> String {
    format!("{{ edges {{ node {inner} }} pageInfo {{ hasNextPage endCursor }} }}")
}

/// Pull the list of `node` values out of a Connection JSON value
/// (`{edges:[{node,...}], pageInfo:{...}}`).
pub fn connection_nodes(data: &serde_json::Value) -> Vec<serde_json::Value> {
    data.get("edges")
        .and_then(|e| e.as_array())
        .map(|edges| {
            edges
                .iter()
                .filter_map(|e| e.get("node").cloned())
                .collect()
        })
        .unwrap_or_default()
}

/// Pull the `pageInfo` object out of a Connection JSON value.
pub fn connection_page_info(data: &serde_json::Value) -> serde_json::Value {
    data.get("pageInfo")
        .cloned()
        .unwrap_or(serde_json::Value::Null)
}

/// Build a composite selection where exactly one composite field is itself a
/// paginated Connection (needs `first`/`after` args + `edges { node }
/// pageInfo` shape) while every other composite field keeps the normal static
/// nested-selection mechanism (`build_rest_composite_selection`'s pattern).
/// Used for wrapper types where only one nested list is paginated: dividends,
/// chart candles, search quotes, holders ownership/transaction/roster lists.
///
/// `default_fields` is the omitted-`fields` fallback — callers pass
/// `valid_fields` again for REST's "omitted = everything" convention, or a
/// curated `*_DEFAULT_FIELDS` constant for MCP's smaller-by-default policy;
/// the two conventions can otherwise differ (e.g. search's MCP default is
/// `["quotes","news"]`, a strict subset of its valid fields).
#[allow(clippy::too_many_arguments)]
pub fn build_paginated_composite_selection(
    fields: Option<&str>,
    valid_fields: &[&str],
    default_fields: &[&str],
    composite_fields: &[(&str, &str)],
    paginated_field: &str,
    paginated_inner_selection: &str,
    limit: Option<u32>,
    cursor: Option<&str>,
) -> String {
    let mut chosen: Vec<&str> = match fields {
        Some(raw) if !raw.trim().is_empty() => raw
            .split(',')
            .map(|s| s.trim())
            .filter(|f| !f.is_empty() && valid_fields.contains(f))
            .collect(),
        _ => default_fields.to_vec(),
    };
    if chosen.is_empty() {
        chosen = default_fields.to_vec();
    }
    let mut sel = String::from("{ ");
    for f in chosen {
        sel.push_str(f);
        if f == paginated_field {
            let mut args = Vec::new();
            if let Some(limit) = limit {
                args.push(format!("first: {limit}"));
            }
            if let Some(cursor) = cursor {
                args.push(format!(
                    "after: \"{}\"",
                    cursor.replace('\\', "\\\\").replace('"', "\\\"")
                ));
            }
            if !args.is_empty() {
                sel.push('(');
                sel.push_str(&args.join(", "));
                sel.push(')');
            }
            sel.push(' ');
            sel.push_str(&build_connection_selection(paginated_inner_selection));
        } else if let Some((_, nested)) = composite_fields.iter().find(|(n, _)| *n == f) {
            sel.push(' ');
            sel.push_str(nested);
        }
        sel.push(' ');
    }
    sel.push('}');
    sel
}

/// Reshape one nested field within an already-fetched object value from
/// Connection shape to REST's legacy bare-array (default) or `{items,
/// pageInfo}` (opt-in) shape. Companion to `build_paginated_composite_selection`
/// on the read side.
pub fn unwrap_nested_connection(
    mut data: serde_json::Value,
    field: &str,
    paginated: bool,
) -> serde_json::Value {
    if let Some(obj) = data.as_object_mut()
        && let Some(conn) = obj.get(field).cloned()
    {
        let nodes = connection_nodes(&conn);
        let replacement = if paginated {
            serde_json::json!({ "items": nodes, "pageInfo": connection_page_info(&conn) })
        } else {
            serde_json::Value::Array(nodes)
        };
        obj.insert(field.to_string(), replacement);
    }
    data
}

/// Always wrap one nested field within an object as `{items, pageInfo}` — the
/// MCP-side (always-on) companion to `unwrap_nested_connection`.
pub fn wrap_nested_connection(mut data: serde_json::Value, field: &str) -> serde_json::Value {
    if let Some(obj) = data.as_object_mut()
        && let Some(conn) = obj.get(field).cloned()
    {
        let replacement = serde_json::json!({
            "items": connection_nodes(&conn),
            "pageInfo": connection_page_info(&conn),
        });
        obj.insert(field.to_string(), replacement);
    }
    data
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_graphql::connection::CursorType;

    async fn page(items: Vec<i32>, first: Option<i32>, after: Option<String>) -> Page<i32> {
        paginate(&items, first, after).await.unwrap()
    }

    #[tokio::test]
    async fn first_page_with_no_args_returns_everything() {
        let conn = page(vec![1, 2, 3], None, None).await;
        assert_eq!(conn.edges.len(), 3);
        assert!(!conn.has_next_page);
        assert!(!conn.has_previous_page);
    }

    #[tokio::test]
    async fn first_limits_page_size_and_sets_has_next_page() {
        let conn = page(vec![1, 2, 3, 4, 5], Some(2), None).await;
        assert_eq!(conn.edges.len(), 2);
        assert_eq!(conn.edges[0].node, 1);
        assert_eq!(conn.edges[1].node, 2);
        assert!(conn.has_next_page);
        assert!(!conn.has_previous_page);
    }

    #[tokio::test]
    async fn after_cursor_resumes_from_next_item() {
        let first_page = page(vec![1, 2, 3, 4, 5], Some(2), None).await;
        let cursor = first_page.edges.last().unwrap().cursor.encode_cursor();
        let second_page = page(vec![1, 2, 3, 4, 5], Some(2), Some(cursor)).await;
        assert_eq!(second_page.edges.len(), 2);
        assert_eq!(second_page.edges[0].node, 3);
        assert_eq!(second_page.edges[1].node, 4);
        assert!(second_page.has_previous_page);
        assert!(second_page.has_next_page);
    }

    #[tokio::test]
    async fn last_page_has_no_next_page() {
        let conn = page(vec![1, 2, 3], Some(10), None).await;
        assert_eq!(conn.edges.len(), 3);
        assert!(!conn.has_next_page);
    }

    #[test]
    fn connection_nodes_extracts_node_values() {
        let data = serde_json::json!({
            "edges": [{"node": {"a": 1}}, {"node": {"a": 2}}],
            "pageInfo": {"hasNextPage": true, "endCursor": "xyz"}
        });
        let nodes = connection_nodes(&data);
        assert_eq!(
            nodes,
            vec![serde_json::json!({"a": 1}), serde_json::json!({"a": 2})]
        );
        assert_eq!(
            connection_page_info(&data),
            serde_json::json!({"hasNextPage": true, "endCursor": "xyz"})
        );
    }
}
