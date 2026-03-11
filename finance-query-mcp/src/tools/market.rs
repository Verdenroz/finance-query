use finance_query::{IndicesRegion, Region, Sector, finance};
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::{finance_err, ser_err};

fn parse_region(s: &str) -> Region {
    match s.to_uppercase().as_str() {
        "US" => Region::UnitedStates,
        "GB" | "UK" => Region::UnitedKingdom,
        "DE" => Region::Germany,
        "CA" => Region::Canada,
        "AU" => Region::Australia,
        "FR" => Region::France,
        "IN" => Region::India,
        "CN" => Region::China,
        "HK" => Region::HongKong,
        "BR" => Region::Brazil,
        "TW" => Region::Taiwan,
        "SG" => Region::Singapore,
        "IT" => Region::Italy,
        "ES" => Region::Spain,
        _ => Region::UnitedStates,
    }
}

pub async fn get_market_summary(region: Option<String>) -> Result<CallToolResult, McpError> {
    let r = parse_region(region.as_deref().unwrap_or("US"));
    let summary = finance::market_summary(Some(r)).await.map_err(finance_err)?;
    let json = serde_json::to_string(&summary).map_err(ser_err)?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(json)]))
}

pub async fn get_fear_and_greed() -> Result<CallToolResult, McpError> {
    let fng = finance::fear_and_greed().await.map_err(finance_err)?;
    let json = serde_json::to_string(&fng).map_err(ser_err)?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(json)]))
}

pub async fn get_trending(region: Option<String>) -> Result<CallToolResult, McpError> {
    let r = region.as_deref().map(parse_region);
    let trending = finance::trending(r).await.map_err(finance_err)?;
    let json = serde_json::to_string(&trending).map_err(ser_err)?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(json)]))
}

pub async fn get_indices(region: Option<String>) -> Result<CallToolResult, McpError> {
    let r = region.as_deref().and_then(|s| s.parse::<IndicesRegion>().ok());
    let quotes = finance::indices(r).await.map_err(finance_err)?;
    let json = serde_json::to_string(&quotes).map_err(ser_err)?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(json)]))
}

pub async fn get_market_hours(region: Option<String>) -> Result<CallToolResult, McpError> {
    let hours = finance::hours(region.as_deref()).await.map_err(finance_err)?;
    let json = serde_json::to_string(&hours).map_err(ser_err)?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(json)]))
}

pub async fn get_sector(sector: String) -> Result<CallToolResult, McpError> {
    let s: Sector = sector.parse().map_err(|_| {
        crate::error::invalid_params(format!(
            "Invalid sector '{}'. Valid: technology, financial-services, consumer-cyclical, \
             communication-services, healthcare, industrials, consumer-defensive, energy, \
             basic-materials, real-estate, utilities",
            sector
        ))
    })?;
    let data = finance::sector(s).await.map_err(finance_err)?;
    let json = serde_json::to_string(&data).map_err(ser_err)?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(json)]))
}

pub async fn get_industry(industry: String) -> Result<CallToolResult, McpError> {
    let data = finance::industry(&industry).await.map_err(finance_err)?;
    let json = serde_json::to_string(&data).map_err(ser_err)?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(json)]))
}
