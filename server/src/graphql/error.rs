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
        FinanceError::RateLimited { .. } => ("RATE_LIMITED", 429),
        FinanceError::Timeout { .. } => ("TIMEOUT", 408),
        FinanceError::AuthenticationFailed { .. } => ("UNAUTHORIZED", 401),
        FinanceError::ServerError { status, .. } => ("SERVER_ERROR", *status),
        _ => ("INTERNAL_ERROR", 500),
    };

    Error::new(err.to_string()).extend_with(|_, e| {
        e.set("code", code);
        e.set("status", status);
    })
}
