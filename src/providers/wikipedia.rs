//! Wikipedia provider (keyless) — S&P 500 index constituents only.

use super::{IndicesProvider, Operation, ProviderAdapter, ProviderCore};
use crate::error::Result;

pub(crate) struct WikipediaProvider;

impl ProviderCore for WikipediaProvider {
    fn id(&self) -> super::Provider {
        super::Provider::Wikipedia
    }
}

#[async_trait::async_trait]
impl IndicesProvider for WikipediaProvider {
    async fn fetch_indices_quote(
        &self,
        _symbol: &str,
    ) -> Result<finance_query_models::indices::IndexQuote> {
        Err(self.not_supported(Operation::IndicesQuote))
    }

    async fn fetch_index_constituents(
        &self,
        index: finance_query_models::indices::MajorIndex,
    ) -> Result<Vec<finance_query_models::indices::IndexConstituent>> {
        crate::adapters::wikipedia::fetch_index_constituents_response(index).await
    }

    async fn fetch_index_constituent_changes(
        &self,
        index: finance_query_models::indices::MajorIndex,
    ) -> Result<Vec<finance_query_models::indices::IndexConstituentChange>> {
        crate::adapters::wikipedia::fetch_index_constituent_changes_response(index).await
    }
}

#[async_trait::async_trait]
impl ProviderAdapter for WikipediaProvider {
    fn as_indices(&self) -> Option<&dyn IndicesProvider> {
        Some(self)
    }
}
