//! FRED (Federal Reserve Economic Data) provider implementation.

use super::{EconomicProvider, ProviderAdapter, ProviderCore};
use crate::error::Result;

pub(crate) struct FredProvider;

impl ProviderCore for FredProvider {
    fn id(&self) -> super::Provider {
        super::Provider::Fred
    }
}

#[async_trait::async_trait]
impl EconomicProvider for FredProvider {
    async fn fetch_economic_series(
        &self,
        series_id: &str,
    ) -> Result<crate::models::economic::EconomicSeries> {
        crate::adapters::fred::fetch_economic_series_response(series_id).await
    }
}

#[async_trait::async_trait]
impl ProviderAdapter for FredProvider {
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

    fn as_economic(&self) -> Option<&dyn EconomicProvider> {
        Some(self)
    }
}
