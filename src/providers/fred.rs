//! FRED (Federal Reserve Economic Data) provider implementation.

use crate::error::Result;

pub(crate) struct FredProvider;

#[async_trait::async_trait]
impl super::ProviderAdapter for FredProvider {
    fn id(&self) -> &'static str {
        "fred"
    }

    fn capabilities(&self) -> super::Capability {
        super::Capability::ECONOMIC
    }

    async fn initialize(&self) -> crate::error::Result<()> {
        let key = std::env::var("FRED_API_KEY").map_err(|_| {
            crate::error::FinanceError::InvalidParameter {
                param: "fred".into(),
                reason:
                    "FRED_API_KEY not set. Set the environment variable or call fred::init(key)."
                        .into(),
            }
        })?;
        // Init the FRED singleton from the env var so users don't need
        // a separate fred::init() call. Ignore "already initialized" errors.
        let _ = crate::adapters::fred::init(key);
        Ok(())
    }

    async fn fetch_economic_series(
        &self,
        series_id: &str,
    ) -> Result<crate::models::economic::EconomicSeries> {
        crate::adapters::fred::fetch_economic_series_response(series_id).await
    }
}
