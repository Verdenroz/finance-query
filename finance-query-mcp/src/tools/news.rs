use finance_query::{finance, Ticker};
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::{finance_err, ser_err};

pub async fn get_news(symbol: Option<String>) -> Result<CallToolResult, McpError> {
    let json = match symbol {
        Some(sym) => {
            let ticker = Ticker::new(&sym).await.map_err(finance_err)?;
            let news = ticker.news().await.map_err(finance_err)?;
            serde_json::to_string(&news).map_err(ser_err)?
        }
        None => {
            let news = finance::news().await.map_err(finance_err)?;
            serde_json::to_string(&news).map_err(ser_err)?
        }
    };
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(json)]))
}
