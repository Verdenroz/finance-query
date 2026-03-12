use crate::cache::{self, Cache};
use finance_query::Ticker;

use super::{ServiceError, ServiceResult};

/// Analysis type identifiers matching the REST path param.
#[derive(Debug, Clone, Copy)]
pub enum AnalysisType {
    Recommendations,
    UpgradesDowngrades,
    EarningsEstimate,
    EarningsHistory,
}

impl AnalysisType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "recommendations" => Some(Self::Recommendations),
            "upgrades-downgrades" => Some(Self::UpgradesDowngrades),
            "earnings-estimate" => Some(Self::EarningsEstimate),
            "earnings-history" => Some(Self::EarningsHistory),
            _ => None,
        }
    }

    pub fn valid_types() -> &'static str {
        "recommendations, upgrades-downgrades, earnings-estimate, earnings-history"
    }
}

pub async fn get_analysis(
    cache: &Cache,
    symbol: &str,
    analysis_type: AnalysisType,
    analysis_type_str: &str,
) -> ServiceResult {
    let cache_key = Cache::key("analysis", &[&symbol.to_uppercase(), analysis_type_str]);
    let symbol = symbol.to_string();

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::ANALYSIS,
            cache::is_market_open(),
            || async move {
                let ticker = Ticker::new(&symbol).await?;
                let json: serde_json::Value = match analysis_type {
                    AnalysisType::Recommendations => {
                        let data = ticker.recommendation_trend().await?;
                        serde_json::to_value(data).map_err(|e| Box::new(e) as ServiceError)?
                    }
                    AnalysisType::UpgradesDowngrades => {
                        let data = ticker.grading_history().await?;
                        serde_json::to_value(data).map_err(|e| Box::new(e) as ServiceError)?
                    }
                    AnalysisType::EarningsEstimate => {
                        let data = ticker.earnings_trend().await?;
                        serde_json::to_value(data).map_err(|e| Box::new(e) as ServiceError)?
                    }
                    AnalysisType::EarningsHistory => {
                        let data = ticker.earnings_history().await?;
                        serde_json::to_value(data).map_err(|e| Box::new(e) as ServiceError)?
                    }
                };
                Ok(json)
            },
        )
        .await
}

pub async fn get_recommendations(cache: &Cache, symbol: &str, limit: u32) -> ServiceResult {
    let cache_key = Cache::key(
        "recommendations",
        &[&symbol.to_uppercase(), &limit.to_string()],
    );
    let symbol = symbol.to_string();

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::ANALYSIS,
            cache::is_market_open(),
            || async move {
                let ticker = Ticker::new(&symbol).await?;
                let recommendation = ticker.recommendations(limit).await?;
                serde_json::to_value(&recommendation).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}

pub async fn get_batch_recommendations(
    cache: &Cache,
    symbols: Vec<&str>,
    limit: u32,
) -> ServiceResult {
    let mut symbols = symbols;
    symbols.sort();
    let symbols_key = symbols.join(",").to_uppercase();
    let cache_key = Cache::key("recommendations", &[&symbols_key, &limit.to_string()]);

    cache
        .get_or_fetch(&cache_key, cache::ttl::ANALYSIS, false, || async move {
            let tickers = finance_query::Tickers::new(symbols).await?;
            let batch_response = tickers.recommendations(limit).await?;
            tracing::info!(
                "Recommendations fetch complete: {} success, {} errors",
                batch_response.success_count(),
                batch_response.error_count()
            );
            serde_json::to_value(&batch_response).map_err(|e| Box::new(e) as ServiceError)
        })
        .await
}
