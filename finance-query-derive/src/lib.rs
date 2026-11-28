//! # finance-query-derive
//!
//! Procedural macros for the `finance-query` library.
//!
//! This crate provides derive macros that automatically generate code for working with
//! financial data structures, particularly for integration with the Polars DataFrame library.
//!
//! ## Features
//!
//! - **`ToDataFrame`**: Automatically implement DataFrame conversion for structs
//!
//! ## Usage
//!
//! This crate is automatically included when you enable the `dataframe` feature in `finance-query`:
//!
//! ```toml
//! [dependencies]
//! finance-query = { version = "3", features = ["dataframe"] }
//! ```
//!
//! ## Example
//!
//! ```ignore
//! use finance_query::ToDataFrame;
//!
//! #[derive(ToDataFrame)]
//! struct Quote {
//!     symbol: String,
//!     price: Option<f64>,
//!     volume: Option<i64>,
//! }
//!
//! let quote = Quote {
//!     symbol: "AAPL".to_string(),
//!     price: Some(150.0),
//!     volume: Some(1000000),
//! };
//!
//! let df = quote.to_dataframe()?;
//! let df = Quote::vec_to_dataframe(&[quote])?;
//! ```
//!
//! The generated code refers to `::finance_query`, so the deriving crate needs
//! `finance-query` with the `dataframe` feature as a dependency. See
//! [`ToDataFrame`](derive@ToDataFrame) for the field mappings and attributes.

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

mod dataframe;
mod fields_init;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Data, DeriveInput, Fields, GenericArgument, GenericParam, Ident, PathArguments, Type,
    TypeParam, TypeParamBound, parse_macro_input,
};

/// Derive macro for automatic DataFrame conversion.
///
/// Generates inherent `to_dataframe(&self)` and `vec_to_dataframe(&[Self])`
/// methods returning `PolarsResult<DataFrame>`, and implements
/// `finance_query::ToDataFrame` by delegating to them. Each field becomes a
/// column named after it, in field order; `Option<T>` fields become nullable
/// columns of `T`'s mapping.
///
/// # Field mappings
///
/// - `String` → `String`
/// - `i32`, `i64`, `u32`, `u64`, `f32`, `f64`, `bool` → the matching dtype
/// - `i8`, `i16` → `Int32`; `u8`, `u16` → `UInt32`
/// - `usize` → `UInt64`; `isize` → `Int64`
/// - `FormattedValue<T>` and `F::Value<T>` → `T`, from the `.raw` value
/// - `Vec<String>` → `List(String)` with `#[dataframe(list)]`, otherwise skipped
/// - any other type implementing `Display` (enums such as `Provider`) → `String`, via `to_string()`
/// - chrono date/time types, `serde_json::Value`, non-`Display` structs, other `Vec`s and maps → skipped
///
/// Types are matched by name, so a numeric type alias or a `Value` imported
/// with `use serde_json::Value` falls through to the `Display` rule. Give such
/// fields `#[dataframe(skip)]` or write the full type.
///
/// # Attributes
///
/// - `#[dataframe(skip)]` leaves the field out.
/// - `#[dataframe(rename = "name")]` sets the column name.
/// - `#[dataframe(flatten)]` on a field whose type also derives `ToDataFrame`
///   emits that type's columns as `{field}_{column}`, null where the field is `None`.
/// - `#[dataframe(flatten, prefix = "name")]` uses `{name}_{column}` instead, and
///   `prefix = ""` emits the bare `{column}`. `prefix` requires `flatten`.
/// - `#[dataframe(time_exempt = "reason")]` exempts a column from the time naming test
///   below. The reason must not be empty.
/// - `#[dataframe(list)]` on a `Vec<String>` field emits a `List(String)` column.
/// - `#[dataframe(nested)]` on a `Vec<T>`, `Option<Vec<T>>`, `T` or `Option<T>`
///   field, where `T: ToDataFrame`, keeps the rows out of `to_dataframe` and adds
///   `{field}_dataframe()` for that field's rows, plus `to_nested_dataframe()` and
///   `vec_to_nested_dataframe()` with one `List(Struct)` column per nested field.
///   A `None` or empty field is an empty list. The same methods are the
///   `ToDataFrame` trait's `to_nested_dataframe` and `vec_to_nested_dataframe`.
///   A single `T` or `Option<T>` field uses `T`'s nested frame, so a wrapper that
///   only holds row lists becomes one row with its own `List(Struct)` columns;
///   `Vec<T>` child frames stay flat. It cannot be combined with `skip`, `list`
///   or `flatten`.
/// - `#[dataframe(crate = "path")]` on the struct points the generated code at
///   `finance_query` re-exported under another path, and adds a `#[cfg(test)]` test
///   that checks the empty frame's time naming: a name ending in `date`, `time`,
///   `timestamp`, `expiration` or `at` must be a `Date` or `Datetime` column (or a
///   list of either), and a `Date` or `Datetime` column must carry one of those
///   words or `start`, `end`, `ending`, `filed`, `published`, `updated`, `quarter`,
///   `year` or `period` in its name.
///
/// # Time columns
///
/// The struct field keeps its type; only the column is typed. Every datetime
/// column is `Datetime(Microseconds, UTC)` and every date column is `Date`,
/// whatever the source unit, so columns from different sources join without casts.
///
/// - `#[dataframe(datetime = "s" | "ms" | "us" | "ns")]` on `i64`,
///   `FormattedValue<i64>` or `F::Value<i64>` reads an epoch instant.
/// - `#[dataframe(datetime)]` on a `String` parses RFC 3339 / ISO 8601 with an
///   offset or `Z`.
/// - `#[dataframe(date)]` on a `String` parses the leading `YYYY-MM-DD`, so
///   `"2024-01-02 16:30:00"` and `"2024-01-02T00:00:00.000"` both work.
/// - `#[dataframe(date = "s" | "ms")]` on the epoch integer types takes the UTC calendar day.
/// - `nonzero` next to an epoch form turns `0` into null, for fields where
///   `0` means unset.
/// - `Vec` of the epoch integer types, or `Option<Vec<..>>`, needs `list` and
///   becomes `List(Datetime)` or `List(Date)`.
///
/// A value that does not parse, or an epoch that overflows the microsecond
/// range, becomes null and never an error. The raw value stays on the struct.
/// `Option` fields are nullable as usual. Combining a time attribute with
/// `skip`, `flatten` or `nested`, a `datetime` without a unit on an integer, a
/// unit on a `String`, or any other field type fails to compile.
///
/// ```ignore
/// #[derive(ToDataFrame)]
/// pub struct Event {
///     #[dataframe(datetime = "s")]
///     pub timestamp: i64,
///     #[dataframe(date)]
///     pub filing_date: String,
///     #[dataframe(date = "s", nonzero)]
///     pub expire_date: i64,
/// }
/// // Columns: Datetime(us, UTC), Date, Date
/// ```
///
/// A unit-less `datetime` on an integer fails to compile:
///
/// ```compile_fail
/// # use finance_query::ToDataFrame;
/// #[derive(ToDataFrame)]
/// struct Row {
///     #[dataframe(datetime)]
///     time: i64,
/// }
/// ```
///
/// So does a unit on a `String`:
///
/// ```compile_fail
/// # use finance_query::ToDataFrame;
/// #[derive(ToDataFrame)]
/// struct Row {
///     #[dataframe(datetime = "s")]
///     time: String,
/// }
/// ```
///
/// A time attribute on any other type fails too:
///
/// ```compile_fail
/// # use finance_query::ToDataFrame;
/// #[derive(ToDataFrame)]
/// struct Row {
///     #[dataframe(date = "s")]
///     time: f64,
/// }
/// ```
///
/// A `Vec` needs `list`:
///
/// ```compile_fail
/// # use finance_query::ToDataFrame;
/// #[derive(ToDataFrame)]
/// struct Row {
///     #[dataframe(datetime = "s")]
///     times: Vec<i64>,
/// }
/// ```
///
/// A time attribute cannot be combined with `flatten`, `nested` or `skip`:
///
/// ```compile_fail
/// # use finance_query::ToDataFrame;
/// #[derive(ToDataFrame)]
/// struct Row {
///     #[dataframe(datetime = "s", skip)]
///     time: i64,
/// }
/// ```
///
/// `nonzero` only applies to epoch forms:
///
/// ```compile_fail
/// # use finance_query::ToDataFrame;
/// #[derive(ToDataFrame)]
/// struct Row {
///     #[dataframe(date, nonzero)]
///     day: String,
/// }
/// ```
///
/// A struct generic over `F: Format` gets the impl for `F = Both` only.
///
/// # Example
///
/// ```ignore
/// #[derive(ToDataFrame)]
/// pub struct Quote {
///     pub symbol: String,
///     #[dataframe(rename = "last")]
///     pub price: Option<FormattedValue<f64>>,
///     pub provider: Option<Provider>,
///     #[dataframe(flatten)]
///     pub sentiment: Option<Sentiment>,
/// }
/// // Columns: symbol, last, provider, sentiment_label, sentiment_score, sentiment_confidence
/// ```
///
/// Nested rows reach polars as a child frame or as list columns on the parent:
///
/// ```ignore
/// #[derive(ToDataFrame)]
/// pub struct Fund {
///     pub symbol: String,
///     #[dataframe(nested)]
///     pub holdings: Vec<Holding>,
/// }
///
/// let holdings = fund.holdings_dataframe()?; // one row per holding
/// let nested = fund.to_nested_dataframe()?;  // symbol, holdings: List(Struct)
/// ```
///
/// Combining `nested` with `flatten` fails to compile:
///
/// ```compile_fail
/// # use finance_query::ToDataFrame;
/// #[derive(ToDataFrame)]
/// struct Row {
///     value: i64,
/// }
///
/// #[derive(ToDataFrame)]
/// struct Parent {
///     #[dataframe(nested, flatten)]
///     rows: Vec<Row>,
/// }
/// ```
///
/// `prefix` without `flatten` fails to compile:
///
/// ```compile_fail
/// # use finance_query::ToDataFrame;
/// #[derive(ToDataFrame)]
/// struct Row {
///     value: i64,
/// }
///
/// #[derive(ToDataFrame)]
/// struct Parent {
///     #[dataframe(prefix = "row")]
///     row: Row,
/// }
/// ```
///
/// `time_exempt` needs a reason:
///
/// ```compile_fail
/// # use finance_query::ToDataFrame;
/// #[derive(ToDataFrame)]
/// struct Row {
///     #[dataframe(time_exempt = "")]
///     time: String,
/// }
/// ```
///
/// A nested column name that matches another column also fails to compile:
///
/// ```compile_fail
/// # use finance_query::ToDataFrame;
/// #[derive(ToDataFrame)]
/// struct Row {
///     value: i64,
/// }
///
/// #[derive(ToDataFrame)]
/// struct Parent {
///     rows: String,
///     #[dataframe(nested, rename = "rows")]
///     children: Vec<Row>,
/// }
/// ```
#[proc_macro_derive(ToDataFrame, attributes(dataframe))]
pub fn derive_to_dataframe(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    dataframe::expand(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Derive macro for format-typed struct conversions.
///
/// For structs generic over `F: Format` (e.g. `SummaryDetail<F: Format = Both>`), generates:
///
/// - `impl From<Struct<Both>> for Struct<Raw>` — extracts `.raw` from each `FormattedValue` field
/// - `impl From<Struct<Both>> for Struct<Pretty>` — extracts `.fmt` (or `.long_fmt`) instead
/// - Convenience methods on `Struct<Both>`: `into_raw()`, `as_raw()`, `into_pretty()`, `as_pretty()`
///
/// Field classification:
/// - `Option<F::Value<T>>` (any 2-segment path ending in `::Value`) → formatted field
/// - Everything else → plain field, copied as-is
#[proc_macro_derive(FormatConvert)]
pub fn derive_format_convert(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return syn::Error::new_spanned(
                    &input,
                    "FormatConvert only supports structs with named fields",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(&input, "FormatConvert only supports structs")
                .to_compile_error()
                .into();
        }
    };

    // Find the format type param: a generic param with a bound path ending in "Format"
    let format_param: Option<&Ident> = input.generics.params.iter().find_map(|param| {
        if let GenericParam::Type(TypeParam { ident, bounds, .. }) = param {
            let has_format = bounds.iter().any(|b| {
                if let TypeParamBound::Trait(tb) = b {
                    tb.path
                        .segments
                        .last()
                        .map(|s| s.ident == "Format")
                        .unwrap_or(false)
                } else {
                    false
                }
            });
            if has_format { Some(ident) } else { None }
        } else {
            None
        }
    });

    let Some(format_param) = format_param else {
        return syn::Error::new_spanned(
            &input,
            "FormatConvert requires a generic param bounded by Format (e.g. <F: Format = Both>)",
        )
        .to_compile_error()
        .into();
    };

    // Build field classification: (field_ident, is_formatted)
    let classified: Vec<(&syn::Field, bool)> = fields
        .iter()
        .map(|f| (f, is_format_value_field(&f.ty, format_param)))
        .collect();

    // Generate field conversion expressions for Raw and Pretty
    let raw_field_exprs: Vec<_> = classified
        .iter()
        .map(|(f, is_fmt)| {
            let ident = f.ident.as_ref().unwrap();
            if *is_fmt {
                quote! { #ident: v.#ident.and_then(|fv| fv.raw) }
            } else {
                quote! { #ident: v.#ident }
            }
        })
        .collect();

    let pretty_field_exprs: Vec<_> = classified
        .iter()
        .map(|(f, is_fmt)| {
            let ident = f.ident.as_ref().unwrap();
            if *is_fmt {
                quote! { #ident: v.#ident.and_then(|fv| fv.fmt.or(fv.long_fmt).map(String::from)) }
            } else {
                quote! { #ident: v.#ident }
            }
        })
        .collect();

    let expanded = quote! {
        impl From<#name<crate::format::Both>> for #name<crate::format::Raw> {
            fn from(v: #name<crate::format::Both>) -> Self {
                #name {
                    #(#raw_field_exprs,)*
                }
            }
        }

        impl From<#name<crate::format::Both>> for #name<crate::format::Pretty> {
            fn from(v: #name<crate::format::Both>) -> Self {
                #name {
                    #(#pretty_field_exprs,)*
                }
            }
        }

        impl #name<crate::format::Both> {
            /// Convert into a [`Raw`](crate::format::Raw) view, extracting `.raw` from each `FormattedValue` field.
            pub fn into_raw(self) -> #name<crate::format::Raw> { self.into() }
            /// Clone and convert into a [`Raw`](crate::format::Raw) view.
            pub fn as_raw(&self) -> #name<crate::format::Raw> { self.clone().into() }
            /// Convert into a [`Pretty`](crate::format::Pretty) view, extracting `.fmt` from each `FormattedValue` field.
            pub fn into_pretty(self) -> #name<crate::format::Pretty> { self.into() }
            /// Clone and convert into a [`Pretty`](crate::format::Pretty) view.
            pub fn as_pretty(&self) -> #name<crate::format::Pretty> { self.clone().into() }
        }
    };

    TokenStream::from(expanded)
}

/// Derive macro that generates a hidden, exhaustive initializer for a struct.
///
/// For `struct X { .. }` it emits `#[doc(hidden)] pub struct XFieldsInit { .. }` with
/// every field public, plus `impl From<XFieldsInit> for X`. Unlike `X`, which may be
/// `#[non_exhaustive]`, the initializer can be built with a struct literal from any
/// crate, and the literal still has to set every field.
#[proc_macro_derive(FieldsInit)]
pub fn derive_fields_init(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    fields_init::expand(&input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Returns true if `ty` is `Option<PARAM::Value<T>>` for the given format param name.
fn is_format_value_field(ty: &Type, format_param: &Ident) -> bool {
    let inner = match get_option_inner(ty) {
        Some(t) => t,
        None => return false,
    };
    // Check for PARAM::Value<T>
    if let Type::Path(tp) = inner {
        let segs = &tp.path.segments;
        if segs.len() == 2 {
            return segs[0].ident == *format_param && segs[1].ident == "Value";
        }
    }
    false
}

/// Extracts the inner type T from Option<T>, returning None if the type isn't Option<…>.
fn get_option_inner(ty: &Type) -> Option<&Type> {
    if let Type::Path(tp) = ty {
        let seg = tp.path.segments.last()?;
        if seg.ident != "Option" {
            return None;
        }
        if let PathArguments::AngleBracketed(args) = &seg.arguments {
            return args.args.first().and_then(|a| {
                if let GenericArgument::Type(t) = a {
                    Some(t)
                } else {
                    None
                }
            });
        }
    }
    None
}
