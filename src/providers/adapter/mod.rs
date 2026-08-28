//! Provider adapter traits — one trait per [`Capability`].
//!
//! A provider bridge implements [`ProviderCore`] plus the capability traits it
//! actually serves, then overrides the matching `as_*` accessors on
//! [`ProviderAdapter`] to return `Some(self)`. `ProviderAdapter::capabilities`
//! is *derived* from those accessors, so a provider's declared capability set
//! can never drift from what it implements.
//!
//! Within a capability trait, the primary operation is required (implementing
//! the trait without it is unrepresentable) while secondary operations default
//! to [`FinanceError::NotSupported`]; dispatch ([`super::ProviderSet::fetch`])
//! skips `NotSupported` and falls through to the next routed provider, so
//! ragged per-operation coverage degrades gracefully.

use super::{Operation, Provider};
use crate::error::FinanceError;

/// Identity shared by every capability trait: the provider id and the
/// `NotSupported` error constructor used by default method bodies.
pub trait ProviderCore: Send + Sync {
    /// This provider's identity, used for routing and health reporting.
    fn id(&self) -> Provider;

    /// The error a default method body returns for an unimplemented operation.
    fn not_supported(&self, operation: Operation) -> FinanceError {
        operation.not_supported(self.id())
    }
}

mod dispatch;
mod equity;
mod markets;

pub use dispatch::ProviderAdapter;
pub use equity::{
    ChartProvider, CorporateProvider, FilingsProvider, FundamentalsProvider, OptionsProvider,
    QuoteProvider,
};
#[cfg(any(
    feature = "binance",
    feature = "crypto",
    feature = "defi",
    feature = "kraken"
))]
pub use markets::CryptoProvider;
#[cfg(any(
    feature = "alphavantage",
    feature = "bls",
    feature = "fiscaldata",
    feature = "fred",
    feature = "worldbank"
))]
pub use markets::EconomicProvider;
#[cfg(any(feature = "frankfurter", feature = "gdelt"))]
pub use markets::ForexProvider;
pub use markets::{
    CalendarProvider, CommoditiesProvider, DiscoveryProvider, FuturesProvider, IndicesProvider,
    MarketProvider,
};
