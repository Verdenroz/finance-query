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

/// Constructs the adapter for one built-in provider id without initialising it.
fn adapter_for(
    id: Provider,
    config: &ClientConfig,
    yahoo_session: &mut Option<Arc<LazySession>>,
) -> Result<Arc<dyn ProviderAdapter>> {
    Ok(match id {
        Provider::Yahoo => {
            let session = Arc::new(LazySession::new(config.clone()));
            *yahoo_session = Some(Arc::clone(&session));
            Arc::new(yahoo::YahooProvider::new(session))
        }
        #[cfg(feature = "polygon")]
        Provider::Polygon => Arc::new(polygon::PolygonProvider),
        #[cfg(feature = "fmp")]
        Provider::Fmp => Arc::new(fmp::FmpProvider),
        #[cfg(feature = "alphavantage")]
        Provider::AlphaVantage => Arc::new(alphavantage::AlphaVantageProvider),
        #[cfg(feature = "crypto")]
        Provider::CoinGecko => Arc::new(coingecko::CoinGeckoProvider),
        #[cfg(feature = "fred")]
        Provider::Fred => Arc::new(fred::FredProvider),
        #[cfg(feature = "worldbank")]
        Provider::WorldBank => Arc::new(worldbank::WorldBankProvider),
        #[cfg(feature = "fiscaldata")]
        Provider::FiscalData => Arc::new(fiscaldata::FiscalDataProvider),
        #[cfg(feature = "bls")]
        Provider::Bls => Arc::new(bls::BlsProvider),
        #[cfg(feature = "frankfurter")]
        Provider::Frankfurter => Arc::new(frankfurter::FrankfurterProvider),
        #[cfg(feature = "binance")]
        Provider::Binance => Arc::new(binance::BinanceProvider),
        #[cfg(feature = "kraken")]
        Provider::Kraken => Arc::new(kraken::KrakenProvider),
        #[cfg(feature = "finra")]
        Provider::Finra => Arc::new(finra::FinraProvider),
        #[cfg(feature = "defi")]
        Provider::DefiLlama => Arc::new(defillama::DefiLlamaProvider),
        #[cfg(feature = "gdelt")]
        Provider::Gdelt => Arc::new(gdelt::GdeltProvider),
        #[cfg(feature = "cftc")]
        Provider::Cftc => Arc::new(cftc::CftcProvider),
        #[cfg(feature = "nasdaq")]
        Provider::Nasdaq => Arc::new(nasdaq::NasdaqProvider),
        #[cfg(feature = "wikipedia")]
        Provider::Wikipedia => Arc::new(wikipedia::WikipediaProvider),
        #[cfg(any(feature = "housetrades", feature = "senatetrades"))]
        Provider::CongressTrades => Arc::new(congresstrades::CongressTradesProvider),
        Provider::Edgar => Arc::new(edgar::EdgarProvider),
        Provider::LocalMarketCalendar => Arc::new(market_calendar::LocalMarketCalendarProvider),
        Provider::LocalExchange => Arc::new(local_exchanges::LocalExchangeProvider),
        Provider::Custom(id) => {
            return Err(FinanceError::InvalidParameter {
                param: "provider".to_string(),
                reason: format!(
                    "no adapter registered for custom provider `{}`",
                    id.as_str()
                ),
            });
        }
        other => {
            return Err(FinanceError::InvalidParameter {
                param: "provider".to_string(),
                reason: format!("no adapter registered for provider `{other}`"),
            });
        }
    })
}

/// Routes for a default-built `Ticker`/`Tickers`: sequential, with FRED serving
/// the release dates that `calendar()` merges in. FRED is registered without
/// `initialize`, so a missing key only skips those events.
pub(crate) fn default_equity_routes() -> Routes {
    let routes = Routes::new(Fetch::Sequential);
    #[cfg(feature = "fred")]
    let routes = routes.route(Capability::ECONOMIC, [Provider::Fred]);
    routes
}

/// A default-routed set over an already-open Yahoo session.
pub(crate) fn shared_yahoo_set(session: Arc<LazySession>) -> ProviderSet {
    #[allow(unused_mut)]
    let mut providers: Vec<Arc<dyn ProviderAdapter>> =
        vec![Arc::new(yahoo::YahooProvider::new(Arc::clone(&session)))];
    #[cfg(feature = "fred")]
    providers.push(Arc::new(fred::FredProvider));
    ProviderSet::new(providers, default_equity_routes()).with_yahoo_session(Some(session))
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
    #[cfg(feature = "fred")]
    if !ids.contains(&Provider::Fred) {
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
