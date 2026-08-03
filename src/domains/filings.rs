//! SEC filing query handle.
//!
//! Created via [`Providers::filings`](crate::Providers::filings). Always
//! available — backed by EDGAR (keyless) with optional Polygon fallback.

use crate::error::Result;

domain_handle! {
    /// SEC filing data backed by configured data providers.
    ///
    /// Created via [`Providers::filings`](crate::Providers::filings).
    pub struct Filings { symbol, symbol }
    cache: crate::models::filings::ProviderFilings
}

impl Filings {
    /// Fetch SEC filings for this symbol.
    pub async fn get(&self) -> Result<crate::models::filings::ProviderFilings> {
        fetch_via!(
            self,
            symbol,
            FILINGS,
            as_filings,
            Filings,
            fetch_filings,
            crate::models::filings::ProviderFilings
        )
    }

    /// Fetch the sectioned text of one filing by accession number via the
    /// FILINGS route (currently Polygon only). Not cached.
    pub async fn sections(
        &self,
        accession_number: &str,
        form: crate::models::filings::FilingSectionForm,
    ) -> Result<Vec<crate::models::filings::FilingSection>> {
        let accession = accession_number.to_string();
        self.providers
            .fetch(crate::providers::Capability::FILINGS, move |p| {
                let accession = accession.clone();
                let p = p.clone();
                async move {
                    p.as_filings()
                        .ok_or_else(|| {
                            p.not_supported(crate::providers::Operation::FilingSections)
                        })?
                        .fetch_filing_sections(&accession, form)
                        .await
                }
            })
            .await
    }

    /// Fetch risk factors extracted from this symbol's SEC filings via the
    /// FILINGS route (currently Polygon only). Not cached.
    pub async fn risk_factors(&self) -> Result<Vec<crate::models::filings::RiskFactor>> {
        let symbol: String = self.symbol().to_string();
        self.providers
            .fetch(crate::providers::Capability::FILINGS, move |p| {
                let symbol = symbol.clone();
                let p = p.clone();
                async move {
                    p.as_filings()
                        .ok_or_else(|| p.not_supported(crate::providers::Operation::RiskFactors))?
                        .fetch_risk_factors(&symbol)
                        .await
                }
            })
            .await
    }
}
