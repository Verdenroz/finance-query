use finance_query::Ticker;
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::{finance_err, invalid_params, ser_err};

pub async fn get_holders(symbol: String, holder_type: String) -> Result<CallToolResult, McpError> {
    let ticker = Ticker::new(&symbol).await.map_err(finance_err)?;
    let json_val: serde_json::Value = match holder_type.to_lowercase().replace('-', "").as_str() {
        "major" => {
            let d = ticker.major_holders().await.map_err(finance_err)?;
            serde_json::to_value(d).map_err(ser_err)?
        }
        "institutional" => {
            let d = ticker.institution_ownership().await.map_err(finance_err)?;
            serde_json::to_value(d).map_err(ser_err)?
        }
        "mutualfund" => {
            let d = ticker.fund_ownership().await.map_err(finance_err)?;
            serde_json::to_value(d).map_err(ser_err)?
        }
        "insidertransactions" => {
            let d = ticker.insider_transactions().await.map_err(finance_err)?;
            serde_json::to_value(d).map_err(ser_err)?
        }
        "insiderpurchases" => {
            let d = ticker.share_purchase_activity().await.map_err(finance_err)?;
            serde_json::to_value(d).map_err(ser_err)?
        }
        "insiderroster" => {
            let d = ticker.insider_holders().await.map_err(finance_err)?;
            serde_json::to_value(d).map_err(ser_err)?
        }
        _ => {
            return Err(invalid_params(format!(
                "Invalid holder_type '{}'. Valid: major, institutional, mutualfund, insider-transactions, insider-purchases, insider-roster",
                holder_type
            )))
        }
    };
    let json = serde_json::to_string(&json_val).map_err(ser_err)?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(json)]))
}

pub async fn get_analysis(symbol: String, analysis_type: String) -> Result<CallToolResult, McpError> {
    let ticker = Ticker::new(&symbol).await.map_err(finance_err)?;
    let json_val: serde_json::Value = match analysis_type.to_lowercase().replace('-', "").as_str() {
        "recommendations" => {
            let d = ticker.recommendation_trend().await.map_err(finance_err)?;
            serde_json::to_value(d).map_err(ser_err)?
        }
        "upgradesdowngrades" => {
            let d = ticker.grading_history().await.map_err(finance_err)?;
            serde_json::to_value(d).map_err(ser_err)?
        }
        "earningsestimate" => {
            let d = ticker.earnings_trend().await.map_err(finance_err)?;
            serde_json::to_value(d).map_err(ser_err)?
        }
        "earningshistory" => {
            let d = ticker.earnings_history().await.map_err(finance_err)?;
            serde_json::to_value(d).map_err(ser_err)?
        }
        _ => {
            return Err(invalid_params(format!(
                "Invalid analysis_type '{}'. Valid: recommendations, upgrades-downgrades, earnings-estimate, earnings-history",
                analysis_type
            )))
        }
    };
    let json = serde_json::to_string(&json_val).map_err(ser_err)?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(json)]))
}
