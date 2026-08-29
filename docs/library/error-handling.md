# Error Handling

!!! abstract "Cargo Docs"
    [docs.rs/finance-query — FinanceError](https://docs.rs/finance-query/latest/finance_query/enum.FinanceError.html)

Finance Query uses a single `FinanceError` enum for all error cases. Every library method returns `Result<T, FinanceError>`.

## Error Variants

<!-- soothfast:bind finance_query::error::FinanceError -->

| Variant | When it occurs | Retriable |
|---------|---------------|-----------|
| `AuthenticationFailed` | Yahoo crumb/cookie auth failure, EDGAR init rejected | ✓ |
| `SymbolNotFound` | Ticker symbol doesn't exist or returned no data | ✗ |
| `RateLimited` | Provider rate limit exceeded | ✓ |
| `HttpError` | Transport-level HTTP failure (DNS, TLS, connection refused) | ✓ |
| `NetworkError` | Same, from a provider that authenticates by query string, so the source URL is withheld | ✓ |
| `JsonParseError` | Provider returned malformed JSON | ✗ |
| `ResponseStructureError` | Missing or malformed response fields | ✗ |
| `InvalidParameter` | Bad input: invalid symbols, unsupported interval/range combo | ✗ |
| `Timeout` | Request timed out | ✓ |
| `ServerError` | Provider returned 5xx status | ✓ |
| `UnexpectedResponse` | Unexpected API response format | ✗ |
| `InternalError` | Internal library error | ✗ |
| `ApiError` | Generic API-level error | ✗ |
| `RuntimeError` | Tokio I/O error | ✓ |
| `IndicatorError` | Indicator calculation failure (`indicators` feature) | ✗ |
| `ExternalApiError` | External (non-Yahoo) API returned an HTTP error status | ✓ (5xx only) |
| `MacroDataError` | FRED / Treasury data fetch or parse failure | ✗ |
| `FeedParseError` | RSS/Atom feed parse failure | ✗ |
| `TranslationError` | Translation backend failure (`translation` feature) | ✗ |
| `NotSupported` | Provider doesn't support the requested operation | ✗ |
| `NoProviderAvailable` | No configured provider supports this operation | ✗ |
| `ProviderNotRegistered` | A route names a provider id no adapter was registered for. Raised by `build()`, so a misconfigured route fails at startup rather than on the first request | ✗ |

<!-- /soothfast:bind -->

## Checking Error Types

Match on the variant to react to specific failures. This example runs as a real test — no network needed:

```rust covers=finance_query::error::FinanceError
use finance_query::FinanceError;

let error = FinanceError::RateLimited { retry_after: Some(2) };

match &error {
    FinanceError::RateLimited { retry_after } => {
        if let Some(secs) = retry_after {
            // In async code, back off before retrying:
            // tokio::time::sleep(std::time::Duration::from_secs(*secs)).await;
            assert_eq!(*secs, 2);
        }
    }
    FinanceError::SymbolNotFound { symbol, .. } => {
        eprintln!("Symbol not found: {:?}", symbol);
    }
    _ => eprintln!("{}", error),
}
```

## Retry Logic

Use `is_retriable()` and `retry_after_secs()` to implement exponential backoff:

```rust capture-output
use finance_query::FinanceError;

fn should_retry(error: &FinanceError, attempt: u32) -> bool {
    if !error.is_retriable() || attempt > 3 {
        return false;
    }
    true
}

fn retry_delay(error: &FinanceError) -> std::time::Duration {
    let base = error.retry_after_secs().unwrap_or(1);
    std::time::Duration::from_secs(base)
}

let rate_limited = FinanceError::RateLimited {
    retry_after: Some(10),
};
println!("attempt 1 retriable: {}", should_retry(&rate_limited, 1));
println!("attempt 4 retriable: {}", should_retry(&rate_limited, 4)); // give up after 3 attempts
println!("retry delay: {:?}", retry_delay(&rate_limited));

assert!(should_retry(&rate_limited, 1));
assert!(!should_retry(&rate_limited, 4));
assert_eq!(retry_delay(&rate_limited).as_secs(), 10);
```

```text soothfast-output
attempt 1 retriable: true
attempt 4 retriable: false
retry delay: 10s
```

The built-in retriable variants are `RateLimited`, `Timeout`, `HttpError`, `NetworkError`, `AuthenticationFailed`, `ServerError`, and `RuntimeError`.

## Error Categorization

<!-- soothfast:bind finance_query::error::ErrorCategory -->
Use `category()` for logging and metrics: every variant maps to one of eight
`ErrorCategory` values — `Auth`, `RateLimit`, `Timeout`, `Server`, `NotFound`,
`Validation`, `Parsing`, and `Other`.
<!-- /soothfast:bind -->

```rust capture-output covers=finance_query::error::ErrorCategory
use finance_query::{ErrorCategory, FinanceError};

let error = FinanceError::RateLimited { retry_after: None };

let log_line = match error.category() {
    ErrorCategory::Auth => "warn: auth failure",
    ErrorCategory::RateLimit => "warn: rate limited, backing off",
    ErrorCategory::Timeout => "warn: timeout",
    ErrorCategory::Server => "error: upstream server error",
    ErrorCategory::NotFound => "info: symbol not found",
    ErrorCategory::Validation => "warn: invalid input",
    ErrorCategory::Parsing => "error: parse failure",
    ErrorCategory::Other => "error: unclassified failure",
};
assert_eq!(log_line, "warn: rate limited, backing off");
println!("category = {:?}", error.category());
println!("log_line = {log_line:?}");
```

```text soothfast-output
category = RateLimit
log_line = "warn: rate limited, backing off"
```

## Adding Context

Enrich errors with symbol and context using the fluent API:

```rust capture-output
use finance_query::FinanceError;

let error = FinanceError::SymbolNotFound {
    symbol: None,
    context: String::new(),
}
.with_symbol("AAPL")
.with_context("from batch quote fetch");

match &error {
    FinanceError::SymbolNotFound { symbol, context } => {
        println!("symbol: {:?}", symbol);
        println!("context: {}", context);
        assert_eq!(symbol.as_deref(), Some("AAPL"));
        assert_eq!(context, "from batch quote fetch");
    }
    _ => unreachable!(),
}
```

```text soothfast-output
symbol: Some("AAPL")
context: from batch quote fetch
```

`with_symbol()` only sets the symbol on `SymbolNotFound` errors. `with_context()` sets context on `AuthenticationFailed`, `SymbolNotFound`, `ResponseStructureError`, and `ServerError`.

## Batch Response Errors

When using `Tickers`, per-symbol errors are stored in the response rather than returned as `Result::Err`:

```rust no_run
use finance_query::Tickers;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tickers = Tickers::builder(vec!["AAPL", "MSFT", "NOTREAL"]).build().await?;
    let response = tickers.quotes().await?;

    // Successful fetch — but individual symbols may have failed
    for (symbol, error_str) in &response.errors {
        eprintln!("{}: {}", symbol, error_str);
    }

    // Convenience methods
    println!("Success: {}, Failed: {}", response.success_count(), response.error_count());
    if !response.all_successful() {
        eprintln!("Some symbols failed");
    }
    Ok(())
}
```

## Provider-Specific Errors

When using multiple providers, `NotSupported` and `NoProviderAvailable` help diagnose missing capabilities. Both carry a `candidates` list of other providers that declare the capability:

```rust
use finance_query::{FinanceError, Operation, Provider};

let error = FinanceError::NotSupported {
    provider: Provider::Yahoo,
    operation: Operation::Quote,
    candidates: Vec::new(),
};

match &error {
    FinanceError::NotSupported { provider, operation, .. } => {
        // e.g. AlphaVantage doesn't do options — expected
        eprintln!("{} does not support {}", provider, operation);
    }
    FinanceError::NoProviderAvailable { operation, .. } => {
        // No configured provider supports this operation
        eprintln!("No provider available for {:?}", operation);
    }
    _ => {}
}
```
