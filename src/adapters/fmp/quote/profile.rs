use crate::{CompanyProfile, FinanceError, Provider, Result};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Profile {
    symbol: String,
    cik: Option<String>,
    ipo_date: Option<String>,
    company_name: Option<String>,
    description: Option<String>,
    exchange: Option<String>,
    currency: Option<String>,
    country: Option<String>,
    sector: Option<String>,
    industry: Option<String>,
    market_cap: Option<f64>,
}

pub(crate) async fn fetch_company_profile(symbol: &str) -> Result<CompanyProfile> {
    let client = crate::adapters::fmp::build_client()?;
    let rows: Vec<Profile> = client.get("/stable/profile", &[("symbol", symbol)]).await?;
    if rows.is_empty() {
        return Err(FinanceError::SymbolNotFound {
            symbol: Some(symbol.into()),
            context: "FMP profile is empty".into(),
        });
    }
    if rows.len() != 1 || rows[0].symbol != symbol {
        return Err(FinanceError::ResponseStructureError {
            field: "symbol".into(),
            context: "FMP profile is ambiguous or mismatched".into(),
        });
    }
    let row = rows
        .into_iter()
        .next()
        .ok_or_else(|| FinanceError::ResponseStructureError {
            field: "profile".into(),
            context: "FMP profile is absent".into(),
        })?;
    if let Some(date) = &row.ipo_date {
        crate::models::discovery::listings::date(date).map_err(|_| {
            FinanceError::ResponseStructureError {
                field: "ipoDate".into(),
                context: "invalid FMP IPO date".into(),
            }
        })?;
    }
    Ok(CompanyProfile {
        symbol: Some(row.symbol),
        cik: row.cik,
        ipo_date: row.ipo_date,
        provider_id: Some(Provider::Fmp),
        name: row.company_name,
        description: row.description,
        exchange: row.exchange,
        currency: row.currency,
        country: row.country,
        sector: row.sector,
        industry: row.industry,
        market_capitalization: row.market_cap,
        asset_type: None,
    })
}
