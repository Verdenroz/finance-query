use finance_query::{LookupOptions, LookupType, finance, Screener};
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::{finance_err, invalid_params, ser_err};

pub async fn search(query: String) -> Result<CallToolResult, McpError> {
    let results = finance::search(&query, &finance_query::SearchOptions::default()).await.map_err(finance_err)?;
    let json = serde_json::to_string(&results).map_err(ser_err)?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(json)]))
}

pub async fn screener(screener_type: String, count: Option<u32>) -> Result<CallToolResult, McpError> {
    let s = screener_type.parse::<Screener>()
        .map_err(|_| invalid_params(format!("Invalid screener: '{screener_type}'. Valid types: {}", Screener::valid_types())))?;
    let n = count.unwrap_or(25);
    let results = finance::screener(s, n).await.map_err(finance_err)?;
    let json = serde_json::to_string(&results).map_err(ser_err)?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(json)]))
}

pub async fn get_lookup(query: String, query_type: Option<String>) -> Result<CallToolResult, McpError> {
    let lt = query_type.as_deref().and_then(|s| match s.to_lowercase().as_str() {
        "equity" | "stock" => Some(LookupType::Equity),
        "etf" => Some(LookupType::Etf),
        "mutualfund" | "mutual_fund" | "mutual-fund" => Some(LookupType::MutualFund),
        "index" => Some(LookupType::Index),
        "future" => Some(LookupType::Future),
        "currency" | "forex" | "fx" => Some(LookupType::Currency),
        "crypto" | "cryptocurrency" => Some(LookupType::Cryptocurrency),
        _ => None,
    });
    let opts = match lt {
        Some(t) => LookupOptions::new().lookup_type(t),
        None => LookupOptions::new(),
    };
    let results = finance::lookup(&query, &opts).await.map_err(finance_err)?;
    let json = serde_json::to_string(&results).map_err(ser_err)?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(json)]))
}
