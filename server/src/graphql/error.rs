//! GraphQL error mapping from library errors.

use async_graphql::{Error, ErrorExtensions};
use finance_query::FinanceError;

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
