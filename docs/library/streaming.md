# Real-time Streaming

!!! abstract "Cargo Docs"
    [docs.rs/finance-query — streaming](https://docs.rs/finance-query/latest/finance_query/streaming/index.html)

Subscribe to live price updates from Yahoo Finance via WebSocket. The streaming API uses a Flow-like `Stream` interface compatible with Rust's `futures` ecosystem.

## Quick Start

```rust
use finance_query::streaming::PriceStream;
use futures::StreamExt;

let mut stream = PriceStream::subscribe(&["AAPL", "NVDA", "TSLA"]).await?;

while let Some(price) = stream.next().await {
    println!("{}: ${:.2} ({:+.2}%)",
        price.id,
        price.price,
        price.change_percent
    );
}
```

## Subscribing

### Simple Subscribe

```rust
use finance_query::streaming::PriceStream;

let mut stream = PriceStream::subscribe(&["AAPL", "GOOGL"]).await?;
```

### Builder Pattern

```rust
use finance_query::streaming::PriceStreamBuilder;
use std::time::Duration;

let mut stream = PriceStreamBuilder::new()
    .symbols(&["AAPL", "MSFT", "NVDA"])
    .reconnect_delay(Duration::from_secs(5))
    .build()
    .await?;
```

## Dynamic Subscriptions

Add or remove symbols after the stream is created:

```rust
use finance_query::streaming::PriceStream;

let stream = PriceStream::subscribe(&["AAPL"]).await?;

// Add more symbols
stream.add_symbols(&["NVDA", "TSLA"]).await;

// Remove symbols
stream.remove_symbols(&["AAPL"]).await;
```

## Multiple Consumers

Use `resubscribe()` to create additional receivers sharing the same WebSocket connection:

```rust
use finance_query::streaming::PriceStream;
use futures::StreamExt;

let mut stream1 = PriceStream::subscribe(&["AAPL", "NVDA"]).await?;
let mut stream2 = stream1.resubscribe();

// Both streams receive the same updates
tokio::spawn(async move {
    while let Some(price) = stream2.next().await {
        println!("Consumer 2: {} ${:.2}", price.id, price.price);
    }
});

while let Some(price) = stream1.next().await {
    println!("Consumer 1: {} ${:.2}", price.id, price.price);
}
```

## PriceUpdate Fields

Each update yielded by the stream contains:

| Field | Type | Description |
|-------|------|-------------|
| `id` | `String` | Ticker symbol (e.g., `"AAPL"`) |
| `price` | `f32` | Current price |
| `change` | `f32` | Price change from previous close |
| `change_percent` | `f32` | Percent change from previous close |
| `day_high` | `f32` | Day's high price |
| `day_low` | `f32` | Day's low price |
| `day_volume` | `i64` | Day's trading volume |
| `open_price` | `f32` | Opening price |
| `previous_close` | `f32` | Previous close price |
| `short_name` | `String` | Short name/description |
| `currency` | `String` | Currency code (e.g., `"USD"`) |
| `exchange` | `String` | Exchange code (e.g., `"NMS"`) |
| `quote_type` | `QuoteType` | Asset type (Equity, Etf, Cryptocurrency, etc.) |
| `market_hours` | `MarketHoursType` | Session (PreMarket, RegularMarket, PostMarket) |
| `time` | `i64` | Unix timestamp in milliseconds |

## Filtering Updates

```rust
use finance_query::streaming::{PriceStream, MarketHoursType};
use futures::StreamExt;

let mut stream = PriceStream::subscribe(&["AAPL", "MSFT", "GOOGL"]).await?;

while let Some(price) = stream.next().await {
    // Only process regular market updates
    if price.market_hours == MarketHoursType::RegularMarket {
        println!("{}: ${:.2}", price.id, price.price);
    }
}
```

## Closing the Stream

```rust
use finance_query::streaming::PriceStream;

let stream = PriceStream::subscribe(&["AAPL"]).await?;

// ... use stream ...

stream.close().await;
```

!!! info "Notes"
    - **Reconnection**: The stream automatically reconnects with a 3-second backoff on connection loss.
    - **Heartbeats**: Subscriptions are refreshed every 15 seconds to keep the connection alive.
    - **Market hours**: Updates are sent during pre-market, regular, and post-market sessions.
    - **Data availability**: Not all fields are populated for every update — Yahoo only sends changed values.
