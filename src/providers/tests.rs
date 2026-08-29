//! Tests for the provider registry, routing table, and dispatch.

use std::sync::Arc;

use super::*;
use crate::error::{FinanceError, Result};

/// A CHART-capable provider that does not implement spark — exercises the
/// default trait method and proves spark now dispatches through the set.
struct NoSparkProvider;

impl ProviderCore for NoSparkProvider {
    fn id(&self) -> Provider {
        Provider::Yahoo
    }
}

#[async_trait::async_trait]
impl ChartProvider for NoSparkProvider {
    async fn fetch_chart(
        &self,
        _: &str,
        _: crate::Interval,
        _: crate::TimeRange,
    ) -> Result<crate::models::chart::Chart> {
        Err(FinanceError::ApiError(
            "not exercised by these tests".into(),
        ))
    }
}

#[async_trait::async_trait]
impl ProviderAdapter for NoSparkProvider {
    fn as_chart(&self) -> Option<&dyn ChartProvider> {
        Some(self)
    }
}

#[test]
fn capabilities_derive_from_accessors() {
    let caps = ProviderAdapter::capabilities(&NoSparkProvider);
    assert_eq!(caps, Capability::CHART);
    assert!(!caps.contains(Capability::QUOTE));
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
            operation: Operation::Spark,
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
        Routes::new(Fetch::Sequential),
    );
    let result = set
        .fetch(Capability::CHART, |p| {
            let p = p.clone();
            async move {
                p.as_chart()
                    .ok_or_else(|| p.not_supported(Operation::Spark))?
                    .fetch_spark(
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

// `Provider::capabilities()` derives from the adapters' accessor overrides
// for every unit-struct provider, so those can't drift by construction.
// Yahoo is the lone hand-declared set (constructing `YahooProvider` needs a
// live auth handshake); this pins the const to the accessor overrides in
// `yahoo.rs` — update both together.
#[test]
fn yahoo_caps_const_matches_declared_capabilities() {
    let expected = Capability::QUOTE
        .union(Capability::CHART)
        .union(Capability::FUNDAMENTALS)
        .union(Capability::CORPORATE)
        .union(Capability::OPTIONS)
        .union(Capability::MARKET)
        .union(Capability::INDICES)
        .union(Capability::COMMODITIES)
        .union(Capability::DISCOVERY)
        .union(Capability::CALENDAR)
        .union(Capability::FUTURES);
    assert_eq!(yahoo::CAPS, expected);
}

/// Polygon gained CALENDAR (holidays) and AV gained MARKET (movers) via
/// accessor overrides; `Provider::capabilities()` must reflect that.
#[cfg(feature = "polygon")]
#[test]
fn polygon_derives_calendar_capability() {
    assert!(
        Provider::Polygon
            .capabilities()
            .contains(Capability::CALENDAR)
    );
}

#[cfg(feature = "fred")]
#[test]
fn fred_derives_calendar_capability() {
    assert!(Provider::Fred.capabilities().contains(Capability::CALENDAR));
}

#[test]
fn local_market_calendar_derives_calendar_capability() {
    assert!(
        Provider::LocalMarketCalendar
            .capabilities()
            .contains(Capability::CALENDAR)
    );
}

#[test]
fn local_exchange_derives_discovery_capability() {
    assert!(
        Provider::LocalExchange
            .capabilities()
            .contains(Capability::DISCOVERY)
    );
}

#[cfg(feature = "alphavantage")]
#[test]
fn alphavantage_derives_market_capability() {
    assert!(
        Provider::AlphaVantage
            .capabilities()
            .contains(Capability::MARKET)
    );
}

#[cfg(feature = "gdelt")]
#[test]
fn gdelt_derives_crypto_and_forex_capability() {
    let caps = Provider::Gdelt.capabilities();
    assert!(caps.contains(Capability::CRYPTO));
    assert!(caps.contains(Capability::FOREX));
}

#[tokio::test]
async fn all_candidates_unsupported_surfaces_precise_operation() {
    // NoSparkProvider supports CHART but not spark: the final error must
    // name the spark operation, not collapse to NoProviderAvailable(CHART).
    let set = ProviderSet::new(
        vec![Arc::new(NoSparkProvider)],
        Routes::new(Fetch::Sequential),
    );
    let err = set
        .fetch(Capability::CHART, |p| {
            let p = p.clone();
            async move {
                p.as_chart()
                    .ok_or_else(|| p.not_supported(Operation::Spark))?
                    .fetch_spark(
                        &["AAPL"],
                        crate::Interval::OneDay,
                        crate::TimeRange::FiveDays,
                    )
                    .await
            }
        })
        .await
        .unwrap_err();
    assert!(matches!(
        err,
        FinanceError::NotSupported {
            operation: Operation::Spark,
            ..
        }
    ));
}

#[tokio::test]
async fn real_errors_outrank_not_supported_in_final_error() {
    // Two candidates: one lacking the op (NotSupported), one failing for
    // real. The real failure is the actionable error and must win.
    struct FailingChartProvider;
    impl ProviderCore for FailingChartProvider {
        fn id(&self) -> Provider {
            Provider::Edgar
        }
    }
    #[async_trait::async_trait]
    impl ChartProvider for FailingChartProvider {
        async fn fetch_chart(
            &self,
            _: &str,
            _: crate::Interval,
            _: crate::TimeRange,
        ) -> Result<crate::models::chart::Chart> {
            Err(FinanceError::ApiError("upstream 500".into()))
        }
        async fn fetch_spark(
            &self,
            _: &[&str],
            _: crate::Interval,
            _: crate::TimeRange,
        ) -> Result<Vec<(String, crate::models::chart::spark::Spark)>> {
            Err(FinanceError::ApiError("upstream 500".into()))
        }
    }
    #[async_trait::async_trait]
    impl ProviderAdapter for FailingChartProvider {
        fn as_chart(&self) -> Option<&dyn ChartProvider> {
            Some(self)
        }
    }

    let routes =
        Routes::new(Fetch::Sequential).route(Capability::CHART, [Provider::Yahoo, Provider::Edgar]);
    let set = ProviderSet::new(
        vec![Arc::new(NoSparkProvider), Arc::new(FailingChartProvider)],
        routes,
    );
    let err = set
        .fetch(Capability::CHART, |p| {
            let p = p.clone();
            async move {
                p.as_chart()
                    .ok_or_else(|| p.not_supported(Operation::Spark))?
                    .fetch_spark(
                        &["AAPL"],
                        crate::Interval::OneDay,
                        crate::TimeRange::FiveDays,
                    )
                    .await
            }
        })
        .await
        .unwrap_err();
    assert!(matches!(err, FinanceError::ApiError(_)), "got {err:?}");
}

/// A [`ChartProvider`] that returns `RateLimited` for its first
/// `fail_times` calls, then succeeds — for exercising [`RetryPolicy`].
struct FlakyChartProvider {
    id: Provider,
    fail_times: usize,
    calls: std::sync::atomic::AtomicUsize,
}

impl FlakyChartProvider {
    fn new(id: Provider, fail_times: usize) -> Self {
        Self {
            id,
            fail_times,
            calls: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    fn call_count(&self) -> usize {
        self.calls.load(std::sync::atomic::Ordering::SeqCst)
    }
}

impl ProviderCore for FlakyChartProvider {
    fn id(&self) -> Provider {
        self.id
    }
}

#[async_trait::async_trait]
impl ChartProvider for FlakyChartProvider {
    async fn fetch_chart(
        &self,
        symbol: &str,
        _: crate::Interval,
        _: crate::TimeRange,
    ) -> Result<crate::models::chart::Chart> {
        let n = self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if n < self.fail_times {
            return Err(FinanceError::RateLimited {
                retry_after: Some(0),
            });
        }
        Ok(crate::models::chart::Chart {
            symbol: symbol.to_string(),
            meta: Default::default(),
            candles: Vec::new(),
            interval: None,
            range: None,
            provider_id: Some(self.id),
        })
    }
}

#[async_trait::async_trait]
impl ProviderAdapter for FlakyChartProvider {
    fn as_chart(&self) -> Option<&dyn ChartProvider> {
        Some(self)
    }
}

async fn fetch_chart_via(set: &ProviderSet) -> Result<crate::models::chart::Chart> {
    set.fetch(Capability::CHART, |p| {
        let p = p.clone();
        async move {
            p.as_chart()
                .ok_or_else(|| p.not_supported(Operation::Chart))?
                .fetch_chart("AAPL", crate::Interval::OneDay, crate::TimeRange::FiveDays)
                .await
        }
    })
    .await
}

#[tokio::test]
async fn no_retry_policy_means_a_single_attempt_per_candidate() {
    let provider = Arc::new(FlakyChartProvider::new(Provider::Yahoo, usize::MAX));
    let set = ProviderSet::new(
        vec![provider.clone() as Arc<dyn ProviderAdapter>],
        Routes::new(Fetch::Sequential),
    );
    let err = fetch_chart_via(&set).await.unwrap_err();
    assert!(matches!(err, FinanceError::RateLimited { .. }));
    assert_eq!(
        provider.call_count(),
        1,
        "no retry policy set: exactly one attempt"
    );
}

#[tokio::test]
async fn retry_policy_retries_rate_limited_then_succeeds() {
    let provider = Arc::new(FlakyChartProvider::new(Provider::Yahoo, 2));
    let set = ProviderSet::new(
        vec![provider.clone() as Arc<dyn ProviderAdapter>],
        Routes::new(Fetch::Sequential),
    )
    .with_retry_policy(Some(RetryPolicy::new(5)));

    let chart = fetch_chart_via(&set)
        .await
        .expect("should eventually succeed");
    assert_eq!(chart.symbol, "AAPL");
    // 2 failures + 1 success = 3 calls.
    assert_eq!(provider.call_count(), 3);
}

#[tokio::test]
async fn retry_policy_exhausts_and_falls_through_to_next_provider() {
    let always_limited = Arc::new(FlakyChartProvider::new(Provider::Yahoo, usize::MAX));
    let succeeds = Arc::new(FlakyChartProvider::new(Provider::Edgar, 0));
    let routes =
        Routes::new(Fetch::Sequential).route(Capability::CHART, [Provider::Yahoo, Provider::Edgar]);
    let set = ProviderSet::new(
        vec![
            always_limited.clone() as Arc<dyn ProviderAdapter>,
            succeeds.clone() as Arc<dyn ProviderAdapter>,
        ],
        routes,
    )
    .with_retry_policy(Some(RetryPolicy::new(3)));

    let chart = fetch_chart_via(&set).await.expect("falls through to Edgar");
    assert_eq!(chart.provider_id, Some(Provider::Edgar));
    // Exhausts all 3 attempts on the first candidate before falling through.
    assert_eq!(always_limited.call_count(), 3);
    assert_eq!(succeeds.call_count(), 1);
}

#[tokio::test]
async fn health_reflects_failures_and_recovers_after_success() {
    let provider = Arc::new(FlakyChartProvider::new(Provider::Yahoo, 1));
    let set = ProviderSet::new(
        vec![provider.clone() as Arc<dyn ProviderAdapter>],
        Routes::new(Fetch::Sequential),
    );

    // First call fails (RateLimited), recorded as a failure.
    assert!(fetch_chart_via(&set).await.is_err());
    let health = set.health();
    assert_eq!(health.len(), 1);
    assert_eq!(health[0].provider, Provider::Yahoo);
    assert_eq!(health[0].recent_failures, 1);
    assert!(health[0].last_error.is_some());

    // Second call succeeds, clearing last_error.
    assert!(fetch_chart_via(&set).await.is_ok());
    let health = set.health();
    assert_eq!(health[0].recent_successes, 1);
    assert!(health[0].last_error.is_none());
}

/// A provider that only implements `as_quote`, so a CHART dispatch hits
/// its `NotSupported` path — which must not count against its health.
struct QuoteOnlyProvider;

impl ProviderCore for QuoteOnlyProvider {
    fn id(&self) -> Provider {
        Provider::Yahoo
    }
}

#[async_trait::async_trait]
impl ProviderAdapter for QuoteOnlyProvider {}

#[tokio::test]
async fn not_supported_is_excluded_from_health_accounting() {
    let set = ProviderSet::new(
        vec![Arc::new(QuoteOnlyProvider) as Arc<dyn ProviderAdapter>],
        Routes::new(Fetch::Sequential),
    );
    // Goes through the real `record_health`, not a copy of its match.
    assert!(fetch_chart_via(&set).await.is_err());

    let health = set.health();
    assert!(health[0].is_healthy);
    assert_eq!(health[0].recent_successes, 0);
    assert_eq!(health[0].recent_failures, 0);
    assert!(health[0].last_error.is_none());
}

/// Companion to `tests/provider_wire_format.rs`, which can only name the
/// variants it hard-codes. The match has no wildcard, so a new variant fails
/// to compile until it is pinned here.
fn id_for(provider: Provider) -> &'static str {
    match provider {
        Provider::Yahoo => "yahoo",
        #[cfg(feature = "polygon")]
        Provider::Polygon => "polygon",
        #[cfg(feature = "fmp")]
        Provider::Fmp => "fmp",
        #[cfg(feature = "alphavantage")]
        Provider::AlphaVantage => "alphavantage",
        #[cfg(feature = "crypto")]
        Provider::CoinGecko => "coingecko",
        #[cfg(feature = "fred")]
        Provider::Fred => "fred",
        #[cfg(feature = "worldbank")]
        Provider::WorldBank => "worldbank",
        #[cfg(feature = "fiscaldata")]
        Provider::FiscalData => "fiscaldata",
        #[cfg(feature = "bls")]
        Provider::Bls => "bls",
        #[cfg(feature = "frankfurter")]
        Provider::Frankfurter => "frankfurter",
        #[cfg(feature = "binance")]
        Provider::Binance => "binance",
        #[cfg(feature = "kraken")]
        Provider::Kraken => "kraken",
        #[cfg(feature = "finra")]
        Provider::Finra => "finra",
        #[cfg(feature = "defi")]
        Provider::DefiLlama => "defillama",
        #[cfg(feature = "gdelt")]
        Provider::Gdelt => "gdelt",
        #[cfg(feature = "cftc")]
        Provider::Cftc => "cftc",
        #[cfg(feature = "nasdaq")]
        Provider::Nasdaq => "nasdaq",
        #[cfg(feature = "wikipedia")]
        Provider::Wikipedia => "wikipedia",
        #[cfg(any(feature = "housetrades", feature = "senatetrades"))]
        Provider::CongressTrades => "congresstrades",
        Provider::Edgar => "edgar",
        Provider::LocalMarketCalendar => "local_market_calendar",
        Provider::LocalExchange => "local_exchange",
        Provider::Custom(id) => id.as_str(),
    }
}

/// The match in `id_for` catches a variant missing from the pin; this catches
/// one missing from `all()`, which is a hand-maintained list.
#[test]
fn every_variant_has_its_serialized_form_pinned() {
    const EXPECTED: usize = 4
        + cfg!(feature = "polygon") as usize
        + cfg!(feature = "fmp") as usize
        + cfg!(feature = "alphavantage") as usize
        + cfg!(feature = "crypto") as usize
        + cfg!(feature = "fred") as usize
        + cfg!(feature = "worldbank") as usize
        + cfg!(feature = "fiscaldata") as usize
        + cfg!(feature = "bls") as usize
        + cfg!(feature = "frankfurter") as usize
        + cfg!(feature = "binance") as usize
        + cfg!(feature = "kraken") as usize
        + cfg!(feature = "finra") as usize
        + cfg!(feature = "defi") as usize
        + cfg!(feature = "gdelt") as usize
        + cfg!(feature = "cftc") as usize
        + cfg!(feature = "nasdaq") as usize
        + cfg!(feature = "wikipedia") as usize
        + cfg!(any(feature = "housetrades", feature = "senatetrades")) as usize;

    let all = Provider::all();
    assert_eq!(all.len(), EXPECTED, "Provider::all() is missing a variant");
    for provider in all {
        let id = id_for(provider);
        assert_eq!(
            serde_json::to_string(&provider).unwrap(),
            format!("\"{id}\"")
        );
        assert_eq!(provider.as_str(), id);
        assert_eq!(Provider::from_id_str(id), Some(provider));
    }
}

#[test]
fn a_route_without_a_mode_uses_the_table_default() {
    let routes = Routes::new(Fetch::Parallel).route(Capability::QUOTE, [Provider::Yahoo]);
    assert_eq!(routes.fetch_mode_for(Capability::QUOTE), Fetch::Parallel);
}

#[test]
fn a_route_mode_overrides_the_table_default() {
    let routes = Routes::new(Fetch::Parallel).route_with(
        Capability::FUNDAMENTALS,
        [Provider::Yahoo],
        Fetch::Sequential,
    );
    assert_eq!(
        routes.fetch_mode_for(Capability::FUNDAMENTALS),
        Fetch::Sequential
    );
    assert_eq!(routes.fetch_mode(), Fetch::Parallel);
}

#[test]
fn an_unrouted_capability_uses_the_table_default() {
    let routes = Routes::new(Fetch::Sequential).route_with(
        Capability::QUOTE,
        [Provider::Yahoo],
        Fetch::Parallel,
    );
    assert_eq!(routes.fetch_mode_for(Capability::CHART), Fetch::Sequential);
}

#[test]
fn modes_are_independent_across_capabilities() {
    let routes = Routes::new(Fetch::Sequential)
        .route_with(Capability::QUOTE, [Provider::Yahoo], Fetch::Parallel)
        .route(Capability::FUNDAMENTALS, [Provider::Yahoo]);
    assert_eq!(routes.fetch_mode_for(Capability::QUOTE), Fetch::Parallel);
    assert_eq!(
        routes.fetch_mode_for(Capability::FUNDAMENTALS),
        Fetch::Sequential
    );
}

struct CountingChartProvider {
    id: Provider,
    calls: std::sync::atomic::AtomicUsize,
}

impl CountingChartProvider {
    fn new(id: Provider) -> Self {
        Self {
            id,
            calls: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    fn calls(&self) -> usize {
        self.calls.load(std::sync::atomic::Ordering::SeqCst)
    }
}

impl ProviderCore for CountingChartProvider {
    fn id(&self) -> Provider {
        self.id
    }
}

#[async_trait::async_trait]
impl ChartProvider for CountingChartProvider {
    async fn fetch_chart(
        &self,
        symbol: &str,
        _: crate::Interval,
        _: crate::TimeRange,
    ) -> Result<crate::models::chart::Chart> {
        self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        tokio::task::yield_now().await;
        Ok(crate::models::chart::Chart {
            symbol: symbol.to_string(),
            meta: Default::default(),
            candles: Vec::new(),
            interval: None,
            range: None,
            provider_id: Some(self.id),
        })
    }
}

impl ProviderAdapter for CountingChartProvider {
    fn as_chart(&self) -> Option<&dyn ChartProvider> {
        Some(self)
    }
}

async fn chart_calls_under(routes: Routes) -> (usize, usize) {
    let first = Arc::new(CountingChartProvider::new(Provider::Yahoo));
    let second = Arc::new(CountingChartProvider::new(Provider::Edgar));
    let set = ProviderSet::new(
        vec![
            first.clone() as Arc<dyn ProviderAdapter>,
            second.clone() as Arc<dyn ProviderAdapter>,
        ],
        routes,
    );
    set.fetch(Capability::CHART, |p| {
        let p = p.clone();
        async move {
            p.as_chart()
                .ok_or_else(|| p.not_supported(Operation::Chart))?
                .fetch_chart("AAPL", crate::Interval::OneDay, crate::TimeRange::FiveDays)
                .await
        }
    })
    .await
    .expect("a provider succeeds");
    (first.calls(), second.calls())
}

#[tokio::test]
async fn a_sequential_route_stops_at_the_first_success() {
    let routes =
        Routes::new(Fetch::Sequential).route(Capability::CHART, [Provider::Yahoo, Provider::Edgar]);
    assert_eq!(chart_calls_under(routes).await, (1, 0));
}

#[tokio::test]
async fn route_with_parallel_overrides_a_sequential_default() {
    let routes = Routes::new(Fetch::Sequential).route_with(
        Capability::CHART,
        [Provider::Yahoo, Provider::Edgar],
        Fetch::Parallel,
    );
    assert_eq!(chart_calls_under(routes).await, (1, 1));
}
