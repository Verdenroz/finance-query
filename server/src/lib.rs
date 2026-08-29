//! finance-query-server library: shared types and modules used by both the
//! HTTP server binary and the MCP server.

pub mod cache;
pub mod graphql;
pub mod lang;
pub mod metrics;
pub mod params;
pub mod rate_limit;
pub mod responses;
pub mod services;

use finance_query::FinanceError;
use finance_query::feeds::FeedSource;
use finance_query::streaming::{NewsStream, PriceStream, PriceUpdate};
use futures_util::{Stream, StreamExt};
use std::collections::{HashMap, HashSet};
use std::pin::Pin;
use std::sync::{Arc, OnceLock};
use std::task::{Context, Poll};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;

/// Fan-out channel depth, matching the library's own price-stream capacity.
const FANOUT_CAPACITY: usize = 1024;

#[derive(Clone)]
pub struct AppState {
    pub cache: cache::Cache,
    pub stream_hub: StreamHub,
    pub feed_hub: FeedHub,
    /// Multi-provider routing for capabilities beyond Yahoo (e.g. Alpha
    /// Vantage-backed company profile/earnings/transcripts). `Providers`
    /// isn't `Clone`, hence the `Arc`.
    pub providers: Arc<finance_query::Providers>,
}

/// Which keyed providers are configured, driving [`route_table`].
struct ProviderFlags {
    fmp: bool,
    alphavantage: bool,
    fred: bool,
    polygon: bool,
}

/// The capability → provider fallback list for every routed capability.
/// Keyless providers stay in the route unconditionally; keyed providers are
/// added only when their flag is set, so a capability never depends solely
/// on a key that isn't configured when a free alternative exists.
fn route_table(
    flags: ProviderFlags,
) -> Vec<(finance_query::Capability, Vec<finance_query::Provider>)> {
    use finance_query::{Capability, Provider};

    let mut routes = Vec::new();

    // Capabilities Yahoo already serves keyless: FMP/AV are additive alternates.
    for cap in [
        Capability::QUOTE,
        Capability::CHART,
        Capability::MARKET,
        Capability::OPTIONS,
    ] {
        let mut route = vec![Provider::Yahoo];
        if flags.fmp && cap != Capability::OPTIONS {
            route.push(Provider::Fmp); // FMP has no OptionsProvider impl
        }
        if flags.alphavantage {
            route.push(Provider::AlphaVantage);
        }
        routes.push((cap, route));
    }

    let mut corporate = vec![Provider::Yahoo];
    if flags.fmp {
        corporate.push(Provider::Fmp);
    }
    if flags.alphavantage {
        corporate.push(Provider::AlphaVantage);
    }
    corporate.push(Provider::Edgar); // press releases via 8-K exhibits
    routes.push((Capability::CORPORATE, corporate));

    let mut fundamentals = Vec::new();
    if flags.fmp {
        fundamentals.push(Provider::Fmp);
    }
    if flags.alphavantage {
        fundamentals.push(Provider::AlphaVantage);
    }
    fundamentals.push(Provider::Yahoo);
    fundamentals.push(Provider::Finra); // short-sale volume only
    routes.push((Capability::FUNDAMENTALS, fundamentals));

    let mut discovery = Vec::new();
    if flags.fmp {
        discovery.push(Provider::Fmp);
    }
    if flags.alphavantage {
        discovery.push(Provider::AlphaVantage);
    }
    discovery.push(Provider::Yahoo);
    discovery.push(Provider::LocalExchange); // static exchange table
    routes.push((Capability::DISCOVERY, discovery));

    let mut calendar = vec![Provider::Yahoo, Provider::LocalMarketCalendar];
    if flags.fmp {
        calendar.push(Provider::Fmp);
    }
    if flags.alphavantage {
        calendar.push(Provider::AlphaVantage);
    }
    calendar.push(Provider::Nasdaq);
    if flags.fred {
        calendar.push(Provider::Fred);
    }
    routes.push((Capability::CALENDAR, calendar));

    let mut commodities = vec![Provider::Yahoo];
    if flags.fmp {
        commodities.push(Provider::Fmp);
    }
    if flags.alphavantage {
        commodities.push(Provider::AlphaVantage);
    }
    routes.push((Capability::COMMODITIES, commodities));

    let mut indices = vec![Provider::Yahoo, Provider::Wikipedia];
    if flags.fmp {
        indices.push(Provider::Fmp); // INDICES has no AV route among integrated providers.
    }
    routes.push((Capability::INDICES, indices));

    routes.push((Capability::FUTURES, vec![Provider::Yahoo, Provider::Cftc]));

    let mut economic = Vec::new();
    if flags.fred {
        economic.push(Provider::Fred);
    }
    economic.push(Provider::WorldBank);
    economic.push(Provider::FiscalData);
    if flags.alphavantage {
        economic.push(Provider::AlphaVantage);
    }
    economic.push(Provider::Bls);
    routes.push((Capability::ECONOMIC, economic));

    let mut crypto = Vec::new();
    if flags.fmp {
        crypto.push(Provider::Fmp);
    }
    crypto.push(Provider::CoinGecko);
    crypto.push(Provider::Binance);
    crypto.push(Provider::Kraken);
    crypto.push(Provider::DefiLlama); // TVL and stablecoin supply only
    crypto.push(Provider::Gdelt); // market-wide news only
    routes.push((Capability::CRYPTO, crypto));

    let mut forex = Vec::new();
    if flags.fmp {
        forex.push(Provider::Fmp);
    }
    if flags.alphavantage {
        forex.push(Provider::AlphaVantage);
    }
    forex.push(Provider::Frankfurter);
    forex.push(Provider::Gdelt); // market-wide news only
    routes.push((Capability::FOREX, forex));

    let mut filings = Vec::new();
    if flags.fmp {
        filings.push(Provider::Fmp);
    }
    filings.push(Provider::Edgar);
    filings.push(Provider::CongressTrades);
    filings.push(Provider::Yahoo);
    routes.push((Capability::FILINGS, filings));

    if flags.polygon {
        // Appended last so a Polygon key widens reach without reordering the
        // providers already serving these capabilities.
        const POLYGON_CAPS: [Capability; 13] = [
            Capability::QUOTE,
            Capability::CHART,
            Capability::OPTIONS,
            Capability::CORPORATE,
            Capability::FUNDAMENTALS,
            Capability::DISCOVERY,
            Capability::CALENDAR,
            Capability::INDICES,
            Capability::FUTURES,
            Capability::ECONOMIC,
            Capability::CRYPTO,
            Capability::FOREX,
            Capability::FILINGS,
        ];
        for (cap, route) in &mut routes {
            if POLYGON_CAPS.contains(cap) {
                route.push(Provider::Polygon);
            }
        }
    }

    routes
}

/// Build the multi-provider routing shared by `AppState`. Each keyed
/// provider is only routed in when its API key env var is set; a field
/// backed solely by an unconfigured provider falls through to
/// `NotSupported`, surfaced as a `501` by `finance_error_to_gql`.
pub async fn build_providers() -> Arc<finance_query::Providers> {
    use finance_query::Providers;

    let log_routing = |name: &str, key: &str, enabled: bool| match enabled {
        true => tracing::info!("{name} routing enabled"),
        false => tracing::info!("{name} not configured (set {key} to enable)"),
    };
    let flags = ProviderFlags {
        fmp: std::env::var("FMP_API_KEY").is_ok(),
        alphavantage: std::env::var("ALPHAVANTAGE_API_KEY").is_ok(),
        fred: std::env::var("FRED_API_KEY").is_ok(),
        polygon: std::env::var("POLYGON_API_KEY").is_ok(),
    };
    log_routing("Alpha Vantage", "ALPHAVANTAGE_API_KEY", flags.alphavantage);
    log_routing("FMP", "FMP_API_KEY", flags.fmp);
    log_routing("FRED", "FRED_API_KEY", flags.fred);
    log_routing("Polygon", "POLYGON_API_KEY", flags.polygon);

    let mut builder = Providers::builder();
    for (cap, route) in route_table(flags) {
        builder = builder.route(cap, route);
    }

    match builder.build().await {
        Ok(providers) => Arc::new(providers),
        Err(e) => {
            tracing::warn!(
                "Failed to initialize provider routing, falling back to Yahoo-only: {e}"
            );
            Arc::new(
                Providers::builder()
                    .build()
                    .await
                    .expect("Yahoo-only Providers build cannot fail"),
            )
        }
    }
}

#[cfg(test)]
mod provider_routing_tests {
    use super::*;
    use finance_query::{Capability, Provider};

    fn route(routes: &[(Capability, Vec<Provider>)], cap: Capability) -> Option<&[Provider]> {
        routes
            .iter()
            .find(|(c, _)| *c == cap)
            .map(|(_, r)| r.as_slice())
    }

    #[test]
    fn no_keys_keeps_a_keyless_floor() {
        let routes = route_table(ProviderFlags {
            fmp: false,
            alphavantage: false,
            fred: false,
            polygon: false,
        });
        assert_eq!(
            route(&routes, Capability::QUOTE),
            Some(&[Provider::Yahoo][..])
        );
        assert_eq!(
            route(&routes, Capability::CRYPTO),
            Some(
                &[
                    Provider::CoinGecko,
                    Provider::Binance,
                    Provider::Kraken,
                    Provider::DefiLlama,
                    Provider::Gdelt
                ][..]
            )
        );
        assert_eq!(
            route(&routes, Capability::FOREX),
            Some(&[Provider::Frankfurter, Provider::Gdelt][..])
        );
        assert_eq!(
            route(&routes, Capability::ECONOMIC),
            Some(&[Provider::WorldBank, Provider::FiscalData, Provider::Bls][..])
        );
        assert_eq!(
            route(&routes, Capability::DISCOVERY),
            Some(&[Provider::Yahoo, Provider::LocalExchange][..])
        );
        assert_eq!(
            route(&routes, Capability::INDICES),
            Some(&[Provider::Yahoo, Provider::Wikipedia][..])
        );
        assert_eq!(
            route(&routes, Capability::COMMODITIES),
            Some(&[Provider::Yahoo][..])
        );
        assert_eq!(
            route(&routes, Capability::FUTURES),
            Some(&[Provider::Yahoo, Provider::Cftc][..])
        );
        assert_eq!(
            route(&routes, Capability::CALENDAR),
            Some(
                &[
                    Provider::Yahoo,
                    Provider::LocalMarketCalendar,
                    Provider::Nasdaq
                ][..]
            )
        );
        assert_eq!(
            route(&routes, Capability::FILINGS),
            Some(&[Provider::Edgar, Provider::CongressTrades, Provider::Yahoo][..])
        );
    }

    #[test]
    fn fmp_key_adds_fmp_everywhere_it_serves_but_not_options() {
        let routes = route_table(ProviderFlags {
            fmp: true,
            alphavantage: false,
            fred: false,
            polygon: false,
        });
        assert!(
            route(&routes, Capability::QUOTE)
                .unwrap()
                .contains(&Provider::Fmp)
        );
        assert!(
            !route(&routes, Capability::OPTIONS)
                .unwrap()
                .contains(&Provider::Fmp)
        );
        assert_eq!(
            route(&routes, Capability::INDICES),
            Some(&[Provider::Yahoo, Provider::Wikipedia, Provider::Fmp][..])
        );
        assert_eq!(
            route(&routes, Capability::FILINGS),
            Some(
                &[
                    Provider::Fmp,
                    Provider::Edgar,
                    Provider::CongressTrades,
                    Provider::Yahoo
                ][..]
            )
        );
    }

    #[test]
    fn fundamentals_always_has_a_yahoo_floor() {
        let routes = route_table(ProviderFlags {
            fmp: true,
            alphavantage: true,
            fred: false,
            polygon: false,
        });
        let fundamentals = route(&routes, Capability::FUNDAMENTALS).unwrap();
        let position = |p| fundamentals.iter().position(|q| *q == p);
        assert!(position(Provider::Yahoo) > position(Provider::Fmp));
        assert!(position(Provider::Yahoo) > position(Provider::AlphaVantage));
    }

    #[test]
    fn keyless_providers_are_reachable_with_no_key_set() {
        let routes = route_table(ProviderFlags {
            fmp: false,
            alphavantage: false,
            fred: false,
            polygon: false,
        });
        let serves = |cap, provider| route(&routes, cap).unwrap().contains(&provider);
        assert!(serves(Capability::FUNDAMENTALS, Provider::Finra));
        assert!(serves(Capability::CRYPTO, Provider::DefiLlama));
        assert!(serves(Capability::FUTURES, Provider::Cftc));
        assert!(serves(Capability::DISCOVERY, Provider::LocalExchange));
        assert!(serves(Capability::ECONOMIC, Provider::Bls));
    }

    #[test]
    fn a_polygon_key_appends_without_reordering() {
        let keyless = route_table(ProviderFlags {
            fmp: false,
            alphavantage: false,
            fred: false,
            polygon: false,
        });
        let keyed = route_table(ProviderFlags {
            fmp: false,
            alphavantage: false,
            fred: false,
            polygon: true,
        });
        for (cap, before) in &keyless {
            let after = route(&keyed, *cap).unwrap();
            match after.len() == before.len() {
                true => assert_eq!(after, before.as_slice()),
                false => {
                    assert_eq!(&after[..before.len()], before.as_slice());
                    assert_eq!(after.last(), Some(&Provider::Polygon));
                }
            }
        }
        assert!(
            route(&keyed, Capability::QUOTE)
                .unwrap()
                .contains(&Provider::Polygon)
        );
    }

    #[test]
    fn polygon_stays_out_of_capabilities_it_does_not_serve() {
        let routes = route_table(ProviderFlags {
            fmp: false,
            alphavantage: false,
            fred: false,
            polygon: true,
        });
        assert!(
            !route(&routes, Capability::MARKET)
                .unwrap()
                .contains(&Provider::Polygon)
        );
        assert!(
            !route(&routes, Capability::COMMODITIES)
                .unwrap()
                .contains(&Provider::Polygon)
        );
    }
}

/// One price tick plus its wire JSON, shared by every connected client.
///
/// Serializing per client rebuilt identical JSON once per client per tick; the
/// first consumer that needs the wire form now builds it for all of them, and
/// consumers that never need it (GraphQL) pay nothing.
pub struct SharedTick {
    update: PriceUpdate,
    json: OnceLock<Arc<str>>,
}

impl SharedTick {
    fn new(update: PriceUpdate) -> Self {
        Self {
            update,
            json: OnceLock::new(),
        }
    }

    /// The decoded tick.
    pub fn update(&self) -> &PriceUpdate {
        &self.update
    }

    /// The tick's JSON encoding, built once and shared.
    pub fn json(&self) -> Arc<str> {
        self.json
            .get_or_init(|| {
                serde_json::to_string(&self.update)
                    .unwrap_or_default()
                    .into()
            })
            .clone()
    }
}

/// A per-client receiver over the hub's shared tick fan-out.
pub struct TickStream {
    inner: BroadcastStream<Arc<SharedTick>>,
}

impl Stream for TickStream {
    type Item = Arc<SharedTick>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        loop {
            match Pin::new(&mut this.inner).poll_next(cx) {
                Poll::Ready(Some(Ok(tick))) => return Poll::Ready(Some(tick)),
                // A lag means this client missed ticks, not that the feed ended.
                Poll::Ready(Some(Err(_))) => continue,
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

/// Process-wide hub that maintains a single upstream Yahoo Finance stream.
///
/// Multiple downstream WebSocket clients can subscribe/unsubscribe to symbols.
/// Symbol subscriptions are ref-counted so each symbol is only subscribed once upstream.
#[derive(Clone, Default)]
pub struct StreamHub {
    inner: Arc<tokio::sync::Mutex<StreamHubInner>>,
}

#[derive(Default)]
struct StreamHubInner {
    upstream: Option<PriceStream>,
    /// Re-broadcast of `upstream` carrying shareable ticks.
    fanout: Option<broadcast::Sender<Arc<SharedTick>>>,
    pump: Option<tokio::task::JoinHandle<()>>,
    symbol_ref_counts: HashMap<String, usize>,
}

/// Read the upstream stream once and hand every client the same tick.
async fn pump_ticks(mut upstream: PriceStream, fanout: broadcast::Sender<Arc<SharedTick>>) {
    while let Some(update) = upstream.next().await {
        let _ = fanout.send(Arc::new(SharedTick::new(update)));
    }
}

impl StreamHub {
    /// Create a new empty hub.
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn resubscribe(&self) -> Option<TickStream> {
        let inner = self.inner.lock().await;
        inner.fanout.as_ref().map(|fanout| TickStream {
            inner: BroadcastStream::new(fanout.subscribe()),
        })
    }

    pub async fn subscribe_symbols(&self, symbols: &[String]) -> Result<(), FinanceError> {
        let unique: HashSet<String> = symbols
            .iter()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        if unique.is_empty() {
            return Ok(());
        }

        let mut inner = self.inner.lock().await;

        // Track which symbols are newly needed upstream.
        let mut newly_needed: Vec<String> = Vec::new();
        for symbol in &unique {
            let count = inner.symbol_ref_counts.entry(symbol.clone()).or_insert(0);
            if *count == 0 {
                newly_needed.push(symbol.clone());
            }
            *count += 1;
        }
        metrics::STREAM_SUBSCRIPTIONS_ACTIVE.set(inner.symbol_ref_counts.len() as f64);

        // Create upstream stream if this is the first active subscription.
        if inner.upstream.is_none() {
            let stream = PriceStream::subscribe(unique.iter().map(|s| s.as_str())).await?;
            let (fanout, _) = broadcast::channel(FANOUT_CAPACITY);
            inner.pump = Some(tokio::spawn(pump_ticks(
                stream.resubscribe(),
                fanout.clone(),
            )));
            inner.fanout = Some(fanout);
            inner.upstream = Some(stream);
            return Ok(());
        }

        // Add newly needed symbols to upstream.
        if !newly_needed.is_empty()
            && let Some(upstream) = inner.upstream.as_ref()
        {
            upstream
                .add_symbols(newly_needed.iter().map(|s| s.as_str()))
                .await;
        }

        Ok(())
    }

    pub async fn unsubscribe_symbols(&self, symbols: &[String]) {
        let unique: HashSet<String> = symbols
            .iter()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        if unique.is_empty() {
            return;
        }

        let mut inner = self.inner.lock().await;

        let mut newly_unneeded: Vec<String> = Vec::new();
        for symbol in &unique {
            if let Some(count) = inner.symbol_ref_counts.get_mut(symbol)
                && *count > 0
            {
                *count -= 1;
                if *count == 0 {
                    newly_unneeded.push(symbol.clone());
                }
            }
        }

        for symbol in &newly_unneeded {
            inner.symbol_ref_counts.remove(symbol);
        }
        metrics::STREAM_SUBSCRIPTIONS_ACTIVE.set(inner.symbol_ref_counts.len() as f64);

        if let Some(upstream) = inner.upstream.as_ref()
            && !newly_unneeded.is_empty()
        {
            upstream
                .remove_symbols(newly_unneeded.iter().map(|s| s.as_str()))
                .await;
        }

        // If nothing is subscribed anywhere, close upstream to stop background tasks.
        if inner.symbol_ref_counts.is_empty() {
            if let Some(pump) = inner.pump.take() {
                pump.abort();
            }
            inner.fanout = None;
            if let Some(upstream) = inner.upstream.take() {
                upstream.close().await;
            }
        }
    }
}

/// Process-wide hub that maintains a single upstream `NewsStream`, polling
/// RSS/Atom sources on an interval (RSS/Atom has no push transport of its
/// own — see `finance_query::streaming::NewsStream`).
///
/// Multiple downstream WebSocket/GraphQL clients can subscribe/unsubscribe to
/// sources. Sources are ref-counted by URL (mirroring `StreamHub`'s symbol
/// ref-counting) so each source is only polled once upstream regardless of
/// how many clients want it.
#[derive(Clone, Default)]
pub struct FeedHub {
    inner: Arc<tokio::sync::Mutex<FeedHubInner>>,
}

#[derive(Default)]
struct FeedHubInner {
    upstream: Option<NewsStream>,
    source_ref_counts: HashMap<String, usize>,
}

impl FeedHub {
    /// Create a new empty hub.
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn resubscribe(&self) -> Option<NewsStream> {
        let inner = self.inner.lock().await;
        inner.upstream.as_ref().map(|s| s.resubscribe())
    }

    pub async fn subscribe_sources(&self, sources: &[FeedSource]) {
        if sources.is_empty() {
            return;
        }

        let mut inner = self.inner.lock().await;

        // Track which sources are newly needed upstream.
        let mut newly_needed: Vec<FeedSource> = Vec::new();
        for source in sources {
            let count = inner.source_ref_counts.entry(source.url()).or_insert(0);
            if *count == 0 {
                newly_needed.push(source.clone());
            }
            *count += 1;
        }
        metrics::FEED_SUBSCRIPTIONS_ACTIVE.set(inner.source_ref_counts.len() as f64);

        // Create upstream stream if this is the first active subscription.
        if inner.upstream.is_none() {
            let stream = NewsStream::subscribe(sources.iter().cloned()).await;
            inner.upstream = Some(stream);
            return;
        }

        // Add newly needed sources to upstream.
        if !newly_needed.is_empty()
            && let Some(upstream) = inner.upstream.as_ref()
        {
            upstream.add_sources(newly_needed).await;
        }
    }

    pub async fn unsubscribe_sources(&self, sources: &[FeedSource]) {
        if sources.is_empty() {
            return;
        }

        let mut inner = self.inner.lock().await;

        let mut newly_unneeded: Vec<FeedSource> = Vec::new();
        for source in sources {
            let url = source.url();
            if let Some(count) = inner.source_ref_counts.get_mut(&url)
                && *count > 0
            {
                *count -= 1;
                if *count == 0 {
                    newly_unneeded.push(source.clone());
                }
            }
        }

        for source in &newly_unneeded {
            inner.source_ref_counts.remove(&source.url());
        }
        metrics::FEED_SUBSCRIPTIONS_ACTIVE.set(inner.source_ref_counts.len() as f64);

        if let Some(upstream) = inner.upstream.as_ref()
            && !newly_unneeded.is_empty()
        {
            upstream.remove_sources(newly_unneeded).await;
        }

        // If nothing is subscribed anywhere, close upstream to stop the poll loop.
        if inner.source_ref_counts.is_empty()
            && let Some(upstream) = inner.upstream.take()
        {
            upstream.close().await;
        }
    }
}
