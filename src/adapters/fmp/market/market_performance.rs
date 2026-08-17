//! Market performance endpoints: sector/industry PE, sector performance, gainers/losers/actives.

use futures::future::try_join_all;
use serde::{Deserialize, Serialize};

use crate::error::Result;

use crate::adapters::fmp::build_client;

// ============================================================================
// Response types
// ============================================================================

/// Sector price-to-earnings ratio entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SectorPeDTO {
    /// Date.
    pub date: Option<String>,
    /// Sector name.
    pub sector: Option<String>,
    /// Exchange.
    pub exchange: Option<String>,
    /// PE ratio.
    pub pe: Option<f64>,
}

/// Industry price-to-earnings ratio entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct IndustryPeDTO {
    /// Date.
    pub date: Option<String>,
    /// Industry name.
    pub industry: Option<String>,
    /// Exchange.
    pub exchange: Option<String>,
    /// PE ratio.
    pub pe: Option<f64>,
}

/// Sector performance entry. One row per (sector, exchange) pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SectorPerformanceDTO {
    /// Date.
    pub date: Option<String>,
    /// Sector name.
    pub sector: Option<String>,
    /// Exchange the average was computed over.
    pub exchange: Option<String>,
    /// Average percentage change across the sector's constituents.
    #[serde(rename = "averageChange")]
    pub average_change: Option<f64>,
}

/// Market mover entry (gainers, losers, most active).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct MarketMoverDTO {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Company name.
    pub name: Option<String>,
    /// Price change.
    pub change: Option<f64>,
    /// Price.
    pub price: Option<f64>,
    /// Change percentage.
    #[serde(rename = "changesPercentage")]
    pub changes_percentage: Option<f64>,
    /// Exchange the symbol trades on.
    pub exchange: Option<String>,
}

const SECTORS: [&str; 11] = [
    "Utilities",
    "Basic Materials",
    "Communication Services",
    "Consumer Cyclical",
    "Consumer Defensive",
    "Energy",
    "Financial Services",
    "Healthcare",
    "Industrials",
    "Real Estate",
    "Technology",
];

const SNAPSHOT_LOOKBACK_DAYS: usize = 5;

// ============================================================================
// Public API
// ============================================================================

/// Fetch sector PE ratios.
pub async fn sectors_pe() -> Result<Vec<SectorPeDTO>> {
    latest_snapshot("/stable/sector-pe-snapshot").await
}

/// Fetch industry PE ratios.
pub async fn industries_pe() -> Result<Vec<IndustryPeDTO>> {
    latest_snapshot("/stable/industry-pe-snapshot").await
}

/// Fetch current sector performance.
pub async fn sector_performance() -> Result<Vec<SectorPerformanceDTO>> {
    latest_snapshot("/stable/sector-performance-snapshot").await
}

/// Fetch historical sector performance over `[from, to]`.
///
/// The upstream endpoint takes one sector per call, so this fans out across
/// [`SECTORS`] concurrently and returns the rows flat. Every row is keyed by
/// (date, sector, exchange) upstream, so callers must not collapse on date
/// alone.
pub async fn historical_sector_performance(
    from: &str,
    to: &str,
) -> Result<Vec<SectorPerformanceDTO>> {
    let client = build_client()?;
    let responses = try_join_all(SECTORS.iter().map(|sector| {
        let client = &client;
        async move {
            client
                .get::<Vec<SectorPerformanceDTO>>(
                    "/stable/historical-sector-performance",
                    &[("sector", sector), ("from", from), ("to", to)],
                )
                .await
        }
    }))
    .await?;

    Ok(responses.into_iter().flatten().collect())
}

/// Weekdays before today, most recent first.
fn recent_weekdays() -> impl Iterator<Item = String> {
    use chrono::Datelike;

    let mut date = chrono::Utc::now().date_naive();
    std::iter::from_fn(move || {
        loop {
            date = date.pred_opt()?;
            if date.weekday() != chrono::Weekday::Sat && date.weekday() != chrono::Weekday::Sun {
                return Some(date.format("%Y-%m-%d").to_string());
            }
        }
    })
}

/// Fetch a snapshot endpoint for the most recent date that has data.
///
/// These endpoints require an exact session date and answer with an empty
/// array for market holidays, which a weekday calculation alone cannot skip,
/// so walk back until one answers.
async fn latest_snapshot<T: serde::de::DeserializeOwned>(path: &str) -> Result<Vec<T>> {
    let client = build_client()?;
    for date in recent_weekdays().take(SNAPSHOT_LOOKBACK_DAYS) {
        let rows: Vec<T> = client.get(path, &[("date", &date)]).await?;
        if !rows.is_empty() {
            return Ok(rows);
        }
    }
    Ok(Vec::new())
}

/// Fetch top stock market gainers.
pub async fn stock_market_gainers() -> Result<Vec<MarketMoverDTO>> {
    let client = build_client()?;
    client.get("/stable/biggest-gainers", &[]).await
}

/// Fetch top stock market losers.
pub async fn stock_market_losers() -> Result<Vec<MarketMoverDTO>> {
    let client = build_client()?;
    client.get("/stable/biggest-losers", &[]).await
}

/// Fetch most active stocks.
pub async fn stock_market_most_active() -> Result<Vec<MarketMoverDTO>> {
    let client = build_client()?;
    client.get("/stable/most-actives", &[]).await
}

/// Fetch sector performance and map to provider-neutral entries.
pub async fn fetch_sector_performance_response()
-> Result<Vec<crate::models::market::performance::SectorPerformance>> {
    Ok(sector_performance()
        .await?
        .into_iter()
        .filter_map(to_sector_performance)
        .collect())
}

fn to_sector_performance(
    dto: SectorPerformanceDTO,
) -> Option<crate::models::market::performance::SectorPerformance> {
    Some(crate::models::market::performance::SectorPerformance {
        sector: dto.sector?,
        exchange: dto.exchange,
        change_percent: dto.average_change,
    })
}

/// Fetch market movers for `direction` and map to provider-neutral entries.
pub async fn fetch_market_movers_response(
    direction: crate::models::market::performance::MoverDirection,
) -> Result<Vec<crate::models::market::performance::MoverQuote>> {
    use crate::models::market::performance::{MoverDirection, MoverQuote};
    let movers = match direction {
        MoverDirection::Gainers => stock_market_gainers().await?,
        MoverDirection::Losers => stock_market_losers().await?,
        MoverDirection::MostActive => stock_market_most_active().await?,
    };
    Ok(movers
        .into_iter()
        .filter_map(|m| {
            Some(MoverQuote {
                symbol: m.symbol?,
                name: m.name,
                price: m.price,
                change: m.change,
                change_percent: m.changes_percentage,
                exchange: m.exchange,
            })
        })
        .collect())
}

/// Fetch sector price/earnings ratios and map to provider-neutral entries.
pub async fn fetch_sector_pe_response() -> Result<Vec<crate::models::market::performance::SectorPe>>
{
    use crate::models::market::performance::SectorPe;
    Ok(sectors_pe()
        .await?
        .into_iter()
        .filter_map(|s| {
            Some(SectorPe {
                sector: s.sector?,
                exchange: s.exchange,
                pe: s.pe,
                date: s.date,
            })
        })
        .collect())
}

/// Fetch industry price/earnings ratios and map to provider-neutral entries.
pub async fn fetch_industry_pe_response()
-> Result<Vec<crate::models::market::performance::IndustryPe>> {
    use crate::models::market::performance::IndustryPe;
    Ok(industries_pe()
        .await?
        .into_iter()
        .filter_map(|s| {
            Some(IndustryPe {
                industry: s.industry?,
                exchange: s.exchange,
                pe: s.pe,
                date: s.date,
            })
        })
        .collect())
}

/// Group flat rows into one entry per date, most recent first.
///
/// Upstream rows are keyed by (date, sector, exchange), so grouping keeps every
/// exchange's row for a sector rather than letting one overwrite another.
fn group_history_by_date(
    rows: Vec<SectorPerformanceDTO>,
    limit: usize,
) -> Vec<crate::models::market::performance::SectorPerformanceHistory> {
    use crate::models::market::performance::{SectorPerformance, SectorPerformanceHistory};

    let mut by_date = std::collections::BTreeMap::<String, Vec<SectorPerformance>>::new();
    for row in rows {
        let Some(date) = row.date.clone() else {
            continue;
        };
        let Some(sector) = to_sector_performance(row) else {
            continue;
        };
        by_date.entry(date).or_default().push(sector);
    }

    by_date
        .into_iter()
        .rev()
        .take(limit)
        .map(|(date, sectors)| SectorPerformanceHistory {
            date: Some(date),
            sectors,
        })
        .collect()
}

/// Fetch canonical historical sector performance for the most recent `limit`
/// sessions, most recent first.
pub async fn fetch_sector_performance_history_response(
    limit: u32,
) -> Result<Vec<crate::models::market::performance::SectorPerformanceHistory>> {
    // The endpoint has no `limit`; ask for a window wide enough to contain
    // `limit` sessions and trim after grouping.
    let today = chrono::Utc::now().date_naive();
    let span = chrono::Days::new(u64::from(limit).saturating_mul(2).max(7));
    let from = today
        .checked_sub_days(span)
        .unwrap_or(today)
        .format("%Y-%m-%d")
        .to_string();
    let to = today.format("%Y-%m-%d").to_string();

    let rows = historical_sector_performance(&from, &to).await?;
    Ok(group_history_by_date(rows, limit as usize))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn history_rows() -> Vec<SectorPerformanceDTO> {
        serde_json::from_value(serde_json::json!([
            {"date": "2026-07-31", "sector": "Technology", "exchange": "NASDAQ", "averageChange": -1.25},
            {"date": "2026-07-31", "sector": "Technology", "exchange": "NYSE", "averageChange": -0.80},
            {"date": "2026-07-31", "sector": "Technology", "exchange": "AMEX", "averageChange": 0.10},
            {"date": "2026-07-30", "sector": "Technology", "exchange": "NASDAQ", "averageChange": 0.50}
        ]))
        .unwrap()
    }

    #[test]
    fn history_keeps_every_exchange_for_a_date() {
        let days = group_history_by_date(history_rows(), 10);

        assert_eq!(days.len(), 2);
        assert_eq!(days[0].date.as_deref(), Some("2026-07-31"));
        assert_eq!(days[0].sectors.len(), 3);

        let mut exchanges: Vec<_> = days[0]
            .sectors
            .iter()
            .map(|s| s.exchange.as_deref().unwrap())
            .collect();
        exchanges.sort_unstable();
        assert_eq!(exchanges, ["AMEX", "NASDAQ", "NYSE"]);

        let nasdaq = days[0]
            .sectors
            .iter()
            .find(|s| s.exchange.as_deref() == Some("NASDAQ"))
            .unwrap();
        assert_eq!(nasdaq.change_percent, Some(-1.25));
    }

    #[test]
    fn history_limit_counts_dates() {
        let days = group_history_by_date(history_rows(), 1);
        assert_eq!(days.len(), 1);
        assert_eq!(days[0].date.as_deref(), Some("2026-07-31"));
        assert_eq!(days[0].sectors.len(), 3);
    }

    #[tokio::test]
    async fn sector_performance_snapshot_keeps_rows_distinguishable() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/stable/sector-performance-snapshot")
            .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
                "apikey".into(),
                "test-key".into(),
            )]))
            .with_status(200)
            .with_body(
                r#"[
                    {"date":"2026-07-31","sector":"Technology","exchange":"NASDAQ","averageChange":1.25},
                    {"date":"2026-07-31","sector":"Technology","exchange":"NYSE","averageChange":0.62},
                    {"date":"2026-07-31","sector":"Healthcare","exchange":"NASDAQ","averageChange":-0.45}
                ]"#,
            )
            .create_async()
            .await;

        let client = crate::adapters::fmp::build_test_client(&server.url()).unwrap();
        let resp: Vec<SectorPerformanceDTO> = client
            .get("/stable/sector-performance-snapshot", &[])
            .await
            .unwrap();

        let canonical: Vec<_> = resp.into_iter().filter_map(to_sector_performance).collect();
        assert_eq!(canonical.len(), 3);
        assert_eq!(canonical[0].sector, "Technology");
        assert_eq!(canonical[0].exchange.as_deref(), Some("NASDAQ"));
        assert_eq!(canonical[0].change_percent, Some(1.25));
        assert_eq!(canonical[1].exchange.as_deref(), Some("NYSE"));
        assert_eq!(canonical[1].change_percent, Some(0.62));
    }

    #[tokio::test]
    async fn test_stock_market_gainers_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/stable/biggest-gainers")
            .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
                "apikey".into(),
                "test-key".into(),
            )]))
            .with_status(200)
            .with_body(
                r#"[{
                    "symbol": "XYZ",
                    "name": "XYZ Corp",
                    "change": 5.20,
                    "price": 42.50,
                    "changesPercentage": 13.93,
                    "exchange": "NASDAQ"
                }]"#,
            )
            .create_async()
            .await;

        let client = crate::adapters::fmp::build_test_client(&server.url()).unwrap();
        let resp: Vec<MarketMoverDTO> = client.get("/stable/biggest-gainers", &[]).await.unwrap();

        let row = &resp[0];
        assert_eq!(row.symbol.as_deref(), Some("XYZ"));
        assert_eq!(row.name.as_deref(), Some("XYZ Corp"));
        assert_eq!(row.change, Some(5.20));
        assert_eq!(row.price, Some(42.50));
        assert_eq!(row.changes_percentage, Some(13.93));
        assert_eq!(row.exchange.as_deref(), Some("NASDAQ"));
    }
}
