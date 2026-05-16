//! Multi-provider financial data aggregation.

#![allow(dead_code)]

#[cfg(feature = "alphavantage")]
pub(crate) mod alphavantage;
#[cfg(feature = "crypto")]
pub(crate) mod coingecko;
#[cfg(feature = "fmp")]
pub(crate) mod fmp;
#[cfg(feature = "fred")]
pub(crate) mod fred;
#[cfg(feature = "polygon")]
pub(crate) mod polygon;
pub(crate) mod types;
pub(crate) mod yahoo;

use crate::adapters::yahoo::client::{ClientConfig, YahooClient};
use crate::error::{FinanceError, Result};
use futures::stream::StreamExt;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Typed identifier for a financial data provider.
///
/// Variants are feature-gated: unavailable providers are excluded at compile time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Provider {
    /// Yahoo Finance (always available).
    Yahoo,
    /// Polygon.io (requires `polygon` feature).
    #[cfg(feature = "polygon")]
    Polygon,
    /// Financial Modeling Prep (requires `fmp` feature).
    #[cfg(feature = "fmp")]
    Fmp,
    /// Alpha Vantage (requires `alphavantage` feature).
    #[cfg(feature = "alphavantage")]
    AlphaVantage,
    /// CoinGecko cryptocurrency data (requires `crypto` feature).
    #[cfg(feature = "crypto")]
    CoinGecko,
    /// FRED economic data (requires `fred` feature).
    #[cfg(feature = "fred")]
    Fred,
}

impl Provider {
    /// Parse a provider id string back to the typed variant.
    pub(crate) fn from_id_str(s: &str) -> Option<Self> {
        match s {
            "yahoo" => Some(Self::Yahoo),
            #[cfg(feature = "polygon")]
            "polygon" => Some(Self::Polygon),
            #[cfg(feature = "fmp")]
            "fmp" => Some(Self::Fmp),
            #[cfg(feature = "alphavantage")]
            "alphavantage" => Some(Self::AlphaVantage),
            #[cfg(feature = "crypto")]
            "coingecko" => Some(Self::CoinGecko),
            #[cfg(feature = "fred")]
            "fred" => Some(Self::Fred),
            _ => None,
        }
    }

    /// String identifier matching [`ProviderAdapter::id`].
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Yahoo => "yahoo",
            #[cfg(feature = "polygon")]
            Self::Polygon => "polygon",
            #[cfg(feature = "fmp")]
            Self::Fmp => "fmp",
            #[cfg(feature = "alphavantage")]
            Self::AlphaVantage => "alphavantage",
            #[cfg(feature = "crypto")]
            Self::CoinGecko => "coingecko",
            #[cfg(feature = "fred")]
            Self::Fred => "fred",
        }
    }
}

impl From<&str> for Provider {
    fn from(s: &str) -> Self {
        Self::from_id_str(s).unwrap_or_else(|| panic!("invalid provider id string: {s}"))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// How providers are queried.
pub enum Fetch {
    /// Try providers in priority order; first success wins.
    Sequential,
    /// Fire all providers concurrently; first success wins.
    Parallel,
    /// Query all providers concurrently; collect all successes.
    All,
}

#[allow(private_interfaces)]
/// How results from multiple providers are combined.
///
/// Implement this trait to define custom merge policies. Built-in policies:
/// - [`Prefer`]: use the first provider's result (default)
/// - [`Enrich`]: primary wins; fallbacks fill missing fields
pub trait Merge: Send + Sync {
    /// Whether this policy needs results from all providers.
    fn wants_all(&self) -> bool {
        false
    }

    /// Merge quote data from a fallback provider into the primary.
    fn merge_quote(
        &self,
        primary: types::QuoteData,
        _fallback: types::QuoteData,
    ) -> types::QuoteData {
        primary
    }
    /// Merge chart data from a fallback provider into the primary.
    fn merge_chart(
        &self,
        primary: types::ChartData,
        _fallback: types::ChartData,
    ) -> types::ChartData {
        primary
    }
    /// Merge financial statement data from a fallback provider into the primary.
    fn merge_financials(
        &self,
        primary: types::FinancialStatementData,
        _fallback: types::FinancialStatementData,
    ) -> types::FinancialStatementData {
        primary
    }
    /// Merge options data from a fallback provider into the primary.
    fn merge_options(
        &self,
        primary: types::OptionsData,
        _fallback: types::OptionsData,
    ) -> types::OptionsData {
        primary
    }
}

/// Use the first provider's successful result.
///
/// This is the default merge policy.
pub struct Prefer;
impl Merge for Prefer {}

/// Primary fills first; fallbacks backfill missing optional fields.
pub struct Enrich;
impl Merge for Enrich {
    #[allow(private_interfaces)]
    fn wants_all(&self) -> bool {
        true
    }
    #[allow(private_interfaces)]
    fn merge_quote(&self, p: types::QuoteData, f: types::QuoteData) -> types::QuoteData {
        types::backfill_quote_data(p, f)
    }
    #[allow(private_interfaces)]
    fn merge_chart(&self, p: types::ChartData, f: types::ChartData) -> types::ChartData {
        types::backfill_chart_data(p, f)
    }
    #[allow(private_interfaces)]
    fn merge_financials(
        &self,
        p: types::FinancialStatementData,
        f: types::FinancialStatementData,
    ) -> types::FinancialStatementData {
        types::backfill_financial_data(p, f)
    }
    #[allow(private_interfaces)]
    fn merge_options(&self, p: types::OptionsData, f: types::OptionsData) -> types::OptionsData {
        types::backfill_options_data(p, f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Capability(u16);

impl Capability {
    pub const QUOTE: Self = Self(1 << 0);
    pub const CHART: Self = Self(1 << 1);
    pub const FUNDAMENTALS: Self = Self(1 << 2);
    pub const CORPORATE: Self = Self(1 << 3);
    pub const OPTIONS: Self = Self(1 << 4);
    pub const MARKET: Self = Self(1 << 5);
    pub const CRYPTO: Self = Self(1 << 6);
    pub const ECONOMIC: Self = Self(1 << 7);
    pub const DISCOVERY: Self = Self(1 << 8);
    pub const FOREX: Self = Self(1 << 9);
    pub const INDICES: Self = Self(1 << 10);
    pub const FUTURES: Self = Self(1 << 11);
    pub const COMMODITIES: Self = Self(1 << 12);
    pub const TECHNICALS: Self = Self(1 << 13);
    pub const FILINGS: Self = Self(1 << 14);
    pub const SENTIMENT: Self = Self(1 << 15);
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }
    pub fn name(self) -> &'static str {
        match self.0.trailing_zeros() {
            0 => "quote",
            1 => "chart",
            2 => "fundamentals",
            3 => "corporate",
            4 => "options",
            5 => "market",
            6 => "crypto",
            7 => "economic",
            8 => "discovery",
            9 => "forex",
            10 => "indices",
            11 => "futures",
            12 => "commodities",
            13 => "technicals",
            14 => "filings",
            15 => "sentiment",
            _ => "unknown",
        }
    }
}

impl std::ops::BitOr for Capability {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

#[async_trait::async_trait]
pub(crate) trait ProviderAdapter: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str {
        self.id()
    }
    fn capabilities(&self) -> Capability;

    /// Initialize this provider. Called once during construction.
    ///
    /// Override to read API keys, set up connections, or validate configuration.
    /// Default is a no-op — keyless providers (Yahoo, CoinGecko) need no init.
    async fn initialize(&self) -> Result<()> {
        Ok(())
    }

    fn not_supported(&self, operation: &'static str) -> FinanceError {
        FinanceError::NotSupported {
            provider: self.id(),
            operation,
        }
    }
    async fn fetch_quote(&self, _: &str) -> Result<types::QuoteData> {
        Err(self.not_supported("quote"))
    }
    async fn fetch_chart(
        &self,
        _: &str,
        _: crate::Interval,
        _: crate::TimeRange,
    ) -> Result<types::ChartData> {
        Err(self.not_supported("chart"))
    }
    async fn fetch_chart_range(
        &self,
        _: &str,
        _: crate::Interval,
        _: i64,
        _: i64,
    ) -> Result<types::ChartData> {
        Err(self.not_supported("chart_range"))
    }
    async fn fetch_financials(
        &self,
        _: &str,
        _: crate::StatementType,
        _: crate::Frequency,
    ) -> Result<types::FinancialStatementData> {
        Err(self.not_supported("financials"))
    }
    async fn fetch_news(&self, _: &str) -> Result<Vec<types::NewsData>> {
        Err(self.not_supported("news"))
    }
    async fn fetch_similar_symbols(
        &self,
        _: &str,
        _: u32,
    ) -> Result<Vec<types::SimilarSymbolData>> {
        Err(self.not_supported("recommendations"))
    }
    async fn fetch_options(&self, _: &str, _: Option<i64>) -> Result<types::OptionsData> {
        Err(self.not_supported("options"))
    }
    async fn fetch_events(&self, _: &str) -> Result<types::EventsData> {
        Err(self.not_supported("events"))
    }
    async fn fetch_quotes_batch(&self, _: &[&str]) -> Result<Vec<types::QuoteData>> {
        Err(self.not_supported("quotes_batch"))
    }
    async fn fetch_market_hours(&self, _: &str) -> Result<types::MarketHoursData> {
        Err(self.not_supported("market_hours"))
    }
    async fn fetch_trending(&self) -> Result<Vec<types::TrendingData>> {
        Err(self.not_supported("trending"))
    }
    async fn fetch_market_summary(&self) -> Result<Vec<types::MarketSummaryData>> {
        Err(self.not_supported("market_summary"))
    }
    async fn fetch_crypto_quote(&self, _: &str, _: &str) -> Result<types::CryptoQuoteData> {
        Err(self.not_supported("crypto_quote"))
    }
    async fn fetch_crypto_coins(&self, _: &str, _: u32) -> Result<Vec<types::CryptoCoinData>> {
        Err(self.not_supported("crypto_coins"))
    }
    async fn fetch_economic_series(&self, _: &str) -> Result<types::EconomicSeriesData> {
        Err(self.not_supported("economic_series"))
    }
    async fn fetch_treasury_yields(&self, _: u32) -> Result<types::TreasuryYieldData> {
        Err(self.not_supported("treasury_yields"))
    }
}

pub(crate) struct ProviderSet {
    providers: Vec<Arc<dyn ProviderAdapter>>,
    yahoo_client: Option<Arc<YahooClient>>,
    fetch: Fetch,
    merge: Arc<dyn Merge>,
}

impl ProviderSet {
    pub fn new(
        providers: Vec<Arc<dyn ProviderAdapter>>,
        yahoo_client: Option<Arc<YahooClient>>,
        fetch: Fetch,
        merge: Arc<dyn Merge>,
    ) -> Self {
        Self {
            providers,
            yahoo_client,
            fetch,
            merge,
        }
    }
    pub(crate) fn fetch_mode(&self) -> Fetch {
        self.fetch
    }
    pub(crate) fn merger(&self) -> &Arc<dyn Merge> {
        &self.merge
    }
    fn capable_of(&self, cap: Capability) -> Vec<&Arc<dyn ProviderAdapter>> {
        self.providers
            .iter()
            .filter(|p| p.capabilities().contains(cap))
            .collect()
    }
    fn no_provider(cap: Capability) -> FinanceError {
        FinanceError::NoProviderAvailable {
            operation: cap.name(),
        }
    }
    fn finish_err(cap: Capability, last: Option<FinanceError>) -> FinanceError {
        last.unwrap_or_else(|| Self::no_provider(cap))
    }

    pub(crate) async fn try_fetch<T, F, Fut>(&self, cap: Capability, f: F) -> Result<T>
    where
        F: Fn(&Arc<dyn ProviderAdapter>) -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let candidates = self.capable_of(cap);
        if candidates.is_empty() {
            return Err(Self::no_provider(cap));
        }
        match self.fetch {
            Fetch::Sequential => {
                let mut last = None;
                for p in &candidates {
                    match f(p).await {
                        Ok(v) => return Ok(v),
                        Err(FinanceError::NotSupported { .. }) => continue,
                        Err(e) => last = Some(e),
                    }
                }
                Err(Self::finish_err(cap, last))
            }
            Fetch::Parallel | Fetch::All => {
                // For single-result callers, Fetch::All degrades to Fetch::Parallel:
                // fire all providers concurrently and return the first success.
                // Use try_fetch_all directly when all results are needed for merging.
                let mut futs = futures::stream::FuturesUnordered::new();
                for p in &candidates {
                    futs.push(f(p));
                }
                let mut last = None;
                while let Some(r) = futs.next().await {
                    match r {
                        Ok(v) => return Ok(v),
                        Err(FinanceError::NotSupported { .. }) => continue,
                        Err(e) => last = Some(e),
                    }
                }
                Err(Self::finish_err(cap, last))
            }
        }
    }

    pub(crate) async fn try_fetch_all<T, F, Fut>(&self, cap: Capability, f: F) -> Result<Vec<T>>
    where
        F: Fn(&Arc<dyn ProviderAdapter>) -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let candidates = self.capable_of(cap);
        if candidates.is_empty() {
            return Err(Self::no_provider(cap));
        }
        match self.fetch {
            Fetch::Sequential => {
                let mut successes = Vec::new();
                let mut last = None;
                for p in &candidates {
                    match f(p).await {
                        Ok(v) => {
                            successes.push(v);
                        }
                        Err(FinanceError::NotSupported { .. }) => continue,
                        Err(e) => last = Some(e),
                    }
                }
                if successes.is_empty() {
                    Err(Self::finish_err(cap, last))
                } else {
                    Ok(successes)
                }
            }
            Fetch::Parallel | Fetch::All => {
                let mut futs = futures::stream::FuturesUnordered::new();
                for (pri, p) in candidates.iter().enumerate() {
                    let future = f(p);
                    futs.push(async move { (pri, future.await) });
                }
                let mut successes = Vec::new();
                let mut last = None;
                while let Some((pri, r)) = futs.next().await {
                    match r {
                        Ok(v) => successes.push((pri, v)),
                        Err(FinanceError::NotSupported { .. }) => continue,
                        Err(e) => last = Some(e),
                    }
                }
                if successes.is_empty() {
                    return Err(Self::finish_err(cap, last));
                }
                successes.sort_by_key(|(p, _)| *p);
                Ok(successes.into_iter().map(|(_, v)| v).collect())
            }
        }
    }

    pub(crate) fn first_yahoo(&self) -> Result<Arc<YahooClient>> {
        self.yahoo_client
            .as_ref()
            .map(Arc::clone)
            .ok_or_else(|| FinanceError::NoProviderAvailable { operation: "yahoo" })
    }
}

pub(crate) fn range_to_dates(range: crate::TimeRange) -> (String, String) {
    use chrono::{Datelike, Utc};
    let end = Utc::now();
    if range == crate::TimeRange::YearToDate {
        let year = end.year();
        let start = chrono::NaiveDate::from_ymd_opt(year, 1, 1)
            .and_then(|d| d.and_hms_opt(0, 0, 0))
            .map(|dt| dt.and_utc())
            .unwrap_or(end);
        return (
            start.format("%Y-%m-%d").to_string(),
            end.format("%Y-%m-%d").to_string(),
        );
    }
    let days = match range {
        crate::TimeRange::OneDay => 1,
        crate::TimeRange::FiveDays => 5,
        crate::TimeRange::OneMonth => 30,
        crate::TimeRange::ThreeMonths => 90,
        crate::TimeRange::SixMonths => 180,
        crate::TimeRange::OneYear => 365,
        crate::TimeRange::TwoYears => 730,
        crate::TimeRange::FiveYears => 1825,
        crate::TimeRange::TenYears => 3650,
        crate::TimeRange::Max => 36500,
        crate::TimeRange::YearToDate => unreachable!(),
    };
    let start = end - chrono::Duration::days(days);
    (
        start.format("%Y-%m-%d").to_string(),
        end.format("%Y-%m-%d").to_string(),
    )
}

pub(crate) async fn build_providers(
    ids: &[Provider],
    config: &ClientConfig,
    fetch: Fetch,
    merge: Arc<dyn Merge>,
) -> Result<ProviderSet> {
    use yahoo::YahooProvider;
    let mut providers: Vec<Arc<dyn ProviderAdapter>> = Vec::new();
    let mut yahoo_client: Option<Arc<YahooClient>> = None;
    for &id in ids {
        match id {
            Provider::Yahoo => {
                let yp = YahooProvider::new(config).await?;
                yahoo_client = Some(yp.client_arc());
                providers.push(Arc::new(yp));
            }
            #[cfg(feature = "polygon")]
            Provider::Polygon => {
                let pp = polygon::PolygonProvider;
                pp.initialize().await?;
                providers.push(Arc::new(pp));
            }
            #[cfg(feature = "fmp")]
            Provider::Fmp => {
                let fp = fmp::FmpProvider;
                fp.initialize().await?;
                providers.push(Arc::new(fp));
            }
            #[cfg(feature = "alphavantage")]
            Provider::AlphaVantage => {
                let av = alphavantage::AlphaVantageProvider;
                av.initialize().await?;
                providers.push(Arc::new(av));
            }
            #[cfg(feature = "crypto")]
            Provider::CoinGecko => providers.push(Arc::new(coingecko::CoinGeckoProvider)),
            #[cfg(feature = "fred")]
            Provider::Fred => {
                let fp = fred::FredProvider;
                fp.initialize().await?;
                providers.push(Arc::new(fp));
            }
        }
    }
    Ok(ProviderSet::new(providers, yahoo_client, fetch, merge))
}
