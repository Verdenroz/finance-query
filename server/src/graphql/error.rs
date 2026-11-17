//! GraphQL error mapping from library errors.

use async_graphql::{Error, ErrorExtensions};
use finance_query::FinanceError;
use serde::de::DeserializeOwned;
use std::future::Future;

/// Deserialize a service-layer JSON response into its typed GraphQL shape.
///
/// Collapses the `serde_json::from_value(json).map_err(|e| async_graphql::Error::new(e.to_string()))`
/// tail repeated across nearly every resolver.
pub fn from_gql_json<T: DeserializeOwned>(json: serde_json::Value) -> Result<T, Error> {
    serde_json::from_value(json).map_err(|e| Error::new(e.to_string()))
}

/// Run a service-layer future, map its error via [`to_gql_error`], and
/// deserialize the resulting JSON into `T` in one step.
///
/// Fits the common resolver shape of "call a service, get `T` back" with no
/// intermediate processing (format transforms, pagination, filtering, ...) —
/// those still compose `to_gql_error`/`from_gql_json` directly.
pub async fn exec_gql<T, F>(fut: F) -> Result<T, Error>
where
    T: DeserializeOwned,
    F: Future<Output = Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>>>,
{
    let json = fut.await.map_err(to_gql_error)?;
    from_gql_json(json)
}

/// Convert a service-layer error into an `async_graphql::Error` with extensions.
pub fn to_gql_error(err: Box<dyn std::error::Error + Send + Sync>) -> Error {
    if let Some(fe) = err.downcast_ref::<FinanceError>() {
        return finance_error_to_gql(fe);
    }
    Error::new(err.to_string()).extend_with(|_, e| {
        e.set("code", "INTERNAL_ERROR");
        e.set("status", 500);
    })
}

fn finance_error_to_gql(err: &FinanceError) -> Error {
    let (code, status) = match err {
        FinanceError::SymbolNotFound { .. } => ("NOT_FOUND", 404),
        FinanceError::InvalidParameter { .. } => ("BAD_REQUEST", 400),
        FinanceError::RateLimited { .. } => ("RATE_LIMITED", 429),
        FinanceError::Timeout { .. } => ("TIMEOUT", 504),
        // Keyless adapters keep the raw `HttpError`, so their timeouts arrive
        // here rather than as `Timeout`.
        FinanceError::HttpError(e) if e.is_timeout() => ("TIMEOUT", 504),
        FinanceError::HttpError(_) | FinanceError::NetworkError { .. } => ("BAD_GATEWAY", 502),
        FinanceError::AuthenticationFailed { .. } => ("UNAUTHORIZED", 401),
        FinanceError::NotEntitled { .. } => ("FORBIDDEN", 403),
        FinanceError::ServerError { status, .. } => ("SERVER_ERROR", *status),
        FinanceError::NotSupported { .. }
        | FinanceError::NoProviderSupports { .. }
        | FinanceError::NoProviderAvailable { .. }
        | FinanceError::ProviderNotConfigured { .. } => ("NOT_SUPPORTED", 501),
        _ => ("INTERNAL_ERROR", 500),
    };

    Error::new(client_message(err)).extend_with(|_, e| {
        e.set("code", code);
        e.set("status", status);
    })
}

const LIBRARY_ROUTING_HINT: &str = "; route it ";

/// Error text safe to show API clients: transport failures name only the
/// upstream host (a `reqwest::Error` renders the full request URL), and the
/// library-only `Providers::builder()` routing advice is dropped.
fn client_message(err: &FinanceError) -> String {
    if let FinanceError::HttpError(e) = err {
        let host = e.url().and_then(|u| u.host_str()).unwrap_or("upstream");
        return if e.is_timeout() {
            format!("Request to {host} timed out")
        } else {
            format!("Request to {host} failed")
        };
    }
    let text = err.to_string();
    match text.find(LIBRARY_ROUTING_HINT) {
        Some(at) => text[..at].to_string(),
        None => text,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use finance_query::{Operation, Provider};

    /// A `reqwest::Error` that `is_timeout()`, from a socket that accepts and
    /// never answers.
    async fn timed_out() -> reqwest::Error {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let _held = listener.accept().await;
            std::future::pending::<()>().await;
        });
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(50))
            .build()
            .unwrap()
            .get(format!("http://{addr}/"))
            .send()
            .await
            .unwrap_err()
    }

    fn status_of(err: &FinanceError) -> Option<async_graphql::Value> {
        finance_error_to_gql(err)
            .extensions
            .and_then(|ext| ext.get("status").cloned())
    }

    async fn refused() -> reqwest::Error {
        reqwest::Client::new()
            .get("http://127.0.0.1:1/secret/path?token=abc")
            .send()
            .await
            .unwrap_err()
    }

    #[tokio::test]
    async fn a_keyless_transport_timeout_is_a_gateway_timeout() {
        let err = FinanceError::HttpError(timed_out().await);
        assert_eq!(status_of(&err), Some(async_graphql::Value::from(504)));
    }

    #[tokio::test]
    async fn a_transport_failure_that_is_not_a_timeout_is_a_bad_gateway() {
        let err = FinanceError::HttpError(refused().await);
        assert_eq!(status_of(&err), Some(async_graphql::Value::from(502)));
    }

    #[test]
    fn library_timeouts_and_withheld_network_errors_use_gateway_statuses() {
        let timeout = FinanceError::Timeout { timeout_ms: 5 };
        assert_eq!(status_of(&timeout), Some(async_graphql::Value::from(504)));
        let net = FinanceError::NetworkError { api: "x".into() };
        assert_eq!(status_of(&net), Some(async_graphql::Value::from(502)));
    }

    #[tokio::test]
    async fn transport_messages_name_the_host_only() {
        let msg = client_message(&FinanceError::HttpError(refused().await));
        assert_eq!(msg, "Request to 127.0.0.1 failed");
        let msg = client_message(&FinanceError::HttpError(timed_out().await));
        assert_eq!(msg, "Request to 127.0.0.1 timed out");
    }

    #[test]
    fn rest_clients_never_see_the_library_routing_hint() {
        let not_supported = FinanceError::NotSupported {
            provider: Provider::Finra,
            operation: Operation::PriceTargetSummary,
            candidates: vec![Provider::Finra, Provider::Fmp],
        };
        let all = FinanceError::NoProviderSupports {
            operation: Operation::PriceTargetSummary,
            tried: vec![Provider::Finra, Provider::Fmp],
        };
        assert!(not_supported.to_string().contains("Providers::builder"));
        for err in [not_supported, all] {
            let msg = client_message(&err);
            assert!(!msg.contains("Providers::builder"), "{msg}");
            assert!(!msg.contains("route it"), "{msg}");
        }
    }
}
