//! Harness shared by the parameter-contract tests.
//!
//! Everything here drives the real axum extractors rather than a hand-rolled
//! parser, so an assertion cannot drift from what the server actually accepts.

#![allow(dead_code)]

use axum::extract::{FromRequestParts, Path, Query};
use axum::http::{Request, StatusCode};
use axum::routing::get;
use axum::{Router, body::Body};
use tower::ServiceExt;

/// Issue a GET against `app` and return its status and body.
pub async fn get_status(app: Router, uri: &str) -> (StatusCode, String) {
    let res = app
        .oneshot(
            Request::builder()
                .uri(uri)
                .body(Body::empty())
                .expect("request builds"),
        )
        .await
        .expect("router responds");
    let status = res.status();
    let bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .expect("body reads");
    (status, String::from_utf8_lossy(&bytes).into_owned())
}

/// Issue a JSON POST against `app` and return its status and body.
pub async fn post_status(app: Router, uri: &str, body: &str) -> (StatusCode, String) {
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(uri)
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .expect("request builds"),
        )
        .await
        .expect("router responds");
    let status = res.status();
    let bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .expect("body reads");
    (status, String::from_utf8_lossy(&bytes).into_owned())
}

/// A route whose sole path segment is extracted as `T`, echoing its `Debug`.
pub fn path_route<T>(template: &str) -> Router
where
    T: serde::de::DeserializeOwned + std::fmt::Debug + Send + 'static,
{
    Router::new().route(
        template,
        get(|Path(v): Path<T>| async move { format!("{v:?}") }),
    )
}

/// A route matching the `/{symbol}/{kind}` shape, echoing the typed second segment.
pub fn symbol_kind_route<T>(template: &str) -> Router
where
    T: serde::de::DeserializeOwned + std::fmt::Debug + Send + 'static,
{
    Router::new().route(
        template,
        get(|Path((_symbol, v)): Path<(String, T)>| async move { format!("{v:?}") }),
    )
}

/// A route that extracts `T` from the query string and reports success only.
pub fn query_route<T>(template: &str) -> Router
where
    T: serde::de::DeserializeOwned + Send + 'static,
{
    Router::new().route(template, get(|Query(_q): Query<T>| async move { "ok" }))
}

/// Deserialize a params struct from a query string through `Query<T>` itself.
pub async fn from_query<T>(query: &str) -> Result<T, String>
where
    T: serde::de::DeserializeOwned + Send + 'static,
{
    let (mut parts, _) = Request::builder()
        .uri(format!("/x?{query}"))
        .body(())
        .expect("request builds")
        .into_parts();
    Query::<T>::from_request_parts(&mut parts, &())
        .await
        .map(|Query(v)| v)
        .map_err(|e| format!("{e:?}"))
}
