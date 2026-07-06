//! Multi-provider financial data aggregation.

pub mod config;

#[cfg(feature = "alphavantage")]
pub(crate) mod alphavantage;
#[cfg(feature = "crypto")]
pub(crate) mod coingecko;
pub(crate) mod edgar;
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
use crate::models::quote::QuoteSummaryResponse;
use futures::stream::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Typed identifier for a financial data provider.
///
/// Variants are feature-gated: unavailable providers are excluded at compile time.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Provider {
    #[default]
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
    /// SEC EDGAR filings (always available, keyless).
    Edgar,
}

impl Provider {
    /// Parse a provider id string back to the typed variant.
    /// Returns `None` if the string doesn't match any known provider.
    /// Prefer this over string conversion to avoid panics.
    pub fn from_id_str(s: &str) -> Option<Self> {
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
            "edgar" => Some(Self::Edgar),
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
            Self::Edgar => "edgar",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// How providers are queried.
pub enum Fetch {
    /// Try providers in priority order; first success wins.
    Sequential,
    /// Fire all providers concurrently; first success wins.
    Parallel,
}

/// Capability bits that a provider can declare.
///
/// Route a capability to specific providers using `.route(Capability::QUOTE, [Provider::Fmp])`.
/// If no route is configured for a capability, all providers declaring that capability are used.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Capability(u32);

impl Capability {
    /// Equity quote data — price, volume, market cap, fundamentals summary.
    pub const QUOTE: Self = Self(1 << 0);
    /// Historical OHLCV chart data across intervals and ranges.
    pub const CHART: Self = Self(1 << 1);
    /// Financial statements — income, balance sheet, cash flow.
    pub const FUNDAMENTALS: Self = Self(1 << 2);
    /// Corporate events — news, recommendations, SEC filings metadata.
    pub const CORPORATE: Self = Self(1 << 3);
    /// Options chains and contract data.
    pub const OPTIONS: Self = Self(1 << 4);
    // bit 5 reserved for future use

    /// Cryptocurrency quotes and market data.
    pub const CRYPTO: Self = Self(1 << 6);
    /// Macro-economic data series (FRED, GDP, CPI, etc.).
    pub const ECONOMIC: Self = Self(1 << 7);
    // bit 8 reserved for future use

    /// Foreign exchange currency pair quotes.
    pub const FOREX: Self = Self(1 << 9);
    /// Stock market index quotes (S&P 500, NASDAQ, etc.).
    pub const INDICES: Self = Self(1 << 10);
    /// Futures contract quotes.
    pub const FUTURES: Self = Self(1 << 11);
    /// Commodity price quotes (gold, oil, etc.).
    pub const COMMODITIES: Self = Self(1 << 12);
    // bit 13 reserved for future use

    /// SEC EDGAR filing data.
    pub const FILINGS: Self = Self(1 << 14);

    /// Returns `true` if this capability set includes all bits in `other`.
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Returns a short lowercase name for this capability (e.g., `"quote"`, `"chart"`).
    ///
    /// Returns `"unknown"` for combined capability flags or unrecognised bits.
    pub fn name(self) -> &'static str {
        match self.0 {
            x if x == Self::QUOTE.0 => "quote",
            x if x == Self::CHART.0 => "chart",
            x if x == Self::FUNDAMENTALS.0 => "fundamentals",
            x if x == Self::CORPORATE.0 => "corporate",
            x if x == Self::OPTIONS.0 => "options",
            x if x == Self::CRYPTO.0 => "crypto",
            x if x == Self::ECONOMIC.0 => "economic",
            x if x == Self::FOREX.0 => "forex",
            x if x == Self::INDICES.0 => "indices",
            x if x == Self::FUTURES.0 => "futures",
            x if x == Self::COMMODITIES.0 => "commodities",
            x if x == Self::FILINGS.0 => "filings",
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

/// Per-capability provider routing table.
///
/// Maps each [`Capability`] to an ordered list of [`Provider`]s to try.
/// When a capability has no entry, all providers declaring that capability are used.
pub struct Routes {
    pub(crate) map: HashMap<Capability, Vec<Provider>>,
    pub(crate) fetch: Fetch,
}

impl Routes {
    pub fn new(fetch: Fetch) -> Self {
        Self {
            map: HashMap::new(),
            fetch,
        }
    }
}

#[async_trait::async_trait]
pub(crate) trait ProviderAdapter: Send + Sync {
    fn id(&self) -> &'static str;
    fn capabilities(&self) -> Capability;

    /// Initialize this provider. Called once during construction.
    async fn initialize(&self) -> Result<()> {
        Ok(())
    }

    fn not_supported(&self, operation: &'static str) -> FinanceError {
        FinanceError::NotSupported {
            provider: self.id(),
            operation,
        }
    }

    // Single-ticker quote routing; Ticker uses first_yahoo() directly for crumb auth.
    // Wired up for future multi-provider single-ticker quote routing.
    async fn fetch_quote(&self, _: &str) -> Result<QuoteSummaryResponse> {
        Err(self.not_supported("quote"))
    }
    async fn fetch_chart(
        &self,
        _: &str,
        _: crate::Interval,
        _: crate::TimeRange,
    ) -> Result<crate::models::chart::Chart> {
        Err(self.not_supported("chart"))
    }
    async fn fetch_chart_range(
        &self,
        _: &str,
        _: crate::Interval,
        _: i64,
        _: i64,
    ) -> Result<crate::models::chart::Chart> {
        Err(self.not_supported("chart_range"))
    }
    async fn fetch_financials(
        &self,
        _: &str,
        _: crate::StatementType,
        _: crate::Frequency,
    ) -> Result<crate::models::fundamentals::FinancialStatement> {
        Err(self.not_supported("financials"))
    }
    async fn fetch_news(&self, _: &str) -> Result<Vec<crate::models::corporate::news::News>> {
        Err(self.not_supported("news"))
    }
    async fn fetch_similar_symbols(
        &self,
        _: &str,
        _: u32,
    ) -> Result<Vec<crate::models::corporate::recommendation::SimilarSymbol>> {
        Err(self.not_supported("recommendations"))
    }
    async fn fetch_options(
        &self,
        _: &str,
        _: Option<i64>,
    ) -> Result<crate::models::options::Options> {
        Err(self.not_supported("options"))
    }
    async fn fetch_events(&self, _: &str) -> Result<crate::models::chart::events::ChartEvents> {
        Err(self.not_supported("events"))
    }
    /// Fetch quotes for multiple symbols in a single request.
    /// Returns `(symbol, QuoteSummaryResponse)` pairs — only partially populated
    /// (price module only) since batch endpoints don't return full quoteSummary data.
    async fn fetch_quotes_batch(&self, _: &[&str]) -> Result<Vec<(String, QuoteSummaryResponse)>> {
        Err(self.not_supported("quotes_batch"))
    }

    /// Fetch lightweight sparkline data for multiple symbols in a single request.
    /// Returns successfully-parsed `(symbol, Spark)` pairs; callers fill in
    /// missing-symbol errors for any symbol absent from the result.
    async fn fetch_spark(
        &self,
        _: &[&str],
        _: crate::Interval,
        _: crate::TimeRange,
    ) -> Result<Vec<(String, crate::models::chart::spark::Spark)>> {
        Err(self.not_supported("spark"))
    }

    #[cfg(any(
        feature = "crypto",
        feature = "alphavantage",
        feature = "fmp",
        feature = "polygon"
    ))]
    async fn fetch_crypto_quote(
        &self,
        _: &str,
        _: &str,
    ) -> Result<crate::models::crypto::CryptoQuote> {
        Err(self.not_supported("crypto_quote"))
    }

    #[cfg(any(feature = "fred", feature = "alphavantage", feature = "polygon"))]
    async fn fetch_economic_series(
        &self,
        _: &str,
    ) -> Result<crate::models::economic::EconomicSeries> {
        Err(self.not_supported("economic_series"))
    }

    #[cfg(any(feature = "polygon", feature = "fmp", feature = "alphavantage"))]
    async fn fetch_forex_quote(
        &self,
        _from: &str,
        _to: &str,
    ) -> Result<crate::models::forex::ForexQuote> {
        Err(self.not_supported("forex_quote"))
    }

    #[cfg(any(feature = "polygon", feature = "fmp"))]
    async fn fetch_indices_quote(&self, _: &str) -> Result<crate::models::indices::IndexQuote> {
        Err(self.not_supported("indices_quote"))
    }

    #[cfg(feature = "polygon")]
    async fn fetch_futures_quote(&self, _: &str) -> Result<crate::models::futures::FuturesQuote> {
        Err(self.not_supported("futures_quote"))
    }

    #[cfg(any(feature = "fmp", feature = "alphavantage"))]
    async fn fetch_commodities_quote(
        &self,
        _: &str,
    ) -> Result<crate::models::commodities::CommodityQuote> {
        Err(self.not_supported("commodities_quote"))
    }

    async fn fetch_filings(&self, _: &str) -> Result<crate::models::filings::ProviderFilings> {
        Err(self.not_supported("filings"))
    }
}

pub(crate) struct ProviderSet {
    providers: Vec<Arc<dyn ProviderAdapter>>,
    yahoo_client: Option<Arc<YahooClient>>,
    routes: Routes,
}

impl ProviderSet {
    pub fn new(
        providers: Vec<Arc<dyn ProviderAdapter>>,
        yahoo_client: Option<Arc<YahooClient>>,
        routes: Routes,
    ) -> Self {
        Self {
            providers,
            yahoo_client,
            routes,
        }
    }

    /// Returns the providers to use for a given capability, respecting any
    /// explicit route configured via `.route()`. When no route is configured,
    /// defaults to Yahoo for all capabilities and EDGAR for filings.
    fn candidates_for(&self, cap: Capability) -> Vec<&Arc<dyn ProviderAdapter>> {
        if let Some(provider_ids) = self.routes.map.get(&cap) {
            provider_ids
                .iter()
                .filter_map(|id| self.providers.iter().find(|p| p.id() == id.as_str()))
                .collect()
        } else if cap == Capability::FILINGS {
            // Default: EDGAR (keyless SEC filings) first, then Yahoo
            let mut v: Vec<&Arc<dyn ProviderAdapter>> = self
                .providers
                .iter()
                .filter(|p| p.id() == "edgar")
                .collect();
            v.extend(self.providers.iter().filter(|p| p.id() == "yahoo"));
            v
        } else {
            // Default: Yahoo only
            self.providers
                .iter()
                .filter(|p| p.id() == "yahoo")
                .collect()
        }
    }

    fn no_provider(cap: Capability) -> FinanceError {
        FinanceError::NoProviderAvailable {
            operation: cap.name(),
        }
    }

    fn finish_err(cap: Capability, last: Option<FinanceError>) -> FinanceError {
        last.unwrap_or_else(|| Self::no_provider(cap))
    }

    pub(crate) async fn fetch<T, F, Fut>(&self, cap: Capability, f: F) -> Result<T>
    where
        F: Fn(&Arc<dyn ProviderAdapter>) -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let candidates = self.candidates_for(cap);
        if candidates.is_empty() {
            return Err(Self::no_provider(cap));
        }
        match self.routes.fetch {
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
            Fetch::Parallel => {
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

    pub(crate) fn first_yahoo(&self) -> Result<Arc<YahooClient>> {
        self.yahoo_client
            .as_ref()
            .map(Arc::clone)
            .ok_or_else(|| FinanceError::NoProviderAvailable { operation: "yahoo" })
    }
}

#[allow(dead_code)] // used by fmp, polygon, alphavantage feature-gated providers
pub(crate) fn json_value_to_f64(value: serde_json::Value) -> Option<f64> {
    value
        .as_f64()
        .or_else(|| value.as_i64().map(|v| v as f64))
        .or_else(|| value.as_u64().map(|v| v as f64))
        .or_else(|| value.as_str().and_then(|s| s.parse::<f64>().ok()))
        .or_else(|| {
            value
                .get("raw")
                .and_then(|raw| raw.as_f64().or_else(|| raw.as_i64().map(|v| v as f64)))
        })
}

#[allow(dead_code)] // used by fmp, polygon, alphavantage feature-gated providers
pub(crate) fn build_financial_statement(
    symbol: String,
    statement_type: String,
    frequency: String,
    provider_id: Provider,
    data: std::collections::HashMap<String, std::collections::HashMap<String, serde_json::Value>>,
) -> crate::models::fundamentals::FinancialStatement {
    let statement = data
        .into_iter()
        .filter_map(|(metric, values)| {
            let values: std::collections::HashMap<String, f64> = values
                .into_iter()
                .filter_map(|(date, value)| json_value_to_f64(value).map(|v| (date, v)))
                .collect();
            if values.is_empty() {
                None
            } else {
                Some((metric, values))
            }
        })
        .collect();
    crate::models::fundamentals::FinancialStatement {
        symbol,
        statement_type,
        frequency,
        statement,
        provider_id: Some(provider_id),
    }
}

pub(crate) fn build_options(
    symbol: String,
    provider_id: Provider,
    expiration_dates: Vec<i64>,
    calls: Vec<crate::models::options::OptionContract>,
    puts: Vec<crate::models::options::OptionContract>,
) -> crate::models::options::Options {
    use std::collections::BTreeMap;

    let mut chains_by_expiration: BTreeMap<
        i64,
        (
            Vec<crate::models::options::OptionContract>,
            Vec<crate::models::options::OptionContract>,
        ),
    > = BTreeMap::new();

    for contract in calls {
        let exp = contract.expiration.unwrap_or(0);
        chains_by_expiration
            .entry(exp)
            .or_default()
            .0
            .push(contract);
    }
    for contract in puts {
        let exp = contract.expiration.unwrap_or(0);
        chains_by_expiration
            .entry(exp)
            .or_default()
            .1
            .push(contract);
    }

    let option_chains: Vec<crate::models::options::response::OptionChainData> =
        chains_by_expiration
            .into_iter()
            .map(
                |(expiration, (c, p))| crate::models::options::response::OptionChainData {
                    expiration_date: expiration,
                    has_mini_options: None,
                    calls: Some(c),
                    puts: Some(p),
                },
            )
            .collect();

    let expiration_dates = if expiration_dates.is_empty() {
        option_chains
            .iter()
            .map(|chain| chain.expiration_date)
            .collect()
    } else {
        let mut v: Vec<i64> = expiration_dates;
        v.sort_unstable();
        v.dedup();
        v
    };

    let mut strikes: Vec<f64> = option_chains
        .iter()
        .flat_map(|chain| {
            chain
                .calls
                .as_deref()
                .unwrap_or_default()
                .iter()
                .map(|c| c.strike)
                .chain(
                    chain
                        .puts
                        .as_deref()
                        .unwrap_or_default()
                        .iter()
                        .map(|p| p.strike),
                )
        })
        .collect();
    strikes.sort_by(|a, b| a.total_cmp(b));
    strikes.dedup_by(|a, b| a.total_cmp(b).is_eq());

    let result = crate::models::options::response::OptionChainResult {
        underlying_symbol: Some(symbol),
        expiration_dates: Some(expiration_dates),
        strikes: Some(strikes),
        has_mini_options: None,
        quote: None,
        options: option_chains,
    };

    crate::models::options::Options {
        option_chain: crate::models::options::response::OptionChainContainer {
            result: vec![result],
            error: None,
        },
        provider_id: Some(provider_id),
    }
}

#[allow(dead_code)] // used by fmp feature-gated provider
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
        crate::TimeRange::YearToDate => unreachable!("YTD handled by early return above"),
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
    routes: Routes,
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
            Provider::Edgar => providers.push(Arc::new(edgar::EdgarProvider)),
        }
    }
    // Auto-inject EDGAR if no other FILINGS-capable provider was configured
    let has_filings = providers
        .iter()
        .any(|p| p.capabilities().contains(Capability::FILINGS));
    if !has_filings {
        providers.push(Arc::new(edgar::EdgarProvider));
    }
    Ok(ProviderSet::new(providers, yahoo_client, routes))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A CHART-capable provider that does not implement spark — exercises the
    /// default trait method and proves spark now dispatches through the set.
    struct NoSparkProvider;

    #[async_trait::async_trait]
    impl ProviderAdapter for NoSparkProvider {
        fn id(&self) -> &'static str {
            "yahoo"
        }
        fn capabilities(&self) -> Capability {
            Capability::CHART
        }
    }

    #[tokio::test]
    async fn fetch_spark_defaults_to_not_supported() {
        let err = NoSparkProvider
            .fetch_spark(
                &["AAPL"],
                crate::Interval::OneDay,
                crate::TimeRange::FiveDays,
            )
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            FinanceError::NotSupported {
                operation: "spark",
                ..
            }
        ));
    }

    #[tokio::test]
    async fn spark_routes_through_provider_set() {
        // The CHART default route resolves to the "yahoo"-id provider; routing a
        // provider that lacks spark must surface an error rather than silently
        // hitting a hardcoded Yahoo client.
        let set = ProviderSet::new(
            vec![Arc::new(NoSparkProvider)],
            None,
            Routes::new(Fetch::Sequential),
        );
        let result = set
            .fetch(Capability::CHART, |p| {
                let p = p.clone();
                async move {
                    p.fetch_spark(
                        &["AAPL"],
                        crate::Interval::OneDay,
                        crate::TimeRange::FiveDays,
                    )
                    .await
                }
            })
            .await;
        assert!(result.is_err());
    }
}
