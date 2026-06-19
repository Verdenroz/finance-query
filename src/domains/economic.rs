//! Macro-economic indicator query handle.
//!
//! Created via [`Providers::economic`](crate::Providers::economic).

use crate::error::Result;

domain_handle! {
    /// A macro-economic data series backed by configured data providers.
    ///
    /// Created via [`Providers::economic`](crate::Providers::economic).
    pub struct EconomicIndicator { series_id, series_id }
    cache: crate::models::economic::EconomicSeries
}

impl EconomicIndicator {
    /// Fetch the full data series for this economic indicator.
    pub async fn series(&self) -> Result<crate::models::economic::EconomicSeries> {
        fetch_via!(
            self,
            series_id,
            ECONOMIC,
            fetch_economic_series,
            crate::models::economic::EconomicSeries
        )
    }
}
