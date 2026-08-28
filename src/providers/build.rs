//! Construction of a [`ProviderSet`] from a list of [`Provider`] ids.

use std::sync::Arc;

use super::*;
use crate::adapters::yahoo::client::{ClientConfig, YahooClient};
use crate::error::{FinanceError, Result};

/// Keyed on identity, not `capabilities().contains(FILINGS)`: a provider may
/// advertise FILINGS only to reach a secondary op (Alpha Vantage does, for
/// insider trades), which would otherwise suppress the real filings source.
fn needs_edgar_injection(ids: &[Provider]) -> bool {
    !ids.contains(&Provider::Edgar)
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
        let adapter: Arc<dyn ProviderAdapter> = match id {
            Provider::Yahoo => {
                let yp = YahooProvider::new(config).await?;
                yahoo_client = Some(yp.client_arc());
                Arc::new(yp)
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
                    reason: format!("no adapter registered for custom provider `{id}`"),
                });
            }
        };
        adapter.initialize().await?;
        providers.push(adapter);
    }
    if needs_edgar_injection(ids) {
        providers.push(Arc::new(edgar::EdgarProvider));
    }
    Ok(ProviderSet::new(providers, yahoo_client, routes))
}

#[cfg(test)]
mod tests {
    use super::*;

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
