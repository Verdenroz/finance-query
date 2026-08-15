# Real-time Streaming

!!! abstract "Cargo Docs"
    [docs.rs/finance-query — streaming](https://docs.rs/finance-query/latest/finance_query/streaming/index.html)

Subscribe to live price updates via WebSocket. The streaming API uses a Flow-like `Stream` interface compatible with Rust's `futures` ecosystem.

## Quick Start

```rust no_run covers=finance_query::streaming::pricing::PriceUpdate
use finance_query::streaming::PriceStream;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = PriceStream::subscribe(["AAPL", "NVDA", "TSLA"]).await?;

    while let Some(price) = stream.next().await {
        println!("{}: ${:.2} ({:+.2}%)",
            price.id,
            price.price,
            price.change_percent
        );
    }
    Ok(())
}
```

<!-- soothfast:claim finance_query::stream_serialize.walltime.median_ns < 5000 -->
<!-- soothfast:claim finance_query::stream_serialize.alloc.allocs <= 4 -->
- Serializing a `PriceUpdate` tick to JSON stays **under 5 µs** and makes
  exactly **4 allocations** — cheap enough to fan out to many consumers on
  every tick.

<!-- soothfast:claim finance_query::stream_deserialize.walltime.median_ns < 5000 -->
- Decoding a JSON tick back into a `PriceUpdate` also stays **under 5 µs**.

## Subscribing

### Simple Subscribe

```rust no_run
use finance_query::streaming::PriceStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = PriceStream::subscribe(["AAPL", "GOOGL"]).await?;
    Ok(())
}
```

### Builder Pattern

```rust no_run
use finance_query::streaming::PriceStreamBuilder;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = PriceStreamBuilder::new()
        .symbols(["AAPL", "MSFT", "NVDA"])
        .retry(Duration::from_secs(5))
        .build()
        .await?;
    Ok(())
}
```

## Dynamic Subscriptions

Add or remove symbols after the stream is created:

```rust no_run
use finance_query::streaming::PriceStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stream = PriceStream::subscribe(["AAPL"]).await?;

    // Add more symbols
    stream.add_symbols(["NVDA", "TSLA"]).await;

    // Remove symbols
    stream.remove_symbols(["AAPL"]).await;
    Ok(())
}
```

## Multiple Consumers

Use `resubscribe()` to create additional receivers sharing the same WebSocket connection:

```rust no_run
use finance_query::streaming::PriceStream;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stream1 = PriceStream::subscribe(["AAPL", "NVDA"]).await?;
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
    Ok(())
}
```

## PriceUpdate Fields

Each update yielded by the stream contains:

<!-- soothfast:bind finance_query::streaming::pricing::PriceUpdate -->

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

<!-- /soothfast:bind -->

## Filtering Updates

```rust no_run
use finance_query::streaming::{MarketHoursType, PriceStream};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = PriceStream::subscribe(["AAPL", "MSFT", "GOOGL"]).await?;

    while let Some(price) = stream.next().await {
        // Only process regular market updates
        if price.market_hours == MarketHoursType::RegularMarket {
            println!("{}: ${:.2}", price.id, price.price);
        }
    }
    Ok(())
}
```

## Closing the Stream

```rust no_run
use finance_query::streaming::PriceStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stream = PriceStream::subscribe(["AAPL"]).await?;

    // ... use stream ...

    stream.close().await;
    Ok(())
}
```

!!! info "Notes"
    - **Reconnection**: The stream automatically reconnects with a 3-second backoff on connection loss.
    - **Heartbeats**: Subscriptions are refreshed every 15 seconds to keep the connection alive.
    - **Market hours**: Updates are sent during pre-market, regular, and post-market sessions.
    - **Data availability**: Not all fields are populated for every update — Yahoo only sends changed values.

## News Streaming

<!-- soothfast:bind finance_query::streaming::news::NewsStream -->
`NewsStream` gives RSS/Atom feeds (see [Feeds](feeds.md)) the same `Stream` interface as `PriceStream`. Since RSS/Atom has no server push, it works by polling the configured sources on an interval instead of holding a WebSocket connection — yielding an initial batch of entries on subscribe, then only newly-seen ones (deduplicated by URL) on each subsequent poll.
<!-- /soothfast:bind -->

```rust no_run covers=finance_query::streaming::news::NewsStream
use finance_query::streaming::NewsStream;
use finance_query::feeds::FeedSource;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let mut stream =
        NewsStream::subscribe([FeedSource::Bloomberg, FeedSource::MarketWatch]).await;

    while let Some(entry) = stream.next().await {
        println!("[{}] {}", entry.source, entry.title);
    }
}
```

### Custom Poll Interval

Building a stream never blocks on the network by itself (the poll loop runs in
the background), so this example runs as a real test:

```rust
use finance_query::streaming::NewsStreamBuilder;
use finance_query::feeds::FeedSource;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let stream = NewsStreamBuilder::new()
        .sources(vec![FeedSource::FederalReserve, FeedSource::SecPressReleases])
        .poll_interval(Duration::from_secs(60))
        .build()
        .await;

    // ... consume stream.next() as in the examples above ...

    stream.close().await;
}
```

The default poll interval is 5 minutes.

### Dynamic Sources and Multiple Consumers

`add_sources`, `remove_sources`, `resubscribe`, and `close` work the same way as on `PriceStream`:

```rust no_run
use finance_query::streaming::NewsStream;
use finance_query::feeds::FeedSource;

#[tokio::main]
async fn main() {
    let stream = NewsStream::subscribe([FeedSource::Bloomberg]).await;

    stream.add_sources([FeedSource::WsjMarkets]).await;
    stream.remove_sources([FeedSource::Bloomberg]).await;

    let other_consumer = stream.resubscribe();

    stream.close().await;
}
```

## Next Steps

- [Ticker API](ticker.md) - Fetch snapshot quotes and historical data for the same symbols
- [Feeds](feeds.md) - The one-shot `fetch`/`fetch_all` API that `NewsStream` polls under the hood
- [Finance Module](finance.md) - Market summary, trending tickers, and sector data
- [Configuration](configuration.md) - Proxy and timeout settings that apply to the WebSocket connection
