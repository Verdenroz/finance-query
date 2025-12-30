# Contributing

We welcome contributions to FinanceQuery! This guide will help you get started.

!!! warning "V1 (Python) is Not Maintained"

    The legacy Python implementation in `/v1` is no longer actively maintained. All development focuses on the Rust library and server. The v1 source code, workflow, and documentation remain available for reference.

## Quick Start

Clone and set up the development environment:

```bash
git clone https://github.com/Verdenroz/finance-query.git
cd finance-query
make install-dev  # Installs rustfmt, clippy, prek, sets up pre-commit hooks
```

This sets up [prek](https://github.com/j178/prek) (a faster Rust-based pre-commit) which runs `fmt`, `clippy`, and `check` automatically before each commit.

## Useful Commands

Run `make help` to see all available commands:

```bash
make serve             # Start dev server (PORT=8000 by default)
make test              # Run ALL tests including network integration tests
make test-fast         # Run only fast tests (excludes network tests)
make fix               # Auto-fix formatting and clippy issues
make lint              # Run pre-commit checks (fmt, clippy, check)
make audit             # Run security audit on dependencies
make docs              # Build and serve MkDocs documentation
make build             # Build library and server in release mode
make docker-compose    # Start v1 and v2 servers together
make clean             # Clean build artifacts
```

## Development Workflow

### 1. Make Your Changes

Work on the library (`src/`) or server (`server/src/`):

```bash
# Start the dev server
make serve

# Run tests as you work
make test-fast  # Quick tests only
make test       # All tests including network calls
```

### 2. Check Your Code

Before committing, run the pre-commit checks:

```bash
make fix   # Auto-fix formatting and clippy issues
make lint  # Verify all checks pass
```

### 3. Test Thoroughly

Run the appropriate tests for your changes:

```bash
# Library changes
cargo test -p finance-query

# Server changes
cargo test -p finance-query-server

# Specific test
cargo test test_ticker_quote

# Integration tests (makes real API calls)
cargo test -- --ignored
```

## Code Standards

### Write Idiomatic Rust

Use standard patterns and avoid unnecessary complexity:

```rust
// Good - simple and clear
pub async fn quote(&self) -> Result<Quote> {
    self.get_quote_data().await
}

// Bad - over-engineered
pub async fn quote(&self) -> Result<Quote, Box<dyn std::error::Error>> {
    match self.get_quote_data().await {
        Ok(data) => Ok(data),
        Err(e) => Err(Box::new(e)),
    }
}
```

### Document Public APIs

Add doc comments to public items:

```rust
/// Fetches the latest quote for the ticker.
///
/// # Example
///
/// ```no_run
/// use finance_query::Ticker;
///
/// let ticker = Ticker::new("AAPL").await?;
/// let quote = ticker.quote(true).await?;
/// println!("Price: ${}", quote.regular_market_price);
/// ```
pub async fn quote(&self, include_logo: bool) -> Result<Quote> {
    // ...
}
```

## Testing Guidelines

### Unit Tests

Keep tests focused and fast:

```rust
#[tokio::test]
async fn test_ticker_builder() {
    let ticker = Ticker::builder("AAPL")
        .region(Region::UnitedStates)
        .build()
        .await
        .unwrap();

    assert_eq!(ticker.symbol(), "AAPL");
}
```

### Integration Tests

Mark network tests with `#[ignore]`:

```rust
#[tokio::test]
#[ignore = "requires network access"]
async fn test_real_quote() {
    let ticker = Ticker::new("AAPL").await.unwrap();
    let quote = ticker.quote(true).await.unwrap();
    assert!(!quote.symbol.is_empty());
}
```

### Doc Tests

Use `no_run` for examples that require network access:

```rust
/// # Example
///
/// ```no_run
/// # use finance_query::Ticker;
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let ticker = Ticker::new("AAPL").await?;
/// let quote = ticker.quote(true).await?;
/// # Ok(())
/// # }
/// ```
```

## Submitting Changes

### 1. Create a Branch

Use descriptive branch names:

```bash
git checkout -b fix/quote-timezone-handling
git checkout -b feat/add-options-chain
```

### 2. Commit Your Changes

Write clear commit messages:

```bash
git add .
git commit -m "fix: handle timezone correctly in market hours"
```

### 3. Push and Create PR

```bash
git push origin fix/quote-timezone-handling
```

Open a pull request on GitHub with:

- Clear description of what changed and why
- Link to any related issues
- Test results showing everything passes

## Common Tasks

### Adding a New Endpoint

**Library side:**

1. Add endpoint URL in `src/endpoints/`:

```rust
// src/endpoints/quote.rs
pub fn options_chain(symbol: &str) -> String {
    format!("{}/v7/finance/options/{}", BASE_URL, symbol)
}
```

2. Define model in `src/models/`:

```rust
// src/models/options.rs
#[derive(Debug, Clone, Deserialize)]
pub struct OptionsChain {
    pub symbol: String,
    pub expiration_dates: Vec<i64>,
    // ...
}
```

3. Add method to `Ticker`:

```rust
// src/ticker/core.rs
pub async fn options(&self) -> Result<OptionsChain> {
    let url = endpoints::options_chain(&self.symbol);
    self.client.fetch_json(&url).await
}
```

**Server side:**

```rust
// server/src/main.rs
async fn get_options(
    Path(symbol): Path<String>,
) -> Result<Json<OptionsChain>, AppError> {
    let ticker = Ticker::new(&symbol).await?;
    let options = ticker.options().await?;
    Ok(Json(options))
}

// Register route
.route("/v2/options/{symbol}", get(get_options))
```

### Updating Dependencies

Check for outdated dependencies:

```bash
cargo outdated
cargo update
make test  # Ensure everything still works
```

## Getting Help

- **Questions**: Open a GitHub Discussion
- **Bugs**: Open a GitHub Issue with reproduction steps
- **Security**: Email security@finance-query.dev (not public issues)

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
