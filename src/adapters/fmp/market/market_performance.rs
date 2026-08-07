//! Market performance endpoints: sector/industry PE, sector performance, gainers/losers/actives.

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

/// Sector performance entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SectorPerformanceDTO {
    /// Sector name.
    pub sector: Option<String>,
    /// Changes percentage.
    #[serde(rename = "changesPercentage")]
    pub changes_percentage: Option<String>,
    /// Average percentage change returned by the stable snapshot endpoint.
    #[serde(rename = "averageChange")]
    pub average_change: Option<f64>,
}

/// Historical sector performance entry.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct HistoricalSectorPerformanceDTO {
    /// Date.
    pub date: Option<String>,
    /// Utilities sector performance.
    #[serde(rename = "utilitiesChangesPercentage")]
    pub utilities_changes_percentage: Option<f64>,
    /// Basic materials sector performance.
    #[serde(rename = "basicMaterialsChangesPercentage")]
    pub basic_materials_changes_percentage: Option<f64>,
    /// Communication services sector performance.
    #[serde(rename = "communicationServicesChangesPercentage")]
    pub communication_services_changes_percentage: Option<f64>,
    /// Consumer cyclical sector performance.
    #[serde(rename = "consumerCyclicalChangesPercentage")]
    pub consumer_cyclical_changes_percentage: Option<f64>,
    /// Consumer defensive sector performance.
    #[serde(rename = "consumerDefensiveChangesPercentage")]
    pub consumer_defensive_changes_percentage: Option<f64>,
    /// Energy sector performance.
    #[serde(rename = "energyChangesPercentage")]
    pub energy_changes_percentage: Option<f64>,
    /// Financial services sector performance.
    #[serde(rename = "financialServicesChangesPercentage")]
    pub financial_services_changes_percentage: Option<f64>,
    /// Healthcare sector performance.
    #[serde(rename = "healthcareChangesPercentage")]
    pub healthcare_changes_percentage: Option<f64>,
    /// Industrials sector performance.
    #[serde(rename = "industrialsChangesPercentage")]
    pub industrials_changes_percentage: Option<f64>,
    /// Real estate sector performance.
    #[serde(rename = "realEstateChangesPercentage")]
    pub real_estate_changes_percentage: Option<f64>,
    /// Technology sector performance.
    #[serde(rename = "technologyChangesPercentage")]
    pub technology_changes_percentage: Option<f64>,
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
    #[serde(rename = "changePercentage", alias = "changesPercentage")]
    pub changes_percentage: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct StableSectorHistoryDTO {
    date: Option<String>,
    #[serde(rename = "averageChange")]
    average_change: Option<f64>,
}

// ============================================================================
// Public API
// ============================================================================

/// Fetch sector PE ratios.
pub async fn sectors_pe() -> Result<Vec<SectorPeDTO>> {
    let client = build_client()?;
    let date = latest_completed_market_date();
    client
        .get("/stable/sector-pe-snapshot", &[("date", &date)])
        .await
}

/// Fetch industry PE ratios.
pub async fn industries_pe() -> Result<Vec<IndustryPeDTO>> {
    let client = build_client()?;
    let date = latest_completed_market_date();
    client
        .get("/stable/industry-pe-snapshot", &[("date", &date)])
        .await
}

/// Fetch current sector performance.
pub async fn sector_performance() -> Result<Vec<SectorPerformanceDTO>> {
    let client = build_client()?;
    let date = latest_completed_market_date();
    client
        .get("/stable/sector-performance-snapshot", &[("date", &date)])
        .await
}

/// Fetch historical sector performance.
pub async fn historical_sector_performance(
    limit: u32,
) -> Result<Vec<HistoricalSectorPerformanceDTO>> {
    let client = build_client()?;
    let mut by_date = std::collections::BTreeMap::<String, HistoricalSectorPerformanceDTO>::new();
    for sector in [
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
    ] {
        let rows: Vec<StableSectorHistoryDTO> = client
            .get(
                "/stable/historical-sector-performance",
                &[("sector", sector)],
            )
            .await?;
        for row in rows {
            let Some(date) = row.date else { continue };
            let entry = by_date.entry(date.clone()).or_default();
            entry.date = Some(date);
            match sector {
                "Utilities" => entry.utilities_changes_percentage = row.average_change,
                "Basic Materials" => entry.basic_materials_changes_percentage = row.average_change,
                "Communication Services" => {
                    entry.communication_services_changes_percentage = row.average_change;
                }
                "Consumer Cyclical" => {
                    entry.consumer_cyclical_changes_percentage = row.average_change
                }
                "Consumer Defensive" => {
                    entry.consumer_defensive_changes_percentage = row.average_change;
                }
                "Energy" => entry.energy_changes_percentage = row.average_change,
                "Financial Services" => {
                    entry.financial_services_changes_percentage = row.average_change;
                }
                "Healthcare" => entry.healthcare_changes_percentage = row.average_change,
                "Industrials" => entry.industrials_changes_percentage = row.average_change,
                "Real Estate" => entry.real_estate_changes_percentage = row.average_change,
                "Technology" => entry.technology_changes_percentage = row.average_change,
                _ => unreachable!(),
            }
        }
    }
    Ok(by_date.into_values().rev().take(limit as usize).collect())
}

fn latest_completed_market_date() -> String {
    use chrono::Datelike;

    let mut date = chrono::Utc::now().date_naive() - chrono::Days::new(1);
    while date.weekday() == chrono::Weekday::Sat || date.weekday() == chrono::Weekday::Sun {
        date = date.pred_opt().expect("valid previous calendar date");
    }
    date.format("%Y-%m-%d").to_string()
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
    use crate::models::market::performance::{SectorPerformance, parse_percent};
    Ok(sector_performance()
        .await?
        .into_iter()
        .filter_map(|s| {
            Some(SectorPerformance {
                sector: s.sector?,
                change_percent: parse_percent(s.changes_percentage.as_deref()).or(s.average_change),
            })
        })
        .collect())
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

/// Convert one historical row into the canonical per-day sector list.
fn history_to_canonical(
    d: HistoricalSectorPerformanceDTO,
) -> crate::models::market::performance::SectorPerformanceHistory {
    use crate::models::market::performance::SectorPerformance;
    let sector = |name: &str, v: Option<f64>| SectorPerformance {
        sector: name.to_string(),
        change_percent: v,
    };
    crate::models::market::performance::SectorPerformanceHistory {
        date: d.date,
        sectors: vec![
            sector("Utilities", d.utilities_changes_percentage),
            sector("Basic Materials", d.basic_materials_changes_percentage),
            sector(
                "Communication Services",
                d.communication_services_changes_percentage,
            ),
            sector("Consumer Cyclical", d.consumer_cyclical_changes_percentage),
            sector(
                "Consumer Defensive",
                d.consumer_defensive_changes_percentage,
            ),
            sector("Energy", d.energy_changes_percentage),
            sector(
                "Financial Services",
                d.financial_services_changes_percentage,
            ),
            sector("Healthcare", d.healthcare_changes_percentage),
            sector("Industrials", d.industrials_changes_percentage),
            sector("Real Estate", d.real_estate_changes_percentage),
            sector("Technology", d.technology_changes_percentage),
        ],
    }
}

/// Fetch canonical historical sector performance, most recent first.
pub async fn fetch_sector_performance_history_response(
    limit: u32,
) -> Result<Vec<crate::models::market::performance::SectorPerformanceHistory>> {
    let dtos = historical_sector_performance(limit).await?;
    Ok(dtos.into_iter().map(history_to_canonical).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn history_maps_all_eleven_sectors() {
        let row: HistoricalSectorPerformanceDTO = serde_json::from_value(serde_json::json!({
            "date": "2026-07-31",
            "utilitiesChangesPercentage": 0.5,
            "technologyChangesPercentage": -1.25
        }))
        .unwrap();
        let day = history_to_canonical(row);
        assert_eq!(day.date.as_deref(), Some("2026-07-31"));
        assert_eq!(day.sectors.len(), 11);
        let tech = day
            .sectors
            .iter()
            .find(|s| s.sector == "Technology")
            .unwrap();
        assert_eq!(tech.change_percent, Some(-1.25));
        let util = day
            .sectors
            .iter()
            .find(|s| s.sector == "Utilities")
            .unwrap();
        assert_eq!(util.change_percent, Some(0.5));
        // Absent sectors surface as None, not omitted.
        let energy = day.sectors.iter().find(|s| s.sector == "Energy").unwrap();
        assert!(energy.change_percent.is_none());
    }

    #[tokio::test]
    async fn test_sector_performance_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/stable/sector-performance-snapshot")
            .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
                "apikey".into(),
                "test-key".into(),
            )]))
            .with_status(200)
            .with_body(
                serde_json::json!([
                    {
                        "sector": "Technology",
                        "changesPercentage": "1.25%"
                    },
                    {
                        "sector": "Healthcare",
                        "changesPercentage": "-0.45%"
                    }
                ])
                .to_string(),
            )
            .create_async()
            .await;

        let client = crate::adapters::fmp::build_test_client(&server.url()).unwrap();
        let resp: Vec<SectorPerformanceDTO> = client
            .get("/stable/sector-performance-snapshot", &[])
            .await
            .unwrap();
        assert_eq!(resp.len(), 2);
        assert_eq!(resp[0].sector.as_deref(), Some("Technology"));
        assert_eq!(resp[0].changes_percentage.as_deref(), Some("1.25%"));
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
                serde_json::json!([
                    {
                        "symbol": "XYZ",
                        "name": "XYZ Corp",
                        "change": 5.20,
                        "price": 42.50,
                        "changesPercentage": 13.93
                    }
                ])
                .to_string(),
            )
            .create_async()
            .await;

        let client = crate::adapters::fmp::build_test_client(&server.url()).unwrap();
        let resp: Vec<MarketMoverDTO> = client.get("/stable/biggest-gainers", &[]).await.unwrap();
        assert_eq!(resp.len(), 1);
        assert_eq!(resp[0].symbol.as_deref(), Some("XYZ"));
        assert!((resp[0].change.unwrap() - 5.20).abs() < 0.01);
    }
}
