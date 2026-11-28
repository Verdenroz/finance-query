//! Compile-time format type parameters for `FormattedValue`-bearing structs.
//!
//! Structs like [`Quote`](super::quote::Quote) carry a format type parameter `F: Format`
//! that controls what type each numeric field holds:
//!
//! | `F`      | `F::Value<f64>`         | Access pattern            |
//! |----------|-------------------------|---------------------------|
//! | [`Both`] | `FormattedValue<f64>`   | `.raw` / `.fmt` / `.long_fmt` (**default**) |
//! | [`Raw`]  | `f64`                   | direct — no unwrapping |
//! | [`Pretty`] | `String`              | human-readable string     |
//!
//! # Quick start
//!
//! ```no_run
//! use finance_query::{Ticker, format};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // quote() takes its format from the annotation; a bare `Quote` means `Quote<Both>`
//! let quote: finance_query::Quote<format::Raw> = Ticker::new("AAPL").await?.quote().await?;
//! let price: Option<f64> = quote.regular_market_price;
//! # Ok(())
//! # }
//! ```

use crate::models::quote::FormattedValue;

mod sealed {
    /// Blocks external crates from implementing [`Format`](super::Format);
    /// only this module can name `Sealed`, so it can only be satisfied here.
    pub trait Sealed {}
}

/// Marker trait that controls how [`FormattedValue`](super::quote::FormattedValue) fields are typed.
///
/// Sealed — only [`Both`], [`Raw`], and [`Pretty`] implement this trait.
pub trait Format: sealed::Sealed + Clone + std::fmt::Debug + PartialEq + 'static {
    /// The concrete field type for a numeric value of type `T`.
    type Value<T: Clone + std::fmt::Debug + PartialEq + serde::Serialize + for<'de> serde::Deserialize<'de>>: Clone
        + std::fmt::Debug
        + PartialEq
        + serde::Serialize
        + for<'de> serde::Deserialize<'de>;

    /// Extract the raw numeric value from a `Value<T>`, if available.
    ///
    /// Returns `Some(T)` for [`Both`] (from `.raw`) and [`Raw`] (the value itself),
    /// `None` for [`Pretty`] (no numeric representation is stored).
    fn raw_from<
        T: Clone + std::fmt::Debug + PartialEq + serde::Serialize + for<'de> serde::Deserialize<'de>,
    >(
        value: &Self::Value<T>,
    ) -> Option<T>;
}

/// Full format — fields hold `FormattedValue<T>` with `raw`, `fmt`, and `long_fmt`.
///
/// This is the default format parameter, so a bare [`Quote`](super::quote::Quote) is
/// `Quote<Both>`. This is the form that can be deserialized directly from
/// Yahoo Finance JSON.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Both;

/// Raw format — fields hold `T` directly (e.g. `f64`, `i64`).
///
/// Obtain via `Ticker::quote::<Raw>()`,
/// [`Quote::into_raw`](super::quote::Quote::into_raw), or
/// [`Quote::as_raw`](super::quote::Quote::as_raw). No `Option`-wrapping of the value itself;
/// the `Option` at the field level reflects missing data from the API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Raw;

/// Pretty format — fields hold an `Option<String>` with the human-readable representation.
///
/// Obtain via [`Quote::into_pretty`](super::quote::Quote::into_pretty).
/// Falls back to `long_fmt` when `fmt` is absent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pretty;

impl sealed::Sealed for Both {}
impl sealed::Sealed for Raw {}
impl sealed::Sealed for Pretty {}

impl Format for Both {
    type Value<
        T: Clone + std::fmt::Debug + PartialEq + serde::Serialize + for<'de> serde::Deserialize<'de>,
    > = FormattedValue<T>;

    fn raw_from<
        T: Clone + std::fmt::Debug + PartialEq + serde::Serialize + for<'de> serde::Deserialize<'de>,
    >(
        value: &FormattedValue<T>,
    ) -> Option<T> {
        value.raw.clone()
    }
}

impl Format for Raw {
    type Value<
        T: Clone + std::fmt::Debug + PartialEq + serde::Serialize + for<'de> serde::Deserialize<'de>,
    > = T;

    fn raw_from<
        T: Clone + std::fmt::Debug + PartialEq + serde::Serialize + for<'de> serde::Deserialize<'de>,
    >(
        value: &T,
    ) -> Option<T> {
        Some(value.clone())
    }
}

impl Format for Pretty {
    type Value<
        T: Clone + std::fmt::Debug + PartialEq + serde::Serialize + for<'de> serde::Deserialize<'de>,
    > = String;

    fn raw_from<
        T: Clone + std::fmt::Debug + PartialEq + serde::Serialize + for<'de> serde::Deserialize<'de>,
    >(
        _value: &String,
    ) -> Option<T> {
        None
    }
}
