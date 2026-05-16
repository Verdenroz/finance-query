//! Compile and runtime tests for docs/library/error-handling.md
//!
//! Run with: `cargo test --test doc_error_handling`

use finance_query::{ErrorCategory, FinanceError};

// ---------------------------------------------------------------------------
// Error enum — compile-time variant existence checks
// ---------------------------------------------------------------------------

/// Verifies all documented error variants exist.
#[allow(dead_code)]
fn _verify_error_variants() {
    let _ = FinanceError::AuthenticationFailed {
        context: String::new(),
    };
    let _ = FinanceError::SymbolNotFound {
        symbol: None,
        context: String::new(),
    };
    let _ = FinanceError::RateLimited { retry_after: None };
    // Note: FinanceError::HttpError wraps reqwest::Error, which cannot be
    // constructed without a real HTTP request. Verified at type level.
    let _ =
        FinanceError::JsonParseError(serde_json::from_str::<serde_json::Value>("").unwrap_err());
    let _ = FinanceError::ResponseStructureError {
        field: String::new(),
        context: String::new(),
    };
    let _ = FinanceError::InvalidParameter {
        param: String::new(),
        reason: String::new(),
    };
    let _ = FinanceError::Timeout { timeout_ms: 5000 };
    let _ = FinanceError::ServerError {
        status: 500,
        context: String::new(),
    };
    let _ = FinanceError::UnexpectedResponse(String::new());
    let _ = FinanceError::InternalError(String::new());
    let _ = FinanceError::ApiError(String::new());
    let _ = FinanceError::RuntimeError(std::io::Error::other(""));
    let _ = FinanceError::MacroDataError {
        provider: String::new(),
        context: String::new(),
    };
    let _ = FinanceError::FeedParseError {
        url: String::new(),
        context: String::new(),
    };
    let _ = FinanceError::NotSupported {
        provider: "test",
        operation: "quote",
    };
    let _ = FinanceError::NoProviderAvailable { operation: "chart" };
}

// ---------------------------------------------------------------------------
// ErrorCategory — compile-time variant existence
// ---------------------------------------------------------------------------

/// Verifies all ErrorCategory variants exist.
#[allow(dead_code)]
fn _verify_error_categories() {
    let _: ErrorCategory = ErrorCategory::Auth;
    let _: ErrorCategory = ErrorCategory::RateLimit;
    let _: ErrorCategory = ErrorCategory::Timeout;
    let _: ErrorCategory = ErrorCategory::Server;
    let _: ErrorCategory = ErrorCategory::NotFound;
    let _: ErrorCategory = ErrorCategory::Validation;
    let _: ErrorCategory = ErrorCategory::Parsing;
    let _: ErrorCategory = ErrorCategory::Other;
}

// ---------------------------------------------------------------------------
// is_retriable — compile-time verification
// ---------------------------------------------------------------------------

/// Verifies the documented retriable variants return true.
#[test]
fn test_retriable_errors() {
    assert!(FinanceError::RateLimited { retry_after: None }.is_retriable());
    assert!(FinanceError::Timeout { timeout_ms: 5000 }.is_retriable());
    assert!(
        FinanceError::AuthenticationFailed {
            context: String::new()
        }
        .is_retriable()
    );
    assert!(
        FinanceError::ServerError {
            status: 503,
            context: String::new()
        }
        .is_retriable()
    );
}

/// Verifies the documented non-retriable variants return false.
#[test]
fn test_non_retriable_errors() {
    assert!(
        !FinanceError::SymbolNotFound {
            symbol: None,
            context: String::new()
        }
        .is_retriable()
    );
    assert!(
        !FinanceError::InvalidParameter {
            param: String::new(),
            reason: String::new()
        }
        .is_retriable()
    );
    assert!(
        !FinanceError::NotSupported {
            provider: "test",
            operation: "quote"
        }
        .is_retriable()
    );
}

// ---------------------------------------------------------------------------
// retry_after_secs — compile-time verification
// ---------------------------------------------------------------------------

/// Verifies retry_after_secs returns Some for retriable variants.
#[test]
fn test_retry_after_secs() {
    assert_eq!(
        FinanceError::RateLimited {
            retry_after: Some(10)
        }
        .retry_after_secs(),
        Some(10)
    );
    assert_eq!(
        FinanceError::Timeout { timeout_ms: 5000 }.retry_after_secs(),
        Some(2)
    );
    assert_eq!(
        FinanceError::ServerError {
            status: 503,
            context: String::new()
        }
        .retry_after_secs(),
        Some(5)
    );
    assert_eq!(
        FinanceError::SymbolNotFound {
            symbol: None,
            context: String::new()
        }
        .retry_after_secs(),
        None
    );
}

// ---------------------------------------------------------------------------
// category — compile-time verification
// ---------------------------------------------------------------------------

/// Verifies category returns the documented ErrorCategory for each variant.
#[test]
fn test_error_category() {
    assert_eq!(
        FinanceError::AuthenticationFailed {
            context: String::new()
        }
        .category(),
        ErrorCategory::Auth
    );
    assert_eq!(
        FinanceError::RateLimited { retry_after: None }.category(),
        ErrorCategory::RateLimit
    );
    assert_eq!(
        FinanceError::Timeout { timeout_ms: 5000 }.category(),
        ErrorCategory::Timeout
    );
    assert_eq!(
        FinanceError::ServerError {
            status: 500,
            context: String::new()
        }
        .category(),
        ErrorCategory::Server
    );
    assert_eq!(
        FinanceError::SymbolNotFound {
            symbol: None,
            context: String::new()
        }
        .category(),
        ErrorCategory::NotFound
    );
    assert_eq!(
        FinanceError::InvalidParameter {
            param: String::new(),
            reason: String::new()
        }
        .category(),
        ErrorCategory::Validation
    );
}

// ---------------------------------------------------------------------------
// Fluent API — with_symbol / with_context
// ---------------------------------------------------------------------------

/// Verifies with_symbol sets the symbol field on SymbolNotFound.
#[test]
fn test_with_symbol() {
    let error = FinanceError::SymbolNotFound {
        symbol: None,
        context: String::new(),
    }
    .with_symbol("AAPL");

    match error {
        FinanceError::SymbolNotFound { symbol, .. } => {
            assert_eq!(symbol, Some("AAPL".to_string()));
        }
        _ => panic!("Expected SymbolNotFound"),
    }
}

/// Verifies with_context sets context on AuthenticationFailed.
#[test]
fn test_with_context() {
    let error = FinanceError::AuthenticationFailed {
        context: String::new(),
    }
    .with_context("new context");

    match error {
        FinanceError::AuthenticationFailed { context } => {
            assert_eq!(context, "new context");
        }
        _ => panic!("Expected AuthenticationFailed"),
    }
}
