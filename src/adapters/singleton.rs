//! Shared `macro_rules!` scaffolding for the process-global provider
//! singletons (Alpha Vantage, FMP, Polygon, FRED).
//!
//! Each of those adapters stores `{api_key, timeout, limiter}` in a
//! `OnceLock` set exactly once (via `init`/`init_with_timeout`), and builds a
//! fresh `reqwest`-backed client per call by reading that state back out
//! (`reqwest::Client` is runtime-bound, so it can't itself be cached in the
//! singleton — see the FRED/EDGAR module docs for the full rationale). These
//! macros generate the identical boilerplate; the public `init`/
//! `init_with_timeout` functions (and their doc comments) stay hand-written
//! at each call site since their docs genuinely differ per provider.

/// Generates the `{name}` singleton struct, its `OnceLock` static, and a
/// private `set_singleton` helper that stores the API key/timeout/rate
/// limiter exactly once, mapping a repeat call to the standard
/// `FinanceError::InvalidParameter` "already initialized" error.
///
/// Call `set_singleton(api_key, timeout)` from the provider's own
/// `init_with_timeout` to keep that function's public doc comment
/// hand-written and provider-specific.
macro_rules! provider_singleton_state {
    (
        name = $struct_name:ident,
        static_name = $static_name:ident,
        rate_const = $rate_const:ident,
        provider_key = $provider_key:literal,
        already_init_reason = $already_init_reason:literal $(,)?
    ) => {
        struct $struct_name {
            api_key: ::std::string::String,
            timeout: ::std::time::Duration,
            limiter: ::std::sync::Arc<crate::rate_limiter::RateLimiter>,
        }

        static $static_name: ::std::sync::OnceLock<$struct_name> = ::std::sync::OnceLock::new();

        /// Store the API key/timeout/rate-limiter exactly once; a second call
        /// returns the standard "already initialized" error.
        fn set_singleton(
            api_key: impl Into<::std::string::String>,
            timeout: ::std::time::Duration,
        ) -> crate::error::Result<()> {
            $static_name
                .set($struct_name {
                    api_key: api_key.into(),
                    timeout,
                    limiter: ::std::sync::Arc::new(crate::rate_limiter::RateLimiter::new(
                        $rate_const,
                    )),
                })
                .map_err(|_| crate::error::FinanceError::InvalidParameter {
                    param: $provider_key.to_string(),
                    reason: $already_init_reason.to_string(),
                })
        }
    };
}

/// Generates `build_client()` and `build_test_client()` for a provider whose
/// client is constructed via `$builder::new(api_key).timeout(t).build_with_limiter(limiter)`.
///
/// `build_client()` falls back to reading `$env_var` if [`init`]/
/// [`init_with_timeout`] was never called (convenience for scripts/tests that
/// only set the env var), then reads the shared singleton state back out to
/// build a fresh client. Requires [`provider_singleton_state!`] to have been
/// invoked first in the same module (uses its `$struct_name`/`$static_name`).
macro_rules! provider_build_client {
    (
        name = $struct_name:ident,
        static_name = $static_name:ident,
        rate_const = $rate_const:ident,
        provider_key = $provider_key:literal,
        env_var = $env_var:literal,
        env_missing_reason = $env_missing_reason:literal,
        builder = $builder:path,
        client_ty = $client_ty:ty $(,)?
    ) => {
        /// Build a fresh client from the singleton state.
        ///
        /// Used internally by all query functions.
        pub(crate) fn build_client() -> crate::error::Result<$client_ty> {
            if $static_name.get().is_none()
                && let Ok(key) = ::std::env::var($env_var)
            {
                let _ = $static_name.set($struct_name {
                    api_key: key,
                    timeout: ::std::time::Duration::from_secs(30),
                    limiter: ::std::sync::Arc::new(crate::rate_limiter::RateLimiter::new(
                        $rate_const,
                    )),
                });
            }
            let s =
                $static_name
                    .get()
                    .ok_or_else(|| crate::error::FinanceError::InvalidParameter {
                        param: $provider_key.to_string(),
                        reason: $env_missing_reason.to_string(),
                    })?;
            <$builder>::new(&s.api_key)
                .timeout(s.timeout)
                .build_with_limiter(::std::sync::Arc::clone(&s.limiter))
        }

        /// Build a test client pointing at a mock server URL.
        #[cfg(test)]
        pub(crate) fn build_test_client(base_url: &str) -> crate::error::Result<$client_ty> {
            <$builder>::new("test-key")
                .timeout(::std::time::Duration::from_secs(5))
                .base_url(base_url)
                .build_with_limiter(::std::sync::Arc::new(
                    crate::rate_limiter::RateLimiter::new(100.0),
                ))
        }
    };
}

pub(crate) use provider_build_client;
pub(crate) use provider_singleton_state;
