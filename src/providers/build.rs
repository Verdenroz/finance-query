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

/// Constructs the adapter for one built-in provider id without initialising it.
fn adapter_for(
    id: Provider,
    config: &ClientConfig,
    yahoo_session: &mut Option<Arc<LazySession>>,
) -> Result<Arc<dyn ProviderAdapter>> {
    if let Some((_, factory)) = REGISTRY.iter().find(|(provider, _)| *provider == id) {
        return factory(config, yahoo_session);
    }
    Err(FinanceError::InvalidParameter {
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

pub(crate) async fn build_providers(
    ids: &[Provider],
    extra: Vec<Arc<dyn ProviderAdapter>>,
    config: &ClientConfig,
    routes: Routes,
) -> Result<ProviderSet> {
    let mut providers: Vec<Arc<dyn ProviderAdapter>> = Vec::new();
    let mut yahoo_session: Option<Arc<LazySession>> = None;
    for &id in ids {
        let adapter = adapter_for(id, config, &mut yahoo_session)?;
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
                adapter_for(provider, &config, &mut session).is_ok(),
                "{provider} has no arm in adapter_for"
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
