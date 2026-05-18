//! Maps `finance_query::FinanceError` variants to typed Python exceptions.
//!
//! This is the only module that bridges Rust errors to Python errors.
//! Every other binding uses `?` to propagate.

use finance_query::FinanceError;
use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;

create_exception!(_finance_query, FinanceQueryError, PyException);
create_exception!(_finance_query, NetworkError, FinanceQueryError);
create_exception!(_finance_query, RateLimitError, FinanceQueryError);
create_exception!(_finance_query, SymbolNotFound, FinanceQueryError);
create_exception!(_finance_query, ParseError, FinanceQueryError);
create_exception!(_finance_query, ConfigError, FinanceQueryError);

/// Convert a `FinanceError` into the appropriate typed Python exception.
#[allow(dead_code)] // Consumed by binding modules added in later tasks.
pub fn to_py_err(err: FinanceError) -> PyErr {
    let msg = err.to_string();
    match err {
        FinanceError::AuthenticationFailed { .. }
        | FinanceError::InvalidParameter { .. } => ConfigError::new_err(msg),

        FinanceError::SymbolNotFound { .. } => SymbolNotFound::new_err(msg),

        FinanceError::RateLimited { .. } => RateLimitError::new_err(msg),

        FinanceError::HttpError(_)
        | FinanceError::Timeout { .. }
        | FinanceError::ServerError { .. }
        | FinanceError::ExternalApiError { .. }
        | FinanceError::MacroDataError { .. } => NetworkError::new_err(msg),

        FinanceError::JsonParseError(_)
        | FinanceError::ResponseStructureError { .. }
        | FinanceError::UnexpectedResponse(_)
        | FinanceError::FeedParseError { .. } => ParseError::new_err(msg),

        // Catch-alls map to the base type.
        //
        // `IndicatorError` is gated on the upstream `finance-query` crate's
        // `indicators` feature, which our Cargo.toml unconditionally enables,
        // so the variant is always present here.
        FinanceError::InternalError(_)
        | FinanceError::ApiError(_)
        | FinanceError::RuntimeError(_)
        | FinanceError::IndicatorError(_) => FinanceQueryError::new_err(msg),
    }
}

/// Register all exception types on the module.
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("FinanceQueryError", m.py().get_type::<FinanceQueryError>())?;
    m.add("NetworkError", m.py().get_type::<NetworkError>())?;
    m.add("RateLimitError", m.py().get_type::<RateLimitError>())?;
    m.add("SymbolNotFound", m.py().get_type::<SymbolNotFound>())?;
    m.add("ParseError", m.py().get_type::<ParseError>())?;
    m.add("ConfigError", m.py().get_type::<ConfigError>())?;
    Ok(())
}
