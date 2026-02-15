// Copyright (c) 2025 finance-query contributors
// SPDX-License-Identifier: MIT

//! Macros for generating quote summary accessor methods.

/// Generate quote summary accessor methods for Ticker.
///
/// This macro eliminates code duplication by generating accessor methods
/// from a single declaration.
///
/// # Usage
///
/// ```ignore
/// define_quote_accessors! {
///     /// Get price information
///     price -> Price, price,
///
///     /// Get summary detail
///     summary_detail -> SummaryDetail, summary_detail,
/// }
/// ```
///
/// This generates:
/// - `Ticker::price(&self) -> Result<Option<Price>>`
///
/// Each method:
/// 1. Ensures quote summary data is loaded
/// 2. Reads from cache
/// 3. Clones and returns the typed module data
macro_rules! define_quote_accessors {
    (
        $(
            $(#[$meta:meta])*
            $method_name:ident -> $return_type:ty, $field_name:ident
        ),* $(,)?
    ) => {
        impl Ticker {
            $(
                $(#[$meta])*
                pub async fn $method_name(&self) -> crate::error::Result<Option<$return_type>> {
                    let cache = self.ensure_and_read_quote_summary().await?;
                    Ok(cache.as_ref().and_then(|e| e.value.$field_name.clone()))
                }
            )*
        }
    };
}

pub(crate) use define_quote_accessors;
