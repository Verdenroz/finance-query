use rmcp::ErrorData as McpError;

pub fn ser_err(e: serde_json::Error) -> McpError {
    McpError::internal_error(format!("Serialization error: {e}"), None)
}

pub fn invalid_params(msg: impl Into<String>) -> McpError {
    McpError::invalid_params(msg.into(), None)
}

/// Map a library-level error (`finance_query::FinanceError`,
/// `finance_query::backtesting::BacktestError`, ...) to an `McpError` by
/// message passthrough — used by tools that call the library directly
/// instead of bridging through the GraphQL schema (see `tools::backtest`,
/// `tools::providers`), where `gql::gql_errors_to_mcp` doesn't apply.
/// Only referenced by cfg-gated tool modules, so it's unused (and would
/// warn) in a minimal build with none of those features enabled.
#[allow(dead_code)]
pub fn lib_err(e: impl std::fmt::Display) -> McpError {
    McpError::internal_error(e.to_string(), None)
}
