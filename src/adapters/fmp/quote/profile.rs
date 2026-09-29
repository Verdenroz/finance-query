use crate::{CompanyProfile, FinanceError, Provider, Result};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Profile {
    symbol: String,
    isin: Option<String>,
    cusip: Option<String>,
    is_actively_trading: Option<bool>,
    is_etf: Option<bool>,
    is_adr: Option<bool>,
    is_fund: Option<bool>,
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
    into_profile(row)
}

pub(crate) async fn fetch_company_profiles_bulk(part: u32) -> Result<Vec<CompanyProfile>> {
    let client = crate::adapters::fmp::build_client()?;
    let bytes = client
        .get_bytes("/stable/profile-bulk", &[("part", &part.to_string())])
        .await?;
    tokio::task::spawn_blocking(move || parse_bulk(&bytes))
        .await
        .map_err(|_| FinanceError::ResponseStructureError {
            field: "profile_bulk".into(),
            context: "FMP bulk-profile parser did not finish".into(),
        })?
}

fn parse_bulk(bytes: &[u8]) -> Result<Vec<CompanyProfile>> {
    let bytes = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(bytes);
    let first = bytes
        .iter()
        .copied()
        .find(|byte| !byte.is_ascii_whitespace());
    if first.is_none() {
        return Err(FinanceError::ResponseStructureError {
            field: "profile_bulk".into(),
            context: "empty FMP bulk-profile response".into(),
        });
    }
    if first == Some(b'[') {
        let rows: Vec<Profile> =
            serde_json::from_slice(bytes).map_err(|_| FinanceError::ResponseStructureError {
                field: "profile_bulk".into(),
                context: "invalid FMP bulk-profile JSON".into(),
            })?;
        return rows.into_iter().map(into_profile).collect();
    }
    let mut reader = csv::ReaderBuilder::new().from_reader(bytes);
    let header = reader
        .headers()
        .map_err(|_| FinanceError::ResponseStructureError {
            field: "profile_bulk".into(),
            context: "invalid FMP bulk-profile CSV header".into(),
        })?;
    if !header.iter().any(|name| name == "symbol") {
        return Err(FinanceError::ResponseStructureError {
            field: "symbol".into(),
            context: "FMP bulk-profile CSV has no symbol column".into(),
        });
    }
    reader
        .deserialize::<Profile>()
        .enumerate()
        .map(|(index, row)| {
            let row = row.map_err(|_| FinanceError::ResponseStructureError {
                field: "profile_bulk".into(),
                context: format!("invalid FMP bulk-profile CSV record {}", index + 1),
            })?;
            into_profile(row)
        })
        .collect()
}

fn into_profile(row: Profile) -> Result<CompanyProfile> {
    if row.symbol.trim().is_empty() {
        return Err(FinanceError::ResponseStructureError {
            field: "symbol".into(),
            context: "FMP profile has a blank symbol".into(),
        });
    }
    if let Some(date) = &row.ipo_date {
        crate::models::discovery::listings::date(date).map_err(|_| {
            FinanceError::ResponseStructureError {
                field: "ipoDate".into(),
                context: "invalid FMP IPO date".into(),
            }
        })?;
    }
    Ok(CompanyProfile {
        isin: row.isin,
        cusip: row.cusip,
        active: row.is_actively_trading,
        is_etf: row.is_etf,
        is_adr: row.is_adr,
        is_fund: row.is_fund,
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
