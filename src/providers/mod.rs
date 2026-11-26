//! Multi-provider financial data aggregation.

pub(crate) mod adapter;
pub mod config;
pub(crate) mod health;
pub(crate) mod retry;

#[cfg(test)]
pub(crate) mod mock;

#[cfg(feature = "alphavantage")]
pub(crate) mod alphavantage;
#[cfg(feature = "binance")]
pub(crate) mod binance;
#[cfg(feature = "bls")]
pub(crate) mod bls;
#[cfg(feature = "cftc")]
pub(crate) mod cftc;
#[cfg(feature = "crypto")]
pub(crate) mod coingecko;
#[cfg(any(feature = "housetrades", feature = "senatetrades"))]
pub(crate) mod congresstrades;
#[cfg(feature = "defi")]
pub(crate) mod defillama;
pub(crate) mod edgar;
#[cfg(feature = "finra")]
pub(crate) mod finra;
#[cfg(feature = "fiscaldata")]
pub(crate) mod fiscaldata;
#[cfg(feature = "fmp")]
pub(crate) mod fmp;
#[cfg(feature = "frankfurter")]
pub(crate) mod frankfurter;
#[cfg(feature = "fred")]
pub(crate) mod fred;
#[cfg(feature = "gdelt")]
pub(crate) mod gdelt;
#[cfg(feature = "kraken")]
pub(crate) mod kraken;
pub(crate) mod local_exchanges;
pub(crate) mod market_calendar;
#[cfg(feature = "nasdaq")]
pub(crate) mod nasdaq;
#[cfg(feature = "polygon")]
pub(crate) mod polygon;
pub(crate) mod types;
#[cfg(feature = "wikipedia")]
pub(crate) mod wikipedia;
#[cfg(feature = "worldbank")]
pub(crate) mod worldbank;
pub(crate) mod yahoo;
mod yahoo_ttm;

pub use adapter::CryptoProvider;
pub use adapter::EconomicProvider;
pub use adapter::ForexProvider;
/// Not part of the stable public API — see [`ProviderAdapter`].
#[doc(hidden)]
pub use adapter::{
    CalendarProvider, ChartProvider, CommoditiesProvider, CorporateProvider, DiscoveryProvider,
    FilingsProvider, FundamentalsProvider, FuturesProvider, IndicesProvider, MarketProvider,
    OptionsProvider, ProviderAdapter, ProviderCore, QuoteProvider,
};
pub use health::ProviderHealth;
pub use retry::RetryPolicy;

mod build;
mod convert;
mod operation;
mod page;
mod routes;
mod set;
mod stock_bars;
pub(crate) mod stock_listing;

pub(crate) use build::build_providers;
#[allow(unused_imports)] // each is used only by a feature-gated provider bridge
pub(crate) use convert::{build_financial_statement, build_options, range_to_dates};
pub use finance_query_types::Capability;
pub use finance_query_types::{CustomId, Provider};
pub use operation::Operation;
pub use page::{PageCursor, ProviderPage};
pub use routes::{Fetch, Routes};
pub use set::ProviderSet;
pub use stock_bars::StockBarsRequest;
pub use stock_listing::StockListingRequest;

#[cfg(test)]
mod tests;
