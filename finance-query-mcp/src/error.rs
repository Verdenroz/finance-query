use rmcp::ErrorData as McpError;

pub fn ser_err(e: serde_json::Error) -> McpError {
    McpError::internal_error(format!("Serialization error: {e}"), None)
}

pub fn invalid_params(msg: impl Into<String>) -> McpError {
    McpError::invalid_params(msg.into(), None)
}

/// Map a library-level error (`finance_query::backtesting::BacktestError`)
/// to an `McpError` by message passthrough, for tools like `tools::backtest`
/// that call the library directly instead of bridging through GraphQL.
/// Unused (and would warn) in a build without the `backtesting` feature.
#[allow(dead_code)]
pub fn lib_err(e: impl std::fmt::Display) -> McpError {
    McpError::internal_error(e.to_string(), None)
}
