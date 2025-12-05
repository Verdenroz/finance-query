// Copyright (c) 2025 finance-query contributors
// SPDX-License-Identifier: MIT

//! Macros for generating quote summary accessor methods.
//!
//! This module contains declarative macros that eliminate code duplication
//! between sync and async implementations of quote summary accessors.

/// Generate quote summary accessor methods for both sync and async Ticker types.
///
/// This macro eliminates ~300 lines of duplication by generating identical
/// methods for both `AsyncTicker` and `Ticker` from a single declaration.
///
/// # Usage
///
/// ```ignore
/// define_quote_accessors! {
///     /// Get price information
///     price -> Price, "price",
///
///     /// Get summary detail
///     summary_detail -> SummaryDetail, "summaryDetail",
/// }
/// ```
///
/// This generates:
/// - `AsyncTicker::price(&self) -> Result<Option<Price>>`
/// - `Ticker::price(&self) -> Result<Option<Price>>`
///
/// Both methods follow the same pattern:
/// 1. Ensure quote summary data is loaded
/// 2. Read from cache
/// 3. Extract and return the typed module data
macro_rules! define_quote_accessors {
    (
        $(
            $(#[$meta:meta])*
            $method_name:ident -> $return_type:ty, $module_name:literal
        ),* $(,)?
    ) => {
        // Generate async methods for AsyncTicker
        impl AsyncTicker {
            $(
                $(#[$meta])*
                pub async fn $method_name(&self) -> Result<Option<$return_type>> {
                    self.ensure_quote_summary_loaded().await?;
                    let cache = self.quote_summary.read().await;
                    Ok(cache.as_ref().and_then(|r| r.get_typed($module_name).ok()))
                }
            )*
        }

        // Generate sync methods for Ticker
        impl Ticker {
            $(
                $(#[$meta])*
                pub fn $method_name(&self) -> Result<Option<$return_type>> {
                    self.ensure_quote_summary_loaded()?;
                    let cache = self.quote_summary.read().unwrap();
                    Ok(cache.as_ref().and_then(|r| r.get_typed($module_name).ok()))
                }
            )*
        }
    };
}

pub(crate) use define_quote_accessors;
