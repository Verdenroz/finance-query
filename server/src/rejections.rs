//! Rewrites extractor rejections (plain-text 4xx bodies) into the JSON
//! [`ApiError`] envelope the OpenAPI spec promises.

use axum::{
    Json,
    body::{self, Body},
    http::{HeaderMap, Request, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::responses::ApiError;

const MAX_REJECTION_BYTES: usize = 64 * 1024;

const WEBSOCKET_PATHS: [&str; 2] = ["/stream", "/feeds/stream"];

fn is_rewritable(status: StatusCode) -> bool {
    matches!(
        status,
        StatusCode::BAD_REQUEST
            | StatusCode::PAYLOAD_TOO_LARGE
            | StatusCode::UNSUPPORTED_MEDIA_TYPE
            | StatusCode::UNPROCESSABLE_ENTITY
    )
}

fn is_plain_text(headers: &HeaderMap) -> bool {
    headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.starts_with("text/plain"))
}

/// Converts plain-text 400/413/415/422 responses into `{"error", "status"}`
/// JSON, keeping the status and message. JSON responses and the WebSocket
/// upgrade routes pass through untouched.
///
/// Must wrap the router before it is nested under `/v2`, since the
/// WebSocket check matches the nested (prefix-stripped) path.
pub async fn json_rejections(request: Request<Body>, next: Next) -> Response {
    let skip = WEBSOCKET_PATHS.contains(&request.uri().path());
    let response = next.run(request).await;
    if skip || !is_rewritable(response.status()) || !is_plain_text(response.headers()) {
        return response;
    }

    let status = response.status();
    let message = match body::to_bytes(response.into_body(), MAX_REJECTION_BYTES).await {
        Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
        Err(_) => status
            .canonical_reason()
            .unwrap_or("Bad request")
            .to_string(),
    };
    (
        status,
        Json(ApiError {
            error: message,
            status: status.as_u16(),
        }),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::Router;
    use axum::extract::{Path, Query};
    use axum::middleware;
    use axum::routing::{get, post};
    use finance_query::{Interval, ValueFormat};
    use serde::Deserialize;
    use serde_json::Value;
    use tower::ServiceExt;

    #[derive(Deserialize)]
    #[allow(dead_code)]
    struct ChartParams {
        interval: Option<Interval>,
    }

    #[derive(Deserialize)]
    #[allow(dead_code)]
    struct SearchParams {
        q: String,
    }

    #[derive(Deserialize)]
    #[allow(dead_code)]
    struct Payload {
        name: String,
    }

    fn app() -> Router {
        Router::new()
            .route("/chart", get(|Query(_): Query<ChartParams>| async { "ok" }))
            .route(
                "/analysis/{kind}",
                get(|Path(_): Path<ValueFormat>| async { "ok" }),
            )
            .route(
                "/search",
                get(|Query(_): Query<SearchParams>| async { "ok" }),
            )
            .route("/custom", post(|Json(_): Json<Payload>| async { "ok" }))
            .route(
                "/json-error",
                get(|| async {
                    (
                        StatusCode::BAD_REQUEST,
                        Json(ApiError {
                            error: "mine".into(),
                            status: 400,
                        }),
                    )
                }),
            )
            .route(
                "/stream",
                get(|| async { (StatusCode::BAD_REQUEST, "upgrade required") }),
            )
            .layer(middleware::from_fn(json_rejections))
    }

    async fn call(req: Request<Body>) -> (StatusCode, String, String) {
        let res = app().oneshot(req).await.expect("router responds");
        let status = res.status();
        let content_type = res
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default()
            .to_string();
        let bytes = body::to_bytes(res.into_body(), usize::MAX)
            .await
            .expect("body reads");
        (
            status,
            content_type,
            String::from_utf8_lossy(&bytes).into_owned(),
        )
    }

    fn get_req(uri: &str) -> Request<Body> {
        Request::builder()
            .uri(uri)
            .body(Body::empty())
            .expect("request builds")
    }

    async fn assert_json_error(uri: &str, status: StatusCode, needle: &str) {
        let (got, content_type, body) = call(get_req(uri)).await;
        assert_eq!(got, status, "{uri}");
        assert_eq!(content_type, "application/json", "{uri}");
        let json: Value = serde_json::from_str(&body).expect("json body");
        assert_eq!(json["status"], status.as_u16());
        let message = json["error"].as_str().expect("error string");
        assert!(message.contains(needle), "{message}");
    }

    #[tokio::test]
    async fn bad_query_enum_is_json() {
        assert_json_error("/chart?interval=bogus", StatusCode::BAD_REQUEST, "interval").await;
    }

    #[tokio::test]
    async fn bad_path_enum_is_json() {
        assert_json_error("/analysis/bogus", StatusCode::BAD_REQUEST, "bogus").await;
    }

    #[tokio::test]
    async fn missing_required_query_param_is_json() {
        assert_json_error("/search", StatusCode::BAD_REQUEST, "q").await;
    }

    #[tokio::test]
    async fn malformed_post_body_is_json() {
        let req = Request::builder()
            .method("POST")
            .uri("/custom")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from("{not json"))
            .expect("request builds");
        let (status, content_type, body) = call(req).await;
        assert!(status.is_client_error());
        assert_eq!(content_type, "application/json");
        let json: Value = serde_json::from_str(&body).expect("json body");
        assert_eq!(json["status"], status.as_u16());
    }

    #[tokio::test]
    async fn existing_json_errors_are_untouched() {
        let (status, _, body) = call(get_req("/json-error")).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body, r#"{"error":"mine","status":400}"#);
    }

    #[tokio::test]
    async fn websocket_route_is_untouched() {
        let (status, content_type, body) = call(get_req("/stream")).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(content_type.starts_with("text/plain"));
        assert_eq!(body, "upgrade required");
    }
}
