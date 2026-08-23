//! Combined House + Senate PTR disclosures provider implementation.
//!
//! Keyless — available once either `housetrades` or `senatetrades` (or
//! both) is compiled in. Serves only
//! [`FilingsProvider::fetch_congressional_trades`]; every other
//! `FilingsProvider` method stays the trait's `NotSupported` default, since
//! neither adapter has general filing/insider/holdings data.
//!
//! When both features are compiled in, both sources are queried
//! concurrently and merged; either one failing (a network error, or the
//! Senate source getting blocked by Akamai — see
//! `src/adapters/senatetrades/client.rs`) just drops that source's rows
//! rather than failing the call. Only both failing (or the single compiled
//! source failing, when just one is enabled) surfaces an error.

use super::{FilingsProvider, Operation, Provider, ProviderAdapter, ProviderCore};
use crate::error::Result;
use crate::models::filings::CongressionalTrade;

pub(crate) struct CongressTradesProvider;

impl ProviderCore for CongressTradesProvider {
    fn id(&self) -> Provider {
        Provider::CongressTrades
    }
}

#[cfg(all(feature = "housetrades", feature = "senatetrades"))]
fn merge_sources(
    house: Result<Vec<CongressionalTrade>>,
    senate: Result<Vec<CongressionalTrade>>,
) -> Result<Vec<CongressionalTrade>> {
    let mut trades = Vec::new();
    let mut last_err = None;
    for result in [house, senate] {
        match result {
            Ok(mut rows) => trades.append(&mut rows),
            Err(e) => {
                tracing::warn!("congressional trades source unavailable: {e}");
                last_err = Some(e);
            }
        }
    }
    if trades.is_empty()
        && let Some(e) = last_err
    {
        return Err(e);
    }
    trades.sort_by(|a, b| b.transaction_date.cmp(&a.transaction_date));
    Ok(trades)
}

#[async_trait::async_trait]
impl FilingsProvider for CongressTradesProvider {
    async fn fetch_filings(
        &self,
        _symbol: &str,
    ) -> Result<crate::models::filings::ProviderFilings> {
        Err(self.not_supported(Operation::Filings))
    }

    async fn fetch_congressional_trades(&self, symbol: &str) -> Result<Vec<CongressionalTrade>> {
        #[cfg(all(feature = "housetrades", feature = "senatetrades"))]
        {
            let (house, senate) = tokio::join!(
                crate::adapters::housetrades::fetch_congressional_trades_response(symbol),
                crate::adapters::senatetrades::fetch_congressional_trades_response(symbol),
            );
            return merge_sources(house, senate);
        }
        #[cfg(all(feature = "housetrades", not(feature = "senatetrades")))]
        {
            return crate::adapters::housetrades::fetch_congressional_trades_response(symbol).await;
        }
        #[cfg(all(feature = "senatetrades", not(feature = "housetrades")))]
        {
            return crate::adapters::senatetrades::fetch_congressional_trades_response(symbol)
                .await;
        }
    }
}

#[async_trait::async_trait]
impl ProviderAdapter for CongressTradesProvider {
    fn as_filings(&self) -> Option<&dyn FilingsProvider> {
        Some(self)
    }
}
