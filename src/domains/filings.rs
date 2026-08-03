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

    /// Full-text search over this filer's filings via the FILINGS route
    /// (currently EDGAR only). Not cached.
    ///
    /// The handle's symbol is resolved to a CIK and applied as a filter unless
    /// `filters` already names one. Use [`search_all`](Self::search_all) to
    /// search every filer instead.
    pub async fn search(
        &self,
        query: &str,
        filters: crate::models::filings::FilingSearchFilters,
    ) -> Result<Vec<crate::models::filings::FilingSearchHit>> {
        let mut filters = filters;
        if filters.cik.is_none() {
            let cik = crate::adapters::edgar::resolve_cik(self.symbol()).await?;
            filters.cik = Some(format!("{cik:010}"));
        }
        self.search_all(query, filters).await
    }

    /// Full-text search across every filer via the FILINGS route (currently
    /// EDGAR only). Not cached.
    ///
    /// Searches filing *text*, so it answers "which filings mention this"
    /// rather than "what has this company filed" — the query shape
    /// [`get`](Self::get) cannot express.
    pub async fn search_all(
        &self,
        query: &str,
        filters: crate::models::filings::FilingSearchFilters,
    ) -> Result<Vec<crate::models::filings::FilingSearchHit>> {
        let query = query.to_string();
        let filters = std::sync::Arc::new(filters);
        self.providers
            .fetch(crate::providers::Capability::FILINGS, move |p| {
                let query = query.clone();
                let filters = std::sync::Arc::clone(&filters);
                let p = p.clone();
                async move {
                    p.as_filings()
                        .ok_or_else(|| p.not_supported(crate::providers::Operation::FilingSearch))?
                        .fetch_filing_search(&query, &filters)
                        .await
                }
            })
            .await
    }

    /// Insider transactions reported on this issuer's Forms 3/4/5, parsed from
    /// the filed XML, via the FILINGS route (currently EDGAR only).
    ///
    /// `limit` caps how many *filings* are read, not how many transactions come
    /// back — one Form 4 can report several lines. Each filing costs two extra
    /// requests, so keep it modest. Not cached.
    pub async fn insider_trades(
        &self,
        limit: u32,
    ) -> Result<Vec<crate::models::filings::InsiderTrade>> {
        let symbol: String = self.symbol().to_string();
        self.providers
            .fetch(crate::providers::Capability::FILINGS, move |p| {
                let symbol = symbol.clone();
                let p = p.clone();
                async move {
                    p.as_filings()
                        .ok_or_else(|| p.not_supported(crate::providers::Operation::InsiderTrades))?
                        .fetch_insider_trades(&symbol, limit)
                        .await
                }
            })
            .await
    }

    /// The latest 13F-HR information table filed by this symbol's CIK, via the
    /// FILINGS route (currently EDGAR only).
    ///
    /// The handle's symbol identifies the *filer*, so this is only meaningful
    /// for listed institutional managers (e.g. `BRK-B`) — an issuer that files
    /// no 13F errors rather than returning an empty list. Not cached.
    pub async fn institutional_holdings(
        &self,
    ) -> Result<Vec<crate::models::filings::InstitutionalHolding>> {
        let symbol: String = self.symbol().to_string();
        self.providers
            .fetch(crate::providers::Capability::FILINGS, move |p| {
                let symbol = symbol.clone();
                let p = p.clone();
                async move {
                    p.as_filings()
                        .ok_or_else(|| {
                            p.not_supported(crate::providers::Operation::InstitutionalHoldings)
                        })?
                        .fetch_institutional_holdings(&symbol)
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
