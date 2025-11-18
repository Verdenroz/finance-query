use thiserror::Error;

/// Main error type for the library
#[derive(Error, Debug)]
pub enum YahooError {
    /// Authentication with Yahoo Finance failed
    #[error("Yahoo Finance authentication failed")]
    AuthenticationFailed,

    /// The requested symbol was not found
    #[error("Symbol not found: {0}")]
    SymbolNotFound(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded - too many requests")]
    RateLimited,

    /// HTTP request error
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    /// Failed to parse JSON response
    #[error("Failed to parse JSON response: {0}")]
    JsonParseError(#[from] serde_json::Error),

    /// Failed to parse response data
    #[error("Failed to parse response data: {0}")]
    ParseError(String),

    /// Invalid parameter provided
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// Network timeout
    #[error("Request timed out")]
    Timeout,

    /// Unexpected response from Yahoo Finance
    #[error("Unexpected response: {0}")]
    UnexpectedResponse(String),

    /// Internal error
    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Result type alias for library operations
pub type Result<T> = std::result::Result<T, YahooError>;

impl YahooError {
    /// Check if this error is retriable
    pub fn is_retriable(&self) -> bool {
        matches!(
            self,
            YahooError::Timeout
                | YahooError::RateLimited
                | YahooError::HttpError(_)
                | YahooError::AuthenticationFailed
        )
    }

    /// Check if this error indicates an authentication issue
    pub fn is_auth_error(&self) -> bool {
        matches!(self, YahooError::AuthenticationFailed)
    }

    /// Check if this error indicates a not found issue
    pub fn is_not_found(&self) -> bool {
        matches!(self, YahooError::SymbolNotFound(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_is_retriable() {
        assert!(YahooError::Timeout.is_retriable());
        assert!(YahooError::RateLimited.is_retriable());
        assert!(YahooError::AuthenticationFailed.is_retriable());
        assert!(!YahooError::SymbolNotFound("AAPL".to_string()).is_retriable());
        assert!(!YahooError::InvalidParameter("test".to_string()).is_retriable());
    }

    #[test]
    fn test_error_is_auth_error() {
        assert!(YahooError::AuthenticationFailed.is_auth_error());
        assert!(!YahooError::Timeout.is_auth_error());
    }

    #[test]
    fn test_error_is_not_found() {
        assert!(YahooError::SymbolNotFound("AAPL".to_string()).is_not_found());
        assert!(!YahooError::Timeout.is_not_found());
    }
}
