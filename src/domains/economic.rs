//! Macro-economic indicator query handle.
//!
//! Created via [`Providers::economic`](crate::Providers::economic).

use std::sync::Arc;

use crate::error::Result;
use crate::models::economic::{EconomicCategory, EconomicRelease, EconomicSeriesMatch};
use crate::providers::{Capability, Operation, ProviderSet};

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
            as_economic,
            EconomicSeries,
            fetch_economic_series,
            crate::models::economic::EconomicSeries
        )
    }

    /// Fetch this series as it stood on `date` (`YYYY-MM-DD`) instead of as
    /// currently revised (FRED only, via ALFRED's realtime window).
    ///
    /// Macro data is revised after publication, so backtesting a rule against
    /// today's series is look-ahead bias: the values it trades on were not
    /// knowable at the time. This returns the vintage actually published as of
    /// `date`. Cached per date.
    pub async fn as_of(&self, date: &str) -> Result<crate::models::economic::EconomicSeries> {
        let series_id: String = self.series_id().to_string();
        let date = date.to_string();
        let providers = Arc::clone(&self.providers);
        self.cache
            .get_or_try(format!("as_of\u{1f}{date}"), move || async move {
                providers
                    .fetch(Capability::ECONOMIC, move |p| {
                        let (series_id, date) = (series_id.clone(), date.clone());
                        let p = p.clone();
                        async move {
                            p.as_economic()
                                .ok_or_else(|| p.not_supported(Operation::EconomicSeriesAsOf))?
                                .fetch_economic_series_as_of(&series_id, &date)
                                .await
                        }
                    })
                    .await
            })
            .await
    }
}

/// The macro-economic series catalog: search and browse rather than fetch.
///
/// Routes through [`Capability::ECONOMIC`]. [`EconomicIndicator`] needs a
/// series id you already know; this handle is how you find one. FRED is
/// currently the only provider.
///
/// Created via [`Providers::economic_catalog`](crate::Providers::economic_catalog).
pub struct EconomicCatalog {
    providers: Arc<ProviderSet>,
}

impl EconomicCatalog {
    pub(crate) fn with_providers(providers: Arc<ProviderSet>) -> Self {
        Self { providers }
    }

    /// Search the series catalog by free text, most popular first.
    pub async fn search(&self, query: &str, limit: u32) -> Result<Vec<EconomicSeriesMatch>> {
        let query = query.to_string();
        dispatch_via!(
            self,
            ECONOMIC,
            as_economic,
            EconomicSearch,
            fetch_economic_search,
            [query],
            &query,
            limit
        )
    }

    /// List the child categories of `parent_id`. Pass `0` for the root.
    pub async fn categories(&self, parent_id: i64) -> Result<Vec<EconomicCategory>> {
        dispatch_via!(
            self,
            ECONOMIC,
            as_economic,
            EconomicCategories,
            fetch_economic_categories,
            [],
            parent_id
        )
    }

    /// List every scheduled data release the provider publishes.
    pub async fn releases(&self) -> Result<Vec<EconomicRelease>> {
        dispatch_via!(
            self,
            ECONOMIC,
            as_economic,
            EconomicReleases,
            fetch_economic_releases,
            []
        )
    }
}
