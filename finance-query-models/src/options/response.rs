use super::contract::{Contracts, OptionContract};
use crate::Provider;
/// Options Response module
///
/// Handles parsing of Yahoo Finance options API responses.
/// These types are internal implementation details and not exposed in the public API.
use serde::{Deserialize, Serialize};

/// Response wrapper for options endpoint
///
/// With Yahoo, `Ticker::options(None)` returns the chain for the nearest
/// expiration only; [`expiration_dates`](Self::expiration_dates) lists the
/// others to pass as `Some(timestamp)`. Other providers may load several
/// expirations at once.
///
/// Note: While this type is public for return values, users should not manually construct it.
/// Use `Ticker::options()` to obtain options data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Options {
    /// Option chain container
    #[doc(hidden)]
    pub option_chain: OptionChainContainer,

    /// Which data provider served this data (e.g., "yahoo", "polygon").
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub provider_id: Option<Provider>,
}

/// Container for option chain results
#[derive(Debug, Clone, Serialize, Deserialize)]
#[doc(hidden)]
pub struct OptionChainContainer {
    /// Results array
    pub result: Vec<OptionChainResult>,

    /// Error if any
    pub error: Option<serde_json::Value>,
}

/// Single option chain result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[doc(hidden)]
pub struct OptionChainResult {
    /// Underlying symbol
    pub underlying_symbol: Option<String>,

    /// Available expiration dates (Unix timestamps)
    pub expiration_dates: Option<Vec<i64>>,

    /// Available strike prices
    pub strikes: Option<Vec<f64>>,

    /// Whether has mini options
    pub has_mini_options: Option<bool>,

    /// Quote data
    pub quote: Option<serde_json::Value>,

    /// Options data (array of option chains)
    pub options: Vec<OptionChainData>,
}

/// Option chain data for a specific expiration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[doc(hidden)]
pub struct OptionChainData {
    /// Expiration date (Unix timestamp)
    pub expiration_date: i64,

    /// Whether has mini options
    pub has_mini_options: Option<bool>,

    /// Call options
    pub calls: Option<Vec<OptionContract>>,

    /// Put options
    pub puts: Option<Vec<OptionContract>>,
}

impl Options {
    /// Get the first result
    pub(crate) fn first_result(&self) -> Option<&OptionChainResult> {
        self.option_chain.result.first()
    }

    /// Get available expiration dates
    pub fn expiration_dates(&self) -> Vec<i64> {
        self.first_result()
            .and_then(|r| r.expiration_dates.clone())
            .unwrap_or_default()
    }

    /// Map of expiration timestamp → loaded contract count (calls + puts).
    ///
    /// Yahoo only populates chains for the requested expiration, so this
    /// typically covers a single date; absent expirations are omitted.
    pub(crate) fn contract_counts(&self) -> std::collections::HashMap<i64, usize> {
        self.first_result()
            .map(|r| {
                r.options
                    .iter()
                    .map(|chain| {
                        let calls = chain.calls.as_ref().map_or(0, Vec::len);
                        let puts = chain.puts.as_ref().map_or(0, Vec::len);
                        (chain.expiration_date, calls + puts)
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get strike prices
    pub fn strikes(&self) -> Vec<f64> {
        self.first_result()
            .and_then(|r| r.strikes.clone())
            .unwrap_or_default()
    }

    /// Earliest expiration date (Unix timestamp) with a loaded chain.
    ///
    /// For a Yahoo chain fetched with `options(None)` this is the nearest
    /// expiration. Contracts without an expiration are grouped under `0` by
    /// providers that build chains from flat contract lists; that group is
    /// skipped. `None` when no dated chain is loaded.
    pub fn loaded_expiration(&self) -> Option<i64> {
        self.first_result()?
            .options
            .iter()
            .map(|chain| chain.expiration_date)
            .filter(|&date| date != 0)
            .min()
    }

    /// Borrowing iterator over call contracts across all loaded expirations.
    ///
    /// Same contracts as [`calls`](Self::calls) without cloning them.
    pub fn calls_iter(&self) -> impl Iterator<Item = &OptionContract> {
        self.contracts_iter(|chain| chain.calls.as_deref())
    }

    /// Borrowing iterator over put contracts across all loaded expirations.
    ///
    /// Same contracts as [`puts`](Self::puts) without cloning them.
    pub fn puts_iter(&self) -> impl Iterator<Item = &OptionContract> {
        self.contracts_iter(|chain| chain.puts.as_deref())
    }

    fn contracts_iter<'a>(
        &'a self,
        side: impl Fn(&'a OptionChainData) -> Option<&'a [OptionContract]> + 'a,
    ) -> impl Iterator<Item = &'a OptionContract> {
        self.first_result()
            .into_iter()
            .flat_map(|r| r.options.iter())
            .flat_map(move |chain| side(chain).unwrap_or_default())
    }

    /// Get the call contracts of every expiration in this response, flattened.
    ///
    /// With Yahoo, a response fetched with `ticker.options(None)` holds only the
    /// nearest expiration; other providers may load several.
    ///
    /// Returns a `Contracts` wrapper that supports `.to_dataframe()` when
    /// the `dataframe` feature is enabled.
    ///
    /// # Example
    /// ```ignore
    /// let options = ticker.options(None).await?;
    /// for call in &*options.calls() {
    ///     println!("{}: strike={}", call.contract_symbol, call.strike);
    /// }
    /// // With dataframe feature:
    /// let df = options.calls().to_dataframe()?;
    /// ```
    pub fn calls(&self) -> Contracts {
        let contracts = self
            .first_result()
            .map(|r| {
                r.options
                    .iter()
                    .flat_map(|chain| chain.calls.as_deref().unwrap_or_default().iter())
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();
        Contracts(contracts)
    }

    /// Get the put contracts of every expiration in this response, flattened.
    ///
    /// A response fetched with `ticker.options(None)` holds only the nearest
    /// expiration.
    ///
    /// Returns a `Contracts` wrapper that supports `.to_dataframe()` when
    /// the `dataframe` feature is enabled.
    ///
    /// # Example
    /// ```ignore
    /// let options = ticker.options(None).await?;
    /// for put in &*options.puts() {
    ///     println!("{}: strike={}", put.contract_symbol, put.strike);
    /// }
    /// // With dataframe feature:
    /// let df = options.puts().to_dataframe()?;
    /// ```
    pub fn puts(&self) -> Contracts {
        let contracts = self
            .first_result()
            .map(|r| {
                r.options
                    .iter()
                    .flat_map(|chain| chain.puts.as_deref().unwrap_or_default().iter())
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();
        Contracts(contracts)
    }
}

#[cfg(feature = "dataframe")]
impl Options {
    /// Converts all option contracts (calls and puts) to a polars DataFrame.
    ///
    /// Flattens the contracts of every expiration in this response into a single DataFrame
    /// with an additional `option_type` column ("call" or "put").
    ///
    /// For separate DataFrames, use `options.calls().to_dataframe()` or
    /// `options.puts().to_dataframe()`.
    pub fn to_dataframe(&self) -> ::polars::prelude::PolarsResult<::polars::prelude::DataFrame> {
        use polars::prelude::*;

        let calls = self.calls();
        let puts = self.puts();

        let mut calls_df = calls.to_dataframe()?;
        let mut puts_df = puts.to_dataframe()?;

        // Add option_type column
        let call_types = Series::new("option_type".into(), vec!["call"; calls.len()]);
        let put_types = Series::new("option_type".into(), vec!["put"; puts.len()]);

        calls_df.with_column(call_types.into())?;
        puts_df.with_column(put_types.into())?;

        // Combine into single DataFrame
        calls_df.vstack(&puts_df)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn options() -> Options {
        serde_json::from_value(serde_json::json!({
            "optionChain": {
                "result": [{
                    "expirationDates": [100, 200],
                    "options": [{
                        "expirationDate": 100,
                        "calls": [{ "contractSymbol": "C1", "strike": 10.0 }],
                        "puts": [
                            { "contractSymbol": "P1", "strike": 9.0 },
                            { "contractSymbol": "P2", "strike": 8.0 }
                        ]
                    }]
                }],
                "error": null
            }
        }))
        .unwrap()
    }

    #[test]
    fn iterators_borrow_the_same_contracts_as_the_cloning_accessors() {
        let options = options();
        let calls: Vec<_> = options.calls_iter().map(|c| &c.contract_symbol).collect();
        let puts: Vec<_> = options.puts_iter().map(|c| &c.contract_symbol).collect();
        assert_eq!(calls, ["C1"]);
        assert_eq!(puts, ["P1", "P2"]);
        assert_eq!(options.calls().len(), calls.len());
        assert_eq!(options.puts().len(), puts.len());
    }

    #[test]
    fn loaded_expiration_is_the_first_chain() {
        assert_eq!(options().loaded_expiration(), Some(100));
    }

    #[test]
    fn loaded_expiration_skips_undated_contracts_from_flat_providers() {
        let contract =
            |symbol: &str, expiration: Option<i64>| -> crate::models::options::OptionContract {
                serde_json::from_value(serde_json::json!({
                    "contractSymbol": symbol,
                    "strike": 1.0,
                    "expiration": expiration,
                }))
                .unwrap()
            };
        let options = crate::providers::build_options(
            "X".into(),
            Provider::Yahoo,
            vec![],
            vec![
                contract("C0", None),
                contract("C2", Some(200)),
                contract("C1", Some(100)),
            ],
            vec![],
        );
        assert_eq!(options.loaded_expiration(), Some(100));
    }

    #[test]
    fn empty_result_has_no_contracts_or_expiration() {
        let options: Options = serde_json::from_value(
            serde_json::json!({"optionChain": {"result": [], "error": null}}),
        )
        .unwrap();
        assert_eq!(options.loaded_expiration(), None);
        assert_eq!(options.calls_iter().count(), 0);
    }
}
