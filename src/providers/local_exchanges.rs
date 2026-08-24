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
        Ok(EXCHANGES.iter().map(|e| e.to_exchange_info()).collect())
    }
}

#[async_trait::async_trait]
impl ProviderAdapter for LocalExchangeProvider {
    fn as_discovery(&self) -> Option<&dyn DiscoveryProvider> {
        Some(self)
    }
}

struct StaticExchange {
    name: &'static str,
    mic: &'static str,
    operating_mic: &'static str,
    asset_class: &'static str,
    locale: &'static str,
    exchange_type: &'static str,
    url: &'static str,
}

impl StaticExchange {
    fn to_exchange_info(&self) -> ExchangeInfo {
        ExchangeInfo {
            id: None,
            name: Some(self.name.to_string()),
            mic: Some(self.mic.to_string()),
            operating_mic: Some(self.operating_mic.to_string()),
            asset_class: Some(self.asset_class.to_string()),
            locale: Some(self.locale.to_string()),
            exchange_type: Some(self.exchange_type.to_string()),
            url: Some(self.url.to_string()),
        }
    }
}

const EXCHANGES: &[StaticExchange] = &[
    StaticExchange {
        name: "New York Stock Exchange",
        mic: "XNYS",
        operating_mic: "XNYS",
        asset_class: "stocks",
        locale: "us",
        exchange_type: "exchange",
        url: "https://www.nyse.com",
    },
    StaticExchange {
        name: "Nasdaq",
        mic: "XNAS",
        operating_mic: "XNAS",
        asset_class: "stocks",
        locale: "us",
        exchange_type: "exchange",
        url: "https://www.nasdaq.com",
    },
    StaticExchange {
        name: "NYSE Arca",
        mic: "ARCX",
        operating_mic: "XNYS",
        asset_class: "stocks",
        locale: "us",
        exchange_type: "exchange",
        url: "https://www.nyse.com/markets/nyse-arca",
    },
    StaticExchange {
        name: "Cboe BZX Exchange",
        mic: "BATS",
        operating_mic: "XCBO",
        asset_class: "stocks",
        locale: "us",
        exchange_type: "exchange",
        url: "https://www.cboe.com",
    },
    StaticExchange {
        name: "IEX",
        mic: "IEXG",
        operating_mic: "IEXG",
        asset_class: "stocks",
        locale: "us",
        exchange_type: "exchange",
        url: "https://iextrading.com",
    },
    StaticExchange {
        name: "OTC Markets",
        mic: "OTCM",
        operating_mic: "OTCM",
        asset_class: "stocks",
        locale: "us",
        exchange_type: "TRF",
        url: "https://www.otcmarkets.com",
    },
    StaticExchange {
        name: "Toronto Stock Exchange",
        mic: "XTSE",
        operating_mic: "XTSE",
        asset_class: "stocks",
        locale: "ca",
        exchange_type: "exchange",
        url: "https://www.tsx.com",
    },
    StaticExchange {
        name: "TSX Venture Exchange",
        mic: "XTSX",
        operating_mic: "XTSE",
        asset_class: "stocks",
        locale: "ca",
        exchange_type: "exchange",
        url: "https://www.tsx.com/tsxv",
    },
    StaticExchange {
        name: "London Stock Exchange",
        mic: "XLON",
        operating_mic: "XLON",
        asset_class: "stocks",
        locale: "gb",
        exchange_type: "exchange",
        url: "https://www.londonstockexchange.com",
    },
    StaticExchange {
        name: "Euronext Paris",
        mic: "XPAR",
        operating_mic: "XPAR",
        asset_class: "stocks",
        locale: "fr",
        exchange_type: "exchange",
        url: "https://www.euronext.com",
    },
    StaticExchange {
        name: "Deutsche Börse Xetra",
        mic: "XETR",
        operating_mic: "XETR",
        asset_class: "stocks",
        locale: "de",
        exchange_type: "exchange",
        url: "https://www.xetra.com",
    },
    StaticExchange {
        name: "SIX Swiss Exchange",
        mic: "XSWX",
        operating_mic: "XSWX",
        asset_class: "stocks",
        locale: "ch",
        exchange_type: "exchange",
        url: "https://www.six-group.com",
    },
    StaticExchange {
        name: "Tokyo Stock Exchange",
        mic: "XTKS",
        operating_mic: "XTKS",
        asset_class: "stocks",
        locale: "jp",
        exchange_type: "exchange",
        url: "https://www.jpx.co.jp",
    },
    StaticExchange {
        name: "Hong Kong Exchange",
        mic: "XHKG",
        operating_mic: "XHKG",
        asset_class: "stocks",
        locale: "hk",
        exchange_type: "exchange",
        url: "https://www.hkex.com.hk",
    },
    StaticExchange {
        name: "Shanghai Stock Exchange",
        mic: "XSHG",
        operating_mic: "XSHG",
        asset_class: "stocks",
        locale: "cn",
        exchange_type: "exchange",
        url: "http://english.sse.com.cn",
    },
    StaticExchange {
        name: "Shenzhen Stock Exchange",
        mic: "XSHE",
        operating_mic: "XSHE",
        asset_class: "stocks",
        locale: "cn",
        exchange_type: "exchange",
        url: "http://www.szse.cn",
    },
    StaticExchange {
        name: "Australian Securities Exchange",
        mic: "XASX",
        operating_mic: "XASX",
        asset_class: "stocks",
        locale: "au",
        exchange_type: "exchange",
        url: "https://www.asx.com.au",
    },
    StaticExchange {
        name: "National Stock Exchange of India",
        mic: "XNSE",
        operating_mic: "XNSE",
        asset_class: "stocks",
        locale: "in",
        exchange_type: "exchange",
        url: "https://www.nseindia.com",
    },
];

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
