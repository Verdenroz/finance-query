//! Macros for generating batch response types and cached fetch methods.

/// Generate batch response types with identical structure but different field names.
///
/// Each response type contains:
/// - `HashMap<String, T>` for successful results
/// - `HashMap<String, String>` for errors
/// - `success_count()`, `error_count()`, `all_successful()` convenience methods
///
/// # Usage
///
/// ```ignore
/// define_batch_response! {
///     /// Response containing quotes for multiple symbols.
///     BatchQuotesResponse => quotes: Quote
/// }
/// ```
macro_rules! define_batch_response {
    (
        $(#[$meta:meta])*
        $name:ident => $field:ident : $type:ty
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        #[non_exhaustive]
        pub struct $name {
            #[doc = "Successfully fetched data, keyed by symbol"]
            pub $field: std::collections::HashMap<String, $type>,
            #[doc = "Symbols that failed to fetch, with error messages"]
            pub errors: std::collections::HashMap<String, String>,
        }

        impl $name {
            pub(crate) fn with_capacity(capacity: usize) -> Self {
                Self {
                    $field: std::collections::HashMap::with_capacity(capacity),
                    errors: std::collections::HashMap::with_capacity(capacity),
                }
            }

            #[doc = "Number of successfully fetched items"]
            pub fn success_count(&self) -> usize {
                self.$field.len()
            }

            #[doc = "Number of failed symbols"]
            pub fn error_count(&self) -> usize {
                self.errors.len()
            }

            #[doc = "Check if all symbols were successful"]
            pub fn all_successful(&self) -> bool {
                self.errors.is_empty()
            }
        }
    };
}

/// Generate a cached batch-fetch method body.
///
/// Handles the full cache-check → guard → double-check → concurrent-fetch →
/// cache-write → response-build skeleton. The caller provides the cache field,
/// guard strategy, cache key construction, response type, and fetch expression.
///
/// # Bindings in `fetch:`
///
/// The caller chooses identifier names via `fetch: |client, symbol| expr`:
/// - `client: Arc<YahooClient>` — cloned per-future from `$self.client`
/// - `symbol: Arc<str>` — the symbol being fetched
///
/// The fetch expression must evaluate to `crate::error::Result<V>` where `V`
/// is the value type stored in the cache / response.
///
/// # Guard variants
///
/// - `simple($field)` — acquires `$self.$field.lock().await` directly
/// - `map($field, $key)` — acquires via `Self::get_fetch_guard(&$self.$field, $key)`
///
/// # Example
///
/// ```ignore
/// pub async fn financials(
///     &self,
///     statement_type: StatementType,
///     frequency: Frequency,
/// ) -> Result<BatchFinancialsResponse> {
///     batch_fetch_cached!(self;
///         cache: financials_cache,
///         guard: map(financials_fetch, (statement_type, frequency)),
///         key: |s| (s.clone(), statement_type, frequency),
///         response: BatchFinancialsResponse.financials,
///         fetch: |client, symbol| client.get_financials(&symbol, statement_type, frequency).await,
///     )
/// }
/// ```
macro_rules! batch_fetch_cached {
    // Map guard: acquire via get_fetch_guard
    ($self:expr;
        cache: $cache_field:ident,
        guard: map($guard_field:ident, $guard_key:expr),
        $($rest:tt)*
    ) => {
        batch_fetch_cached!(@impl $self;
            cache: $cache_field,
            acquire_guard: {
                let __fetch_guard = Self::get_fetch_guard(&$self.$guard_field, $guard_key).await;
                let _guard = __fetch_guard.lock().await;
            },
            $($rest)*
        )
    };

    // Simple guard: lock directly
    ($self:expr;
        cache: $cache_field:ident,
        guard: simple($guard_field:ident),
        $($rest:tt)*
    ) => {
        batch_fetch_cached!(@impl $self;
            cache: $cache_field,
            acquire_guard: {
                let _guard = $self.$guard_field.lock().await;
            },
            $($rest)*
        )
    };

    // Internal implementation shared by both guard variants
    (@impl $self:expr;
        cache: $cache_field:ident,
        acquire_guard: { $($guard_code:tt)* },
        key: |$ksym:ident| $key_expr:expr,
        response: $resp_ty:ident . $resp_field:ident,
        fetch: |$client:ident, $symbol:ident| $fetch_expr:expr $(,)?
    ) => {{
        let cache_key_fn = |$ksym: &std::sync::Arc<str>| $key_expr;

        // Fast path: check if all symbols are cached and fresh
        {
            let cache = $self.$cache_field.read().await;
            if $self.all_cached(&cache, $self.symbols.iter().map(&cache_key_fn)) {
                let mut response = $resp_ty::with_capacity($self.symbols.len());
                for symbol in &$self.symbols {
                    if let Some(entry) = cache.get(&cache_key_fn(symbol)) {
                        response.$resp_field.insert(symbol.to_string(), entry.value.clone());
                    }
                }
                return Ok(response);
            }
        }

        // Slow path: acquire guard to prevent duplicate concurrent requests
        $($guard_code)*

        // Double-check: another task may have fetched while we waited for the guard
        {
            let cache = $self.$cache_field.read().await;
            if $self.all_cached(&cache, $self.symbols.iter().map(&cache_key_fn)) {
                let mut response = $resp_ty::with_capacity($self.symbols.len());
                for symbol in &$self.symbols {
                    if let Some(entry) = cache.get(&cache_key_fn(symbol)) {
                        response.$resp_field.insert(symbol.to_string(), entry.value.clone());
                    }
                }
                return Ok(response);
            }
        }

        // Concurrent fetch (no lock held during I/O)
        let futures: Vec<_> = $self.symbols.iter().map(|sym_ref| {
            #[allow(unused_variables)]
            let $client = std::sync::Arc::clone(&$self.client);
            let $symbol = std::sync::Arc::clone(sym_ref);
            async move {
                let result: crate::error::Result<_> = (async { $fetch_expr }).await;
                ($symbol, result)
            }
        }).collect();

        let results: Vec<_> = futures::stream::iter(futures)
            .buffer_unordered($self.max_concurrency)
            .collect()
            .await;

        let mut response = $resp_ty::with_capacity($self.symbols.len());
        let mut parsed = Vec::new();

        for (sym, result) in results {
            match result {
                Ok(value) => parsed.push((sym, value)),
                Err(e) => { response.errors.insert(sym.to_string(), e.to_string()); }
            }
        }

        // Cache insert
        if $self.cache_ttl.is_some() {
            let mut cache = $self.$cache_field.write().await;
            for (sym, value) in &parsed {
                $self.cache_insert(&mut cache, cache_key_fn(sym), value.clone());
            }
        }

        // Build response
        for (sym, value) in parsed {
            response.$resp_field.insert(sym.to_string(), value);
        }

        Ok(response)
    }};
}

pub(crate) use batch_fetch_cached;
pub(crate) use define_batch_response;
