//! Construction of a [`ProviderSet`] from a list of [`Provider`] ids.

use std::sync::Arc;

use super::*;
use crate::adapters::yahoo::client::ClientConfig;
use crate::adapters::yahoo::session::LazySession;
use crate::error::{FinanceError, Result};

/// Keyed on identity, not `capabilities().contains(FILINGS)`: a provider may
/// advertise FILINGS only to reach a secondary op (Alpha Vantage does, for
/// insider trades), which would otherwise suppress the real filings source.
fn needs_edgar_injection(ids: &[Provider]) -> bool {
    !ids.contains(&Provider::Edgar)
}

type Factory = fn(&ClientConfig, &mut Option<Arc<LazySession>>) -> Result<Arc<dyn ProviderAdapter>>;

macro_rules! plain {
    ($id:expr, $provider:expr) => {
        ($id, |_, _| Ok(Arc::new($provider)))
    };
}

/// Yahoo shares its lazy session with the set, so factories receive the slot.
/// A provider appears at most once; `registry_has_no_duplicate_ids` enforces it.
const REGISTRY: &[(Provider, Factory)] = &[
    (Provider::Yahoo, |config, yahoo_session| {
        let session = Arc::new(LazySession::new(config.clone()));
        *yahoo_session = Some(Arc::clone(&session));
        Ok(Arc::new(yahoo::YahooProvider::new(session)))
    }),
    #[cfg(feature = "polygon")]
    plain!(Provider::Polygon, polygon::PolygonProvider),
    #[cfg(feature = "fmp")]
    plain!(Provider::Fmp, fmp::FmpProvider),
    #[cfg(feature = "alphavantage")]
    plain!(Provider::AlphaVantage, alphavantage::AlphaVantageProvider),
    #[cfg(feature = "crypto")]
    plain!(Provider::CoinGecko, coingecko::CoinGeckoProvider),
    #[cfg(feature = "fred")]
    plain!(Provider::Fred, fred::FredProvider),
    #[cfg(feature = "worldbank")]
    plain!(Provider::WorldBank, worldbank::WorldBankProvider),
    #[cfg(feature = "fiscaldata")]
    plain!(Provider::FiscalData, fiscaldata::FiscalDataProvider),
    #[cfg(feature = "bls")]
    plain!(Provider::Bls, bls::BlsProvider),
    #[cfg(feature = "frankfurter")]
    plain!(Provider::Frankfurter, frankfurter::FrankfurterProvider),
    #[cfg(feature = "binance")]
    plain!(Provider::Binance, binance::BinanceProvider),
    #[cfg(feature = "kraken")]
    plain!(Provider::Kraken, kraken::KrakenProvider),
    #[cfg(feature = "finra")]
    plain!(Provider::Finra, finra::FinraProvider),
    #[cfg(feature = "defi")]
    plain!(Provider::DefiLlama, defillama::DefiLlamaProvider),
    #[cfg(feature = "gdelt")]
    plain!(Provider::Gdelt, gdelt::GdeltProvider),
    #[cfg(feature = "cftc")]
    plain!(Provider::Cftc, cftc::CftcProvider),
    #[cfg(feature = "nasdaq")]
    plain!(Provider::Nasdaq, nasdaq::NasdaqProvider),
    #[cfg(feature = "wikipedia")]
    plain!(Provider::Wikipedia, wikipedia::WikipediaProvider),
    #[cfg(any(feature = "housetrades", feature = "senatetrades"))]
    plain!(
        Provider::CongressTrades,
        congresstrades::CongressTradesProvider
    ),
    plain!(Provider::Edgar, edgar::EdgarProvider),
    plain!(
        Provider::LocalMarketCalendar,
        market_calendar::LocalMarketCalendarProvider
    ),
    plain!(
        Provider::LocalExchange,
        local_exchanges::LocalExchangeProvider
    ),
];

fn factory_for(id: Provider) -> Result<Factory> {
    REGISTRY
        .iter()
        .find_map(|(provider, factory)| (*provider == id).then_some(*factory))
        .ok_or_else(|| FinanceError::InvalidParameter {
            param: "provider".to_string(),
            reason: match id {
                Provider::Custom(id) => {
                    format!(
                        "no adapter registered for custom provider `{}`",
                        id.as_str()
                    )
                }
                other => format!("no adapter registered for provider `{other}`"),
            },
        })
}

/// A set over an already-open Yahoo session. FRED rides along uninitialized so
/// `calendar()` can merge release dates without requiring a key.
pub(crate) fn shared_yahoo_set(session: Arc<LazySession>) -> ProviderSet {
    #[allow(unused_mut)]
    let mut providers: Vec<Arc<dyn ProviderAdapter>> =
        vec![Arc::new(yahoo::YahooProvider::new(Arc::clone(&session)))];
    #[cfg(feature = "fred")]
    providers.push(Arc::new(fred::FredProvider));
    ProviderSet::new(providers, Routes::new(Fetch::Sequential)).with_yahoo_session(Some(session))
}

#[cfg(any(
    feature = "fmp",
    feature = "polygon",
    feature = "alphavantage",
    feature = "fred"
))]
/// A set holding only `id`, routed for `cap`. The adapter is not initialized,
/// so a missing key surfaces from the call that needs it.
pub(crate) fn solo_set(id: Provider, cap: Capability) -> Result<ProviderSet> {
    let adapter = adapter_for(id, &ClientConfig::default(), &mut None)?;
    Ok(ProviderSet::new(
        vec![adapter],
        Routes::new(Fetch::Sequential).route(cap, [id]),
    ))
}

pub(crate) async fn build_providers(
    ids: &[Provider],
    extra: Vec<Arc<dyn ProviderAdapter>>,
    config: &ClientConfig,
    routes: Routes,
) -> Result<ProviderSet> {
    let mut providers: Vec<Arc<dyn ProviderAdapter>> = Vec::new();
    let mut yahoo_session: Option<Arc<LazySession>> = None;
    for &id in ids {
        let adapter = factory_for(id)?(config, &mut yahoo_session)?;
        adapter.initialize().await?;
        providers.push(adapter);
    }
    for adapter in extra {
        adapter.initialize().await?;
        providers.push(adapter);
    }
    if needs_edgar_injection(ids) {
        providers.push(Arc::new(edgar::EdgarProvider));
    }
    // Uninitialized and unrouted: `calendar()` reaches it for release dates
    // whenever the ECONOMIC route cannot serve them, and a missing key only
    // skips those events.
    #[cfg(feature = "fred")]
    if !providers.iter().any(|p| p.id() == Provider::Fred) {
        providers.push(Arc::new(fred::FredProvider));
    }
    for routed in routes.map.values() {
        for provider in &routed.providers {
            if !providers.iter().any(|p| p.id() == *provider) {
                return Err(FinanceError::ProviderNotRegistered {
                    provider: *provider,
                });
            }
        }
    }
    Ok(ProviderSet::new(providers, routes).with_yahoo_session(yahoo_session))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_built_in_provider_has_an_adapter() {
        let config = ClientConfig::default();
        for provider in Provider::all() {
            let mut session = None;
            assert!(
                factory_for(provider).is_ok_and(|build| build(&config, &mut session).is_ok()),
                "{provider} is not in REGISTRY"
            );
        }
    }

    #[test]
    fn registry_has_no_duplicate_ids() {
        for (i, (id, _)) in REGISTRY.iter().enumerate() {
            assert!(
                REGISTRY[..i].iter().all(|(earlier, _)| earlier != id),
                "{id} is registered twice"
            );
        }
    }

    #[test]
    fn edgar_is_injected_unless_configured_explicitly() {
        assert!(needs_edgar_injection(&[Provider::Yahoo]));
        assert!(!needs_edgar_injection(&[Provider::Yahoo, Provider::Edgar]));
    }

    /// Alpha Vantage advertises FILINGS for insider trades only, so a
    /// capability-based check would drop EDGAR and break `filings()`.
    #[test]
    #[cfg(feature = "alphavantage")]
    fn a_filings_advertising_provider_does_not_suppress_edgar() {
        assert!(
            ProviderAdapter::capabilities(&alphavantage::AlphaVantageProvider)
                .contains(Capability::FILINGS)
        );
        assert!(needs_edgar_injection(&[Provider::AlphaVantage]));
    }
}
