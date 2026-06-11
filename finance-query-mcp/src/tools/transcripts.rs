use finance_query::finance;
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::{finance_err, ser_err};

pub async fn get_transcripts(
    symbol: String,
    limit: Option<u32>,
    lang: Option<String>,
) -> Result<CallToolResult, McpError> {
    let lim = limit.map(|n| n as usize);
    let mut transcripts = finance::earnings_transcripts(&symbol, lim)
        .await
        .map_err(finance_err)?;
    crate::lang::translate(&mut transcripts, lang.as_deref())
        .await
        .map_err(finance_err)?;
    let json = serde_json::to_string(&transcripts).map_err(ser_err)?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        json,
    )]))
}
