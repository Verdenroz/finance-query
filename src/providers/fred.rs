//! FRED (Federal Reserve Economic Data) provider implementation.

use super::types;
use crate::error::Result;

pub(crate) struct FredProvider;

#[async_trait::async_trait]
impl super::ProviderAdapter for FredProvider {
    fn id(&self) -> &'static str {
        "fred"
    }

    fn name(&self) -> &'static str {
        "FRED"
    }

    fn capabilities(&self) -> super::Capability {
        super::Capability::ECONOMIC
    }

    async fn initialize(&self) -> crate::error::Result<()> {
        if std::env::var("FRED_API_KEY").is_err() {
            return Err(crate::error::FinanceError::InvalidParameter {
                param: "fred".into(),
                reason:
                    "FRED_API_KEY not set. Set the environment variable or call fred::init(key)."
                        .into(),
            });
        }
        Ok(())
    }

    async fn fetch_economic_series(&self, series_id: &str) -> Result<types::EconomicSeriesData> {
        let series = crate::adapters::fred::series(series_id).await?;
        Ok(types::EconomicSeriesData {
            provider_id: "fred",
            series_id: series.id,
            title: None,
            units: None,
            frequency: None,
            seasonal_adjustment: None,
            observations: series
                .observations
                .into_iter()
                .map(|o| types::EconomicObservationData {
                    date: o.date,
                    value: o.value.map(|v| v.to_string()).unwrap_or_default(),
                })
                .collect(),
            extras: Default::default(),
        })
    }

    async fn fetch_treasury_yields(&self, year: u32) -> Result<types::TreasuryYieldData> {
        let yields = crate::adapters::fred::treasury_yields(year).await?;
        let latest = yields.into_iter().next_back().unwrap_or_else(|| {
            crate::adapters::fred::TreasuryYield {
                date: "01/01/1970".into(),
                y1m: None,
                y2m: None,
                y3m: None,
                y4m: None,
                y6m: None,
                y1: None,
                y2: None,
                y3: None,
                y5: None,
                y7: None,
                y10: None,
                y20: None,
                y30: None,
            }
        });

        Ok(types::TreasuryYieldData {
            provider_id: "fred",
            date: latest.date,
            month_1: latest.y1m,
            month_3: latest.y3m,
            month_6: latest.y6m,
            year_1: latest.y1,
            year_2: latest.y2,
            year_3: latest.y3,
            year_5: latest.y5,
            year_7: latest.y7,
            year_10: latest.y10,
            year_20: latest.y20,
            year_30: latest.y30,
            extras: Default::default(),
        })
    }
}
