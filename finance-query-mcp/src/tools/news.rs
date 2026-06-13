use finance_query::finance;
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::{finance_err, ser_err};

pub async fn get_news(
    symbol: Option<String>,
    lang: Option<String>,
) -> Result<CallToolResult, McpError> {
    let json = match symbol {
        Some(sym) => {
            // The ticker path translates via the library; only the general
            // branch needs an explicit translate call.
            let ticker = crate::lang::ticker(&sym, lang.as_deref())
                .await
                .map_err(finance_err)?;
            let news = ticker.news().await.map_err(finance_err)?;
            serde_json::to_string(&news).map_err(ser_err)?
        }
        None => {
            let mut news = finance::news().await.map_err(finance_err)?;
            crate::lang::translate(&mut news, lang.as_deref())
                .await
                .map_err(finance_err)?;
            serde_json::to_string(&news).map_err(ser_err)?
        }
    };
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        json,
    )]))
}
