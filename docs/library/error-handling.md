# Error Handling

!!! abstract "Cargo Docs"
    [docs.rs/finance-query — FinanceError](https://docs.rs/finance-query/latest/finance_query/enum.FinanceError.html)

Finance Query uses a single `FinanceError` enum for all error cases. Every library method returns `Result<T, FinanceError>`.

## Error Variants

| Variant | When it occurs | Retriable |
|---------|---------------|-----------|
| `AuthenticationFailed` | Yahoo crumb/cookie auth failure, EDGAR init rejected | ✓ |
| `SymbolNotFound` | Ticker symbol doesn't exist or returned no data | ✗ |
| `RateLimited` | Provider rate limit exceeded | ✓ |
| `HttpError` | Transport-level HTTP failure (DNS, TLS, connection refused) | ✓ |
| `JsonParseError` | Provider returned malformed JSON | ✗ |
| `ResponseStructureError` | Missing or malformed response fields | ✗ |
| `InvalidParameter` | Bad input: invalid symbols, unsupported interval/range combo | ✗ |
| `Timeout` | Request timed out | ✓ |
| `ServerError` | Provider returned 5xx status | ✓ |
| `UnexpectedResponse` | Unexpected API response format | ✗ |
| `InternalError` | Internal library error | ✗ |
| `ApiError` | Generic API-level error | ✗ |
| `RuntimeError` | Tokio I/O error | ✓ |
| `MacroDataError` | FRED / Treasury data fetch or parse failure | ✗ |
| `FeedParseError` | RSS/Atom feed parse failure | ✗ |
| `NotSupported` | Provider doesn't support the requested operation | ✗ |
| `NoProviderAvailable` | No configured provider supports this operation | ✗ |

## Checking Error Types

```rust
use finance_query::FinanceError;

match &error {
    FinanceError::RateLimited { retry_after } => {
        if let Some(secs) = retry_after {
            tokio::time::sleep(std::time::Duration::from_secs(*secs)).await;
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

```rust
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
```

The built-in retriable variants are `RateLimited`, `Timeout`, `HttpError`, `AuthenticationFailed`, `ServerError`, and `RuntimeError`.

## Error Categorization

Use `category()` for logging and metrics:

```rust
use finance_query::{FinanceError, ErrorCategory};

match error.category() {
    ErrorCategory::Auth => tracing::warn!("Auth failure"),
    ErrorCategory::RateLimit => tracing::warn!("Rate limited, backing off"),
    ErrorCategory::Timeout => tracing::warn!("Timeout"),
    ErrorCategory::Server => tracing::error!("Upstream server error"),
    ErrorCategory::NotFound => tracing::info!("Symbol not found"),
    ErrorCategory::Validation => tracing::warn!("Invalid input: {}", error),
    ErrorCategory::Parsing => tracing::error!("Parse failure"),
    ErrorCategory::Other => tracing::error!("{}", error),
}
```

## Adding Context

Enrich errors with symbol and context using the fluent API:

```rust
use finance_query::FinanceError;

let error = FinanceError::SymbolNotFound {
    symbol: None,
    context: String::new(),
}
.with_symbol("AAPL")
.with_context("from batch quote fetch");
```

`with_symbol()` only sets the symbol on `SymbolNotFound` errors. `with_context()` sets context on `AuthenticationFailed`, `SymbolNotFound`, `ResponseStructureError`, and `ServerError`.

## Batch Response Errors

When using `Tickers`, per-symbol errors are stored in the response rather than returned as `Result::Err`:

```rust
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
```

## Provider-Specific Errors

When using multiple providers, `NotSupported` and `NoProviderAvailable` help diagnose missing capabilities:

```rust
match &error {
    FinanceError::NotSupported { provider, operation } => {
        // AlphaVantage doesn't do options — expected
        eprintln!("{} does not support {}", provider, operation);
    }
    FinanceError::NoProviderAvailable { operation } => {
        // No configured provider supports this operation
        eprintln!("No provider available for {}", operation);
    }
    _ => {}
}
```
