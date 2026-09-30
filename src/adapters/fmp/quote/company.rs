//! FMP company information endpoints.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use crate::error::{FinanceError, Result};
use crate::models::discovery::reference::SymbolMatch;

// ============================================================================
// Response types
// ============================================================================

/// Stock peer from FMP.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct StockPeersDTO {
    /// Ticker symbol of the peer.
    pub symbol: Option<String>,
    /// Peer company name.
    #[serde(rename = "companyName")]
    pub company_name: Option<String>,
}

/// Delisted company from FMP.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct DelistedCompanyDTO {
    /// Ticker symbol.
    #[serde(default, deserialize_with = "crate::adapters::fmp::blank_as_none")]
    pub symbol: Option<String>,
    /// Company name.
    #[serde(
        rename = "companyName",
        default,
        deserialize_with = "crate::adapters::fmp::blank_as_none"
    )]
    pub company_name: Option<String>,
    /// Exchange.
    #[serde(default, deserialize_with = "crate::adapters::fmp::blank_as_none")]
    pub exchange: Option<String>,
    /// IPO date.
    #[serde(
        rename = "ipoDate",
        default,
        deserialize_with = "crate::adapters::fmp::blank_as_none"
    )]
    pub ipo_date: Option<String>,
    /// Delisted date.
    #[serde(
        rename = "delistedDate",
        default,
        deserialize_with = "crate::adapters::fmp::blank_as_none"
    )]
    pub delisted_date: Option<String>,
}

// ============================================================================
// Query functions
// ============================================================================

/// Convert stock peers DTOs into canonical SimilarSymbol items.
fn stock_peers_to_canonical(
    peers: Vec<StockPeersDTO>,
    limit: usize,
) -> Vec<crate::models::corporate::recommendation::SimilarSymbol> {
    let mut symbols: Vec<crate::models::corporate::recommendation::SimilarSymbol> = peers
        .into_iter()
        .filter_map(|p| p.symbol)
        .map(
            |s| crate::models::corporate::recommendation::SimilarSymbol {
                symbol: s,
                score: 0.0,
            },
        )
        .collect();
    symbols.truncate(limit);
    symbols
}

/// Fetch canonical similar symbols for a ticker.
pub async fn fetch_canonical_similar_symbols(
    symbol: &str,
    limit: u32,
) -> Result<Vec<crate::models::corporate::recommendation::SimilarSymbol>> {
    let peers = stock_peers(symbol).await?;
    Ok(stock_peers_to_canonical(peers, limit as usize))
}

/// Fetch stock peers for a symbol.
pub async fn stock_peers(symbol: &str) -> Result<Vec<StockPeersDTO>> {
    let client = crate::adapters::fmp::build_client()?;
    client
        .get("/stable/stock-peers", &[("symbol", symbol)])
        .await
}

/// Fetch the first page of delisted companies. `limit` controls the page size.
pub async fn delisted_companies(limit: Option<u32>) -> Result<Vec<DelistedCompanyDTO>> {
    let limit = limit.unwrap_or(MAX_DELISTED_PAGE_SIZE);
    check_delisted_page(0, limit)?;
    let client = crate::adapters::fmp::build_client()?;
    delisted_rows(&client, 0, limit).await
}

#[derive(Default, Serialize, Deserialize)]
struct DelistedCursor {
    page: u32,
    seen: Vec<u64>,
    records: BTreeSet<u64>,
}

fn cursor_fingerprint(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325_u64, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
    })
}

const DELISTED_PAGE_LIMITS: crate::adapters::fmp::client::ResponseLimits =
    crate::adapters::fmp::client::ResponseLimits::bytes(8 * 1024 * 1024);
/// FMP returns at most 100 delisted rows per page, so larger sizes are rejected
/// rather than silently capped.
const MAX_DELISTED_PAGE_SIZE: u32 = 100;
const MAX_DELISTED_PAGES: u32 = 10_000;
const MAX_DELISTED_CURSOR_BYTES: usize = 16 * 1024 * 1024;

fn check_delisted_page(page: u32, limit: u32) -> Result<()> {
    if page >= MAX_DELISTED_PAGES || !(1..=MAX_DELISTED_PAGE_SIZE).contains(&limit) {
        return Err(FinanceError::InvalidParameter {
            param: "page/limit".into(),
            reason: "page must be below 10000 and limit 1..=100".into(),
        });
    }
    Ok(())
}

async fn delisted_rows(
    client: &crate::adapters::fmp::client::FmpClient,
    page: u32,
    limit: u32,
) -> Result<Vec<DelistedCompanyDTO>> {
    client
        .get_limited(
            "/stable/delisted-companies",
            &[("page", &page.to_string()), ("limit", &limit.to_string())],
            Some(DELISTED_PAGE_LIMITS),
        )
        .await
}

async fn delisted_page(
    client: &crate::adapters::fmp::client::FmpClient,
    limit: u32,
    cursor: Option<&crate::PageCursor>,
) -> Result<(Vec<DelistedCompanyDTO>, Option<crate::PageCursor>)> {
    let invalid = || FinanceError::InvalidParameter {
        param: "cursor".into(),
        reason: "invalid FMP delisted continuation".into(),
    };
    let mut state = match cursor {
        Some(cursor) if cursor.target().len() <= MAX_DELISTED_CURSOR_BYTES => {
            serde_json::from_str::<DelistedCursor>(cursor.target()).map_err(|_| invalid())?
        }
        Some(_) => return Err(invalid()),
        None => DelistedCursor::default(),
    };
    if state.page as usize != state.seen.len() {
        return Err(invalid());
    }
    check_delisted_page(state.page, limit)?;
    let rows = delisted_rows(client, state.page, limit).await?;
    if rows.is_empty() {
        return Ok((rows, None));
    }
    let mut ordered: Vec<String> = rows
        .iter()
        .map(serde_json::to_string)
        .collect::<std::result::Result<_, _>>()?;
    ordered.sort();
    ordered.dedup();
    let hash = cursor_fingerprint(ordered.join("\n").as_bytes());
    let mut advanced = false;
    for row in &ordered {
        advanced |= state.records.insert(cursor_fingerprint(row.as_bytes()));
    }
    if state.seen.contains(&hash) || !advanced {
        return Err(FinanceError::ResponseStructureError {
            field: "pagination".into(),
            context: "FMP repeated a delisted page".into(),
        });
    }
    state.seen.push(hash);
    state.page += 1;
    let target = serde_json::to_string(&state)?;
    if target.len() > MAX_DELISTED_CURSOR_BYTES {
        return Err(invalid());
    }
    Ok((
        rows,
        Some(crate::PageCursor::continuation(
            crate::Provider::Fmp,
            target,
        )),
    ))
}

pub(crate) async fn fetch_delisted_stocks_page(
    limit: u32,
    cursor: Option<&crate::PageCursor>,
) -> Result<crate::ProviderPage<SymbolMatch>> {
    let client = crate::adapters::fmp::build_client()?;
    let (rows, next) = delisted_page(&client, limit, cursor).await?;
    let count = rows.len();
    let items = delisted_symbols(rows)?;
    Ok(crate::ProviderPage {
        items,
        next,
        provider_id: crate::Provider::Fmp,
        request_id: None,
        results_count: Some(count),
        query_count: None,
        reported_symbol: None,
        adjusted: None,
    })
}

pub(crate) async fn fetch_delisted_stocks_page_at(
    page: u32,
    limit: u32,
) -> Result<Vec<SymbolMatch>> {
    check_delisted_page(page, limit)?;
    let client = crate::adapters::fmp::build_client()?;
    let rows = delisted_rows(&client, page, limit).await?;
    if rows.len() > limit as usize {
        return Err(FinanceError::ResponseStructureError {
            field: "pagination".into(),
            context: "FMP delisted page exceeds its requested limit".into(),
        });
    }
    delisted_symbols(rows)
}

fn delisted_symbols(rows: Vec<DelistedCompanyDTO>) -> Result<Vec<SymbolMatch>> {
    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        let item = to_delisted_symbol_match(row)
            .filter(|r| !r.symbol.trim().is_empty())
            .ok_or_else(|| FinanceError::ResponseStructureError {
                field: "symbol".into(),
                context: "FMP delisted row has no symbol".into(),
            })?;
        items.push(item);
    }
    Ok(items)
}

/// Convert a delisted-company record into a canonical [`SymbolMatch`],
/// dropping entries without a symbol.
fn to_delisted_symbol_match(dto: DelistedCompanyDTO) -> Option<SymbolMatch> {
    Some(SymbolMatch {
        symbol: dto.symbol?,
        id: None,
        name: dto.company_name,
        exchange: dto.exchange,
        asset_type: None,
        currency: None,
        active: Some(false),
        market_cap_rank: None,
        thumbnail: None,
        image: None,
        ipo_date: dto.ipo_date,
        delisted_date: dto.delisted_date,
    })
}

/// Fetch canonical delisted-security listing status
/// (`DiscoveryProvider::fetch_listing_status(active: false)`).
pub async fn fetch_delisted_listing_status_response() -> Result<Vec<SymbolMatch>> {
    Ok(delisted_companies(None)
        .await?
        .into_iter()
        .filter_map(to_delisted_symbol_match)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_delisted_company_to_symbol_match() {
        let dto: DelistedCompanyDTO = serde_json::from_value(serde_json::json!({
            "symbol": "XYZ",
            "companyName": "XYZ Corp",
            "exchange": "NYSE",
            "ipoDate": "2001-01-01",
            "delistedDate": "2023-06-01"
        }))
        .unwrap();

        let out = to_delisted_symbol_match(dto).unwrap();
        assert_eq!(out.symbol, "XYZ");
        assert_eq!(out.name.as_deref(), Some("XYZ Corp"));
        assert_eq!(out.exchange.as_deref(), Some("NYSE"));
        assert_eq!(out.active, Some(false));
        assert_eq!(out.ipo_date.as_deref(), Some("2001-01-01"));
        assert_eq!(out.delisted_date.as_deref(), Some("2023-06-01"));
    }

    #[test]
    fn drops_delisted_entries_without_a_symbol() {
        let dto: DelistedCompanyDTO = serde_json::from_value(serde_json::json!({
            "companyName": "No Symbol Inc"
        }))
        .unwrap();
        assert!(to_delisted_symbol_match(dto).is_none());
    }

    #[tokio::test]
    async fn test_fmp_rate_limit_returns_rate_limited_error() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(429)
            .with_body("{}")
            .create_async()
            .await;

        let client = crate::adapters::fmp::build_test_client(&server.url()).unwrap();
        let result = client.get_raw("/api/v3/profile/AAPL", &[]).await;

        assert!(matches!(
            result,
            Err(crate::error::FinanceError::RateLimited { .. })
        ));
    }

    #[tokio::test]
    async fn test_fmp_401_returns_authentication_failed() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(401)
            .with_body("{}")
            .create_async()
            .await;

        let client = crate::adapters::fmp::build_test_client(&server.url()).unwrap();
        let result = client.get_raw("/api/v3/profile/AAPL", &[]).await;

        assert!(matches!(
            result,
            Err(crate::error::FinanceError::AuthenticationFailed { .. })
        ));
    }

    #[tokio::test]
    async fn test_fmp_body_api_key_error_returns_authentication_failed() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(200)
            .with_body(r#"{"Error Message":"Invalid API KEY."}"#)
            .create_async()
            .await;

        let client = crate::adapters::fmp::build_test_client(&server.url()).unwrap();
        let result = client.get_raw("/api/v3/profile/AAPL", &[]).await;

        assert!(matches!(
            result,
            Err(crate::error::FinanceError::AuthenticationFailed { .. })
        ));
    }

    #[tokio::test]
    async fn test_fmp_500_returns_server_error() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(500)
            .with_body("{}")
            .create_async()
            .await;

        let client = crate::adapters::fmp::build_test_client(&server.url()).unwrap();
        let result = client.get_raw("/api/v3/profile/AAPL", &[]).await;

        assert!(matches!(
            result,
            Err(crate::error::FinanceError::ServerError { .. })
        ));
    }
}
