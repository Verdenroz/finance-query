use rmcp::ErrorData as McpError;

pub fn ser_err(e: serde_json::Error) -> McpError {
    McpError::internal_error(format!("Serialization error: {e}"), None)
}

pub fn invalid_params(msg: impl Into<String>) -> McpError {
    McpError::invalid_params(msg.into(), None)
}
