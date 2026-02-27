use thiserror::Error;

/// Main error type for the library
#[derive(Error, Debug)]
pub enum FinanceError {
    /// Authentication failed (Yahoo Finance, SEC EDGAR, etc.)
    #[error("Authentication failed: {context}")]
    AuthenticationFailed {
        /// Error context
        context: String,
    },

    /// The requested symbol was not found
    #[error("Symbol not found: {}", symbol.as_ref().map(|s| s.as_str()).unwrap_or("unknown"))]
    SymbolNotFound {
        /// The symbol that was not found
        symbol: Option<String>,
        /// Additional context
        context: String,
    },

    /// Rate limit exceeded
    #[error("Rate limited (retry after {retry_after:?}s)")]
    RateLimited {
        /// Seconds until retry is allowed
        retry_after: Option<u64>,
    },

    /// HTTP request error
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    /// Failed to parse JSON response
    #[error("JSON parse error: {0}")]
    JsonParseError(#[from] serde_json::Error),

    /// Response structure error - missing or malformed fields
    #[error("Response structure error in '{field}': {context}")]
    ResponseStructureError {
        /// Field name that caused the error
        field: String,
        /// Error context
        context: String,
    },

    /// Invalid parameter provided
    #[error("Invalid parameter '{param}': {reason}")]
    InvalidParameter {
        /// Parameter name
        param: String,
        /// Reason for invalidity
        reason: String,
    },

    /// Network timeout
    #[error("Request timeout after {timeout_ms}ms")]
    Timeout {
        /// Timeout duration in milliseconds
        timeout_ms: u64,
    },

    /// Server error (5xx status codes)
    #[error("Server error {status}: {context}")]
    ServerError {
        /// HTTP status code
        status: u16,
        /// Error context
        context: String,
    },

    /// Unexpected API response
    #[error("Unexpected response: {0}")]
    UnexpectedResponse(String),

    /// Internal error
    #[error("Internal error: {0}")]
    InternalError(String),

    /// General API error
    #[error("API error: {0}")]
    ApiError(String),

    /// Tokio runtime error
    #[error("Runtime error: {0}")]
    RuntimeError(#[from] std::io::Error),

    /// Indicator calculation error
    #[cfg(feature = "indicators")]
    #[error("Indicator calculation error: {0}")]
    IndicatorError(#[from] crate::indicators::IndicatorError),

    /// Error from an external (non-Yahoo) data API
    #[error("External API error from '{api}': HTTP {status}")]
    ExternalApiError {
        /// Name of the external API (e.g., "alternative.me", "coingecko")
        api: String,
        /// HTTP status code returned
        status: u16,
    },

    /// Error fetching or parsing macro-economic data (FRED, Treasury, BLS)
    #[error("Macro data error from '{provider}': {context}")]
    MacroDataError {
        /// Provider name (e.g., "FRED", "US Treasury")
        provider: String,
        /// Error context
        context: String,
    },

    /// Error parsing an RSS/Atom feed
    #[error("Feed parse error for '{url}': {context}")]
    FeedParseError {
        /// Feed URL that failed
        url: String,
        /// Error context
        context: String,
    },
}

/// Error category for logging and metrics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    /// Authentication errors
    Auth,
    /// Rate limiting errors
    RateLimit,
    /// Timeout errors
    Timeout,
    /// Server errors (5xx)
    Server,
    /// Not found errors
    NotFound,
    /// Validation errors
    Validation,
    /// Parsing errors
    Parsing,
    /// Other errors
    Other,
}

/// Type alias for Error (for consistency with common Rust patterns)
pub type Error = FinanceError;

/// Result type alias for library operations
pub type Result<T> = std::result::Result<T, FinanceError>;

impl FinanceError {
    /// Check if this error is retriable
    pub fn is_retriable(&self) -> bool {
        matches!(
            self,
            FinanceError::Timeout { .. }
                | FinanceError::RateLimited { .. }
                | FinanceError::HttpError(_)
                | FinanceError::AuthenticationFailed { .. }
                | FinanceError::ServerError { .. }
        )
    }

    /// Check if this error indicates an authentication issue
    pub fn is_auth_error(&self) -> bool {
        matches!(self, FinanceError::AuthenticationFailed { .. })
    }

    /// Check if this error indicates a not found issue
    pub fn is_not_found(&self) -> bool {
        matches!(self, FinanceError::SymbolNotFound { .. })
    }

    /// Get retry delay in seconds (for exponential backoff)
    pub fn retry_after_secs(&self) -> Option<u64> {
        match self {
            Self::RateLimited { retry_after } => *retry_after,
            Self::Timeout { .. } => Some(2),
            Self::ServerError { status, .. } if *status >= 500 => Some(5),
            Self::AuthenticationFailed { .. } => Some(1),
            _ => None,
        }
    }

    /// Categorize errors for logging/metrics
    pub fn category(&self) -> ErrorCategory {
        match self {
            Self::AuthenticationFailed { .. } => ErrorCategory::Auth,
            Self::RateLimited { .. } => ErrorCategory::RateLimit,
            Self::Timeout { .. } => ErrorCategory::Timeout,
            Self::ServerError { .. } => ErrorCategory::Server,
            Self::SymbolNotFound { .. } => ErrorCategory::NotFound,
            Self::InvalidParameter { .. } => ErrorCategory::Validation,
            Self::JsonParseError(_) | Self::ResponseStructureError { .. } => ErrorCategory::Parsing,
            _ => ErrorCategory::Other,
        }
    }

    /// Add symbol context to error (fluent API)
    pub fn with_symbol(mut self, symbol: impl Into<String>) -> Self {
        if let Self::SymbolNotFound {
            symbol: ref mut s, ..
        } = self
        {
            *s = Some(symbol.into());
        }
        self
    }

    /// Add context to error (fluent API)
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        match self {
            Self::AuthenticationFailed {
                context: ref mut c, ..
            } => {
                *c = context.into();
            }
            Self::SymbolNotFound {
                context: ref mut c, ..
            } => {
                *c = context.into();
            }
            Self::ResponseStructureError {
                context: ref mut c, ..
            } => {
                *c = context.into();
            }
            Self::ServerError {
                context: ref mut c, ..
            } => {
                *c = context.into();
            }
            _ => {}
        }
        self
    }
}

// Backward compatibility: Allow ParseError to be created from String
impl FinanceError {
    /// Create a ParseError from a string (for backward compatibility)
    #[deprecated(since = "2.0.0", note = "Use ResponseStructureError instead")]
    pub fn parse_error(msg: impl Into<String>) -> Self {
        let msg = msg.into();
        Self::ResponseStructureError {
            field: "unknown".to_string(),
            context: msg,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_is_retriable() {
        assert!(FinanceError::Timeout { timeout_ms: 5000 }.is_retriable());
        assert!(FinanceError::RateLimited { retry_after: None }.is_retriable());
        assert!(
            FinanceError::AuthenticationFailed {
                context: "test".to_string()
            }
            .is_retriable()
        );
        assert!(
            FinanceError::ServerError {
                status: 500,
                context: "test".to_string()
            }
            .is_retriable()
        );
        assert!(
            !FinanceError::SymbolNotFound {
                symbol: Some("AAPL".to_string()),
                context: "test".to_string()
            }
            .is_retriable()
        );
        assert!(
            !FinanceError::InvalidParameter {
                param: "test".to_string(),
                reason: "invalid".to_string()
            }
            .is_retriable()
        );
    }

    #[test]
    fn test_error_is_auth_error() {
        assert!(
            FinanceError::AuthenticationFailed {
                context: "test".to_string()
            }
            .is_auth_error()
        );
        assert!(!FinanceError::Timeout { timeout_ms: 5000 }.is_auth_error());
    }

    #[test]
    fn test_error_is_not_found() {
        assert!(
            FinanceError::SymbolNotFound {
                symbol: Some("AAPL".to_string()),
                context: "test".to_string()
            }
            .is_not_found()
        );
        assert!(!FinanceError::Timeout { timeout_ms: 5000 }.is_not_found());
    }

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
                context: "test".to_string()
            }
            .retry_after_secs(),
            Some(5)
        );
        assert_eq!(
            FinanceError::SymbolNotFound {
                symbol: None,
                context: "test".to_string()
            }
            .retry_after_secs(),
            None
        );
    }

    #[test]
    fn test_error_category() {
        assert_eq!(
            FinanceError::AuthenticationFailed {
                context: "test".to_string()
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
            FinanceError::SymbolNotFound {
                symbol: None,
                context: "test".to_string()
            }
            .category(),
            ErrorCategory::NotFound
        );
    }

    #[test]
    fn test_with_symbol() {
        let error = FinanceError::SymbolNotFound {
            symbol: None,
            context: "test".to_string(),
        }
        .with_symbol("AAPL");

        if let FinanceError::SymbolNotFound { symbol, .. } = error {
            assert_eq!(symbol, Some("AAPL".to_string()));
        } else {
            panic!("Expected SymbolNotFound");
        }
    }

    #[test]
    fn test_with_context() {
        let error = FinanceError::AuthenticationFailed {
            context: "old".to_string(),
        }
        .with_context("new context");

        if let FinanceError::AuthenticationFailed { context } = error {
            assert_eq!(context, "new context");
        } else {
            panic!("Expected AuthenticationFailed");
        }
    }
}
