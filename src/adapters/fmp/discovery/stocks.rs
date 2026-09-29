use crate::{FinanceError, Result, SymbolMatch};
use serde::Deserialize;

#[derive(Deserialize)]
struct DirectoryRow {
    symbol: String,
    #[serde(alias = "companyName")]
    name: Option<String>,
    exchange: Option<String>,
    #[serde(rename = "type")]
    stock_type: Option<String>,
}

pub(crate) async fn fetch_stock_list(active_only: bool) -> Result<Vec<SymbolMatch>> {
    let client = super::super::build_client()?;
    let path = if active_only {
        "/stable/actively-trading-list"
    } else {
        "/stable/stock-list"
    };
    let rows: Vec<DirectoryRow> = client.get(path, &[]).await?;
    rows.into_iter()
        .map(|row| {
            if row.symbol.trim().is_empty() {
                return Err(FinanceError::ResponseStructureError {
                    field: "symbol".into(),
                    context: "FMP directory contains a blank symbol".into(),
                });
            }
            Ok(SymbolMatch {
                symbol: row.symbol,
                name: row.name,
                exchange: row.exchange,
                asset_type: row.stock_type,
                active: active_only.then_some(true),
                ..Default::default()
            })
        })
        .collect()
}
