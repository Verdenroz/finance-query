# finance-query-derive

Procedural macros for the [finance-query](https://crates.io/crates/finance-query) library.

## Usage

This crate is not meant to be used directly. Enable the `dataframe` feature in `finance-query`:

```toml
[dependencies]
finance-query = { version = "2.0", features = ["dataframe"] }
```

## What It Provides

`ToDataFrame` derive macro for converting structs to Polars DataFrames:

```rust
use finance_query::ToDataFrame;

#[derive(ToDataFrame)]
struct Quote {
    symbol: String,
    price: Option<f64>,
    volume: Option<i64>,
}

let quote = Quote { symbol: "AAPL".into(), price: Some(150.0), volume: Some(1000000) };
let df = quote.to_dataframe()?;
```

## License

MIT
