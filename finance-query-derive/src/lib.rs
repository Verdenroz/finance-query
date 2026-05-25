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
//! finance-query = { version = "2.0", features = ["dataframe"] }
//! ```
//!
//! ## Example
//!
//! ```ignore
//! use finance_query::ToDataFrame;
//! use polars::prelude::*;
//!
//! #[derive(ToDataFrame)]
//! struct Quote {
//!     symbol: String,
//!     price: Option<f64>,
//!     volume: Option<i64>,
//! }
//!
//! // Automatically generates:
//! // - to_dataframe(&self) -> PolarsResult<DataFrame>
//! // - vec_to_dataframe(&[Self]) -> PolarsResult<DataFrame>
//!
//! let quote = Quote {
//!     symbol: "AAPL".to_string(),
//!     price: Some(150.0),
//!     volume: Some(1000000),
//! };
//!
//! let df = quote.to_dataframe()?;
//! ```
//!
//! ## Supported Types
//!
//! The `ToDataFrame` derive macro supports the following field types:
//!
//! - **Primitives**: `i32`, `i64`, `u32`, `u64`, `f64`, `bool`
//! - **Strings**: `String`, `Option<String>`
//! - **Optional primitives**: `Option<i32>`, `Option<f64>`, etc.
//! - **FormattedValue**: `Option<FormattedValue<f64>>`, `Option<FormattedValue<i64>>`
//!   (automatically extracts the `.raw` field)
//!
//! Complex types like nested structs and vectors are automatically skipped and won't
//! appear in the generated DataFrame.
//!
//! ## Generated Methods
//!
//! For each struct with `#[derive(ToDataFrame)]`, two methods are generated:
//!
//! ### `to_dataframe(&self)`
//!
//! Converts a single instance to a one-row DataFrame:
//!
//! ```ignore
//! let quote = Quote { /* ... */ };
//! let df: DataFrame = quote.to_dataframe()?;
//! ```
//!
//! ### `vec_to_dataframe(items: &[Self])`
//!
//! Converts a slice of instances to a multi-row DataFrame:
//!
//! ```ignore
//! let quotes = vec![quote1, quote2, quote3];
//! let df: DataFrame = Quote::vec_to_dataframe(&quotes)?;
//! ```

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    Data, DeriveInput, Fields, GenericArgument, GenericParam, Ident, PathArguments, Type,
    TypeParam, TypeParamBound, TypePath, parse_macro_input,
};

/// Derive macro for automatic DataFrame conversion.
///
/// Generates a `to_dataframe(&self) -> PolarsResult<DataFrame>` method
/// that converts all struct fields to DataFrame columns.
///
/// # Supported Types
///
/// - `String` → String column
/// - `Option<String>` → nullable String column
/// - `Option<FormattedValue<f64>>` → extracts `.raw` as `Option<f64>`
/// - `Option<FormattedValue<i64>>` → extracts `.raw` as `Option<i64>`
/// - `i32`, `i64`, `f64`, `bool` → direct columns
/// - `Option<T>` for primitives → nullable columns
/// - Nested structs/Vec → skipped (complex types not suitable for flat DataFrame)
///
/// # Example
///
/// ```ignore
/// #[derive(ToDataFrame)]
/// pub struct Quote {
///     pub symbol: String,
///     pub price: Option<FormattedValue<f64>>,
/// }
///
/// // Generates:
/// impl Quote {
///     pub fn to_dataframe(&self) -> PolarsResult<DataFrame> {
///         df![
///             "symbol" => [self.symbol.as_str()],
///             "price" => [self.price.as_ref().and_then(|v| v.raw)],
///         ]
///     }
/// }
/// ```
#[proc_macro_derive(ToDataFrame)]
pub fn derive_to_dataframe(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return syn::Error::new_spanned(
                    &input,
                    "ToDataFrame only supports structs with named fields",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(&input, "ToDataFrame only supports structs")
                .to_compile_error()
                .into();
        }
    };

    // If the struct is generic over F: Format, generate impl for <Both> specifically.
    let has_format_param = input.generics.params.iter().any(|param| {
        if let GenericParam::Type(TypeParam { bounds, .. }) = param {
            bounds.iter().any(|b| {
                if let TypeParamBound::Trait(tb) = b {
                    tb.path
                        .segments
                        .last()
                        .map(|s| s.ident == "Format")
                        .unwrap_or(false)
                } else {
                    false
                }
            })
        } else {
            false
        }
    });

    let impl_ty = if has_format_param {
        quote! { #name<crate::format::Both> }
    } else {
        quote! { #name }
    };

    // Detect the format type parameter name (e.g. `F`) to scope is_format_assoc_value correctly.
    let format_param_ident: Option<&Ident> = input.generics.params.iter().find_map(|param| {
        if let GenericParam::Type(TypeParam { ident, bounds, .. }) = param {
            let is_format = bounds.iter().any(|b| {
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
            if is_format { Some(ident) } else { None }
        } else {
            None
        }
    });

    let mut column_names: Vec<String> = Vec::new();
    let mut column_values: Vec<TokenStream2> = Vec::new();

    for field in fields.iter() {
        let field_name = field.ident.as_ref().unwrap();
        let field_name_str = to_snake_case(&field_name.to_string());
        let field_type = &field.ty;

        if let Some(value_expr) = generate_column_value(field_name, field_type, format_param_ident)
        {
            column_names.push(field_name_str);
            column_values.push(value_expr);
        }
    }

    let mut vec_column_values: Vec<TokenStream2> = Vec::new();
    for field in fields.iter() {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;

        if let Some(value_expr) =
            generate_vec_column_value(field_name, field_type, format_param_ident)
        {
            vec_column_values.push(value_expr);
        }
    }

    let expanded = quote! {
        #[cfg(feature = "dataframe")]
        impl #impl_ty {
            /// Converts this struct to a single-row polars DataFrame.
            ///
            /// All scalar fields are included as columns. Nested objects
            /// and complex types are excluded.
            ///
            /// This method is auto-generated by the `ToDataFrame` derive macro.
            pub fn to_dataframe(&self) -> ::polars::prelude::PolarsResult<::polars::prelude::DataFrame> {
                use ::polars::prelude::*;
                df![
                    #( #column_names => #column_values ),*
                ]
            }

            /// Converts a slice of structs to a multi-row polars DataFrame.
            ///
            /// All scalar fields are included as columns. Nested objects
            /// and complex types are excluded.
            ///
            /// This method is auto-generated by the `ToDataFrame` derive macro.
            pub fn vec_to_dataframe(items: &[Self]) -> ::polars::prelude::PolarsResult<::polars::prelude::DataFrame> {
                use ::polars::prelude::*;
                df![
                    #( #column_names => #vec_column_values ),*
                ]
            }
        }
    };

    TokenStream::from(expanded)
}

/// Converts a field name to snake_case for DataFrame column names.
fn to_snake_case(s: &str) -> String {
    s.to_string()
}

/// Generates the value expression for a DataFrame column based on field type.
///
/// Returns `None` for complex types that should be skipped.
fn generate_column_value(
    field_name: &syn::Ident,
    field_type: &Type,
    fmt_param: Option<&Ident>,
) -> Option<TokenStream2> {
    match field_type {
        Type::Path(type_path) if is_string(type_path) => {
            Some(quote! { [self.#field_name.as_str()] })
        }
        Type::Path(type_path) if is_formatted_value(type_path) => {
            Some(quote! { [self.#field_name.raw] })
        }
        Type::Path(type_path) if is_option(type_path) => {
            let inner_type = get_option_inner_type(type_path)?;
            generate_option_value(field_name, inner_type, fmt_param)
        }
        Type::Path(type_path) if is_primitive(type_path) => Some(quote! { [self.#field_name] }),
        _ => None,
    }
}

/// Generates the value expression for a DataFrame column when iterating over a Vec.
///
/// Returns `None` for complex types that should be skipped.
fn generate_vec_column_value(
    field_name: &syn::Ident,
    field_type: &Type,
    fmt_param: Option<&Ident>,
) -> Option<TokenStream2> {
    match field_type {
        Type::Path(type_path) if is_string(type_path) => {
            Some(quote! { items.iter().map(|item| item.#field_name.as_str()).collect::<Vec<_>>() })
        }
        Type::Path(type_path) if is_formatted_value(type_path) => {
            Some(quote! { items.iter().map(|item| item.#field_name.raw).collect::<Vec<_>>() })
        }
        Type::Path(type_path) if is_option(type_path) => {
            let inner_type = get_option_inner_type(type_path)?;
            generate_vec_option_value(field_name, inner_type, fmt_param)
        }
        Type::Path(type_path) if is_primitive(type_path) => {
            Some(quote! { items.iter().map(|item| item.#field_name).collect::<Vec<_>>() })
        }
        _ => None,
    }
}

/// Generates value expression for Option<T> fields when iterating over a Vec.
fn generate_vec_option_value(
    field_name: &syn::Ident,
    inner_type: &Type,
    fmt_param: Option<&Ident>,
) -> Option<TokenStream2> {
    match inner_type {
        Type::Path(type_path) if is_string(type_path) => Some(
            quote! { items.iter().map(|item| item.#field_name.as_deref()).collect::<Vec<_>>() },
        ),
        // Option<FormattedValue<T>> or Option<F::Value<T>> — extract .raw
        Type::Path(type_path)
            if is_formatted_value(type_path) || is_format_assoc_value(type_path, fmt_param) =>
        {
            Some(
                quote! { items.iter().map(|item| item.#field_name.as_ref().and_then(|v| v.raw)).collect::<Vec<_>>() },
            )
        }
        Type::Path(type_path) if is_primitive(type_path) => {
            Some(quote! { items.iter().map(|item| item.#field_name).collect::<Vec<_>>() })
        }
        _ => None,
    }
}

/// Generates value expression for Option<T> fields.
fn generate_option_value(
    field_name: &syn::Ident,
    inner_type: &Type,
    fmt_param: Option<&Ident>,
) -> Option<TokenStream2> {
    match inner_type {
        Type::Path(type_path) if is_string(type_path) => {
            Some(quote! { [self.#field_name.as_deref()] })
        }
        // Option<FormattedValue<T>> or Option<F::Value<T>> — extract .raw
        Type::Path(type_path)
            if is_formatted_value(type_path) || is_format_assoc_value(type_path, fmt_param) =>
        {
            Some(quote! { [self.#field_name.as_ref().and_then(|v| v.raw)] })
        }
        Type::Path(type_path) if is_primitive(type_path) => Some(quote! { [self.#field_name] }),
        _ => None,
    }
}

/// Checks if a type path is `String`.
fn is_string(type_path: &TypePath) -> bool {
    type_path
        .path
        .segments
        .last()
        .map(|seg| seg.ident == "String")
        .unwrap_or(false)
}

/// Checks if a type path is `Option<T>`.
fn is_option(type_path: &TypePath) -> bool {
    type_path
        .path
        .segments
        .last()
        .map(|seg| seg.ident == "Option")
        .unwrap_or(false)
}

/// Checks if a type path is `FormattedValue<T>`.
fn is_formatted_value(type_path: &TypePath) -> bool {
    type_path
        .path
        .segments
        .last()
        .map(|seg| seg.ident == "FormattedValue")
        .unwrap_or(false)
}

/// Checks if a type path is `PARAM::Value<T>` — the associated type form used by `F: Format`.
///
/// Only matches when `fmt_param` is known and the first segment equals it exactly
/// (e.g. `F::Value<f64>` where `F` is the detected format type parameter).
/// This prevents false-positive matches on unrelated 2-segment paths like `serde_json::Value`.
fn is_format_assoc_value(type_path: &TypePath, fmt_param: Option<&Ident>) -> bool {
    let Some(param) = fmt_param else { return false };
    let segs = &type_path.path.segments;
    segs.len() == 2 && segs[0].ident == *param && segs[1].ident == "Value"
}

/// Checks if a type path is a primitive type (i32, i64, f64, bool).
fn is_primitive(type_path: &TypePath) -> bool {
    type_path
        .path
        .segments
        .last()
        .map(|seg| {
            let name = seg.ident.to_string();
            matches!(
                name.as_str(),
                "i32" | "i64" | "f64" | "bool" | "u32" | "u64"
            )
        })
        .unwrap_or(false)
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
                quote! { #ident: v.#ident.and_then(|fv| fv.fmt.or(fv.long_fmt)) }
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

/// Extracts the inner type from Option<T>.
fn get_option_inner_type(type_path: &TypePath) -> Option<&Type> {
    let segment = type_path.path.segments.last()?;
    if segment.ident != "Option" {
        return None;
    }

    match &segment.arguments {
        PathArguments::AngleBracketed(args) => args.args.first().and_then(|arg| {
            if let GenericArgument::Type(ty) = arg {
                Some(ty)
            } else {
                None
            }
        }),
        _ => None,
    }
}
