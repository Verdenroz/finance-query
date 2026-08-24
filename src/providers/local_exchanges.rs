//! A static table of major global exchanges, computed locally rather than
//! fetched. Unlike [`market_calendar`](super::market_calendar)'s holiday
//! rules, MIC codes aren't derivable from a formula — this table needs
//! occasional manual maintenance as venues merge or rebrand.

use super::{DiscoveryProvider, Operation, ProviderAdapter, ProviderCore};
use crate::error::Result;
use crate::models::discovery::reference::{ExchangeInfo, SymbolMatch};

pub(crate) struct LocalExchangeProvider;

impl ProviderCore for LocalExchangeProvider {
    fn id(&self) -> super::Provider {
        super::Provider::LocalExchange
    }
}

#[async_trait::async_trait]
impl DiscoveryProvider for LocalExchangeProvider {
    async fn fetch_symbol_search(&self, _query: &str, _limit: u32) -> Result<Vec<SymbolMatch>> {
        Err(self.not_supported(Operation::SymbolSearch))
    }

    async fn fetch_exchanges(&self) -> Result<Vec<ExchangeInfo>> {
        Ok(EXCHANGES.iter().map(to_exchange_info).collect())
    }
}

#[async_trait::async_trait]
impl ProviderAdapter for LocalExchangeProvider {
    fn as_discovery(&self) -> Option<&dyn DiscoveryProvider> {
        Some(self)
    }
}

/// `(name, mic, operating_mic, locale, url)`. `asset_class` is always
/// `"stocks"` and `exchange_type` is always `"exchange"` except OTC Markets
/// (`"TRF"`), so both stay out of the table rather than repeating per row.
const EXCHANGES: &[(&str, &str, &str, &str, &str)] = &[
    (
        "New York Stock Exchange",
        "XNYS",
        "XNYS",
        "us",
        "https://www.nyse.com",
    ),
    ("Nasdaq", "XNAS", "XNAS", "us", "https://www.nasdaq.com"),
    (
        "NYSE Arca",
        "ARCX",
        "XNYS",
        "us",
        "https://www.nyse.com/markets/nyse-arca",
    ),
    (
        "Cboe BZX Exchange",
        "BATS",
        "XCBO",
        "us",
        "https://www.cboe.com",
    ),
    ("IEX", "IEXG", "IEXG", "us", "https://iextrading.com"),
    (
        "OTC Markets",
        "OTCM",
        "OTCM",
        "us",
        "https://www.otcmarkets.com",
    ),
    (
        "Toronto Stock Exchange",
        "XTSE",
        "XTSE",
        "ca",
        "https://www.tsx.com",
    ),
    (
        "TSX Venture Exchange",
        "XTSX",
        "XTSE",
        "ca",
        "https://www.tsx.com/tsxv",
    ),
    (
        "London Stock Exchange",
        "XLON",
        "XLON",
        "gb",
        "https://www.londonstockexchange.com",
    ),
    (
        "Euronext Paris",
        "XPAR",
        "XPAR",
        "fr",
        "https://www.euronext.com",
    ),
    (
        "Deutsche Börse Xetra",
        "XETR",
        "XETR",
        "de",
        "https://www.xetra.com",
    ),
    (
        "SIX Swiss Exchange",
        "XSWX",
        "XSWX",
        "ch",
        "https://www.six-group.com",
    ),
    (
        "Tokyo Stock Exchange",
        "XTKS",
        "XTKS",
        "jp",
        "https://www.jpx.co.jp",
    ),
    (
        "Hong Kong Exchange",
        "XHKG",
        "XHKG",
        "hk",
        "https://www.hkex.com.hk",
    ),
    (
        "Shanghai Stock Exchange",
        "XSHG",
        "XSHG",
        "cn",
        "http://english.sse.com.cn",
    ),
    (
        "Shenzhen Stock Exchange",
        "XSHE",
        "XSHE",
        "cn",
        "http://www.szse.cn",
    ),
    (
        "Australian Securities Exchange",
        "XASX",
        "XASX",
        "au",
        "https://www.asx.com.au",
    ),
    (
        "National Stock Exchange of India",
        "XNSE",
        "XNSE",
        "in",
        "https://www.nseindia.com",
    ),
];

fn to_exchange_info(
    &(name, mic, operating_mic, locale, url): &(&str, &str, &str, &str, &str),
) -> ExchangeInfo {
    ExchangeInfo {
        id: None,
        name: Some(name.to_string()),
        mic: Some(mic.to_string()),
        operating_mic: Some(operating_mic.to_string()),
        asset_class: Some("stocks".to_string()),
        locale: Some(locale.to_string()),
        exchange_type: Some(if mic == "OTCM" { "TRF" } else { "exchange" }.to_string()),
        url: Some(url.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn returns_the_full_static_table() {
        let out = LocalExchangeProvider.fetch_exchanges().await.unwrap();
        assert_eq!(out.len(), EXCHANGES.len());
    }

    #[tokio::test]
    async fn nyse_and_nasdaq_carry_their_known_mic_codes() {
        let out = LocalExchangeProvider.fetch_exchanges().await.unwrap();
        let nyse = out
            .iter()
            .find(|e| e.name.as_deref() == Some("New York Stock Exchange"))
            .unwrap();
        assert_eq!(nyse.mic.as_deref(), Some("XNYS"));
        let nasdaq = out
            .iter()
            .find(|e| e.name.as_deref() == Some("Nasdaq"))
            .unwrap();
        assert_eq!(nasdaq.mic.as_deref(), Some("XNAS"));
    }

    #[tokio::test]
    async fn otc_markets_is_the_only_trf_exchange_type() {
        let out = LocalExchangeProvider.fetch_exchanges().await.unwrap();
        let otc = out
            .iter()
            .find(|e| e.name.as_deref() == Some("OTC Markets"))
            .unwrap();
        assert_eq!(otc.exchange_type.as_deref(), Some("TRF"));
        assert_eq!(
            out.iter()
                .filter(|e| e.exchange_type.as_deref() == Some("exchange"))
                .count(),
            EXCHANGES.len() - 1
        );
    }

    #[tokio::test]
    async fn symbol_search_is_not_supported() {
        let err = LocalExchangeProvider
            .fetch_symbol_search("aapl", 5)
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            crate::error::FinanceError::NotSupported { .. }
        ));
    }
}
