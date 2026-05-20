//! Domain-specific query handles — non-equity asset classes.
//!
//! These types are constructable only through [`Providers`](crate::Providers)
//! factory methods — they share the same provider connections and configuration.
//!
//! Each handle maps to a [`Capability`](crate::Capability) and routes through
//! the multi-provider dispatch system, making multi-source aggregation
//! a first-class concept rather than an opt-in on Ticker/Tickers.

// ── Macros ──────────────────────────────────────────────────────────

/// Generate a single-field domain handle struct with internal constructor
/// and a string accessor. Callers add methods manually.
macro_rules! domain_handle {
    ($(#[$meta:meta])* pub struct $name:ident { $field:ident, $accessor:ident }) => {
        $(#[$meta])*
        pub struct $name {
            $field: std::sync::Arc<str>,
            providers: std::sync::Arc<crate::providers::ProviderSet>,
        }

        impl $name {
            pub(crate) fn with_providers(
                $field: std::sync::Arc<str>,
                providers: std::sync::Arc<crate::providers::ProviderSet>,
            ) -> Self {
                Self { $field, providers }
            }

            /// The handle's identifier string.
            pub fn $accessor(&self) -> &str {
                &self.$field
            }
        }
    };
}

/// Fetch via the provider dispatch — single symbol field, no extra args.
/// Use inside a method body.
macro_rules! fetch_via {
    ($self:expr, $field:ident, $cap:ident, $fetch:ident, $ret:ty) => {{
        let __sym = $self.$field.clone();
        $self
            .providers
            .fetch(crate::providers::Capability::$cap, move |p| {
                let __s = __sym.clone();
                let p = p.clone();
                async move { p.$fetch(&__s).await }
            })
            .await
    }};
}

/// Fetch with one extra string argument (e.g. `vs_currency` for crypto).
#[allow(unused_macros)]
macro_rules! fetch_via_with {
    ($self:expr, $field:ident, $cap:ident, $fetch:ident, $arg:expr, $ret:ty) => {{
        let __sym = $self.$field.clone();
        let __arg = ($arg).to_string();
        $self
            .providers
            .fetch(crate::providers::Capability::$cap, move |p| {
                let __s = __sym.clone();
                let __a = __arg.clone();
                let p = p.clone();
                async move { p.$fetch(&__s, &__a).await }
            })
            .await
    }};
}

// ── Modules ─────────────────────────────────────────────────────────

#[cfg(any(feature = "fmp", feature = "alphavantage"))]
pub(crate) mod commodities;
#[cfg(any(
    feature = "alphavantage",
    feature = "crypto",
    feature = "fmp",
    feature = "polygon"
))]
pub(crate) mod crypto;
#[cfg(any(feature = "fred", feature = "alphavantage", feature = "polygon"))]
pub(crate) mod economic;
pub(crate) mod filings;
#[cfg(any(feature = "polygon", feature = "fmp", feature = "alphavantage"))]
pub(crate) mod forex;
#[cfg(feature = "polygon")]
pub(crate) mod futures;
#[cfg(any(feature = "polygon", feature = "fmp"))]
pub(crate) mod indices;

// ── Re-exports ──────────────────────────────────────────────────────

#[cfg(any(feature = "fmp", feature = "alphavantage"))]
pub use commodities::Commodity;
#[cfg(any(
    feature = "alphavantage",
    feature = "crypto",
    feature = "fmp",
    feature = "polygon"
))]
pub use crypto::CryptoCoin;
#[cfg(any(feature = "fred", feature = "alphavantage", feature = "polygon"))]
pub use economic::EconomicIndicator;
pub use filings::Filings;
#[cfg(any(feature = "polygon", feature = "fmp", feature = "alphavantage"))]
pub use forex::ForexPair;
#[cfg(feature = "polygon")]
pub use futures::FuturesContract;
#[cfg(any(feature = "polygon", feature = "fmp"))]
pub use indices::Index;
