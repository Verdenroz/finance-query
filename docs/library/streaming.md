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

<!-- soothfast:claim finance_query::stream_serialize.perfcnt.instructions < 17000 -->
<!-- soothfast:claim finance_query::stream_serialize.alloc.allocs <= 4 -->
- Serializing a `PriceUpdate` tick to JSON stays **under 17 thousand CPU instructions** and makes
  exactly **4 allocations** — cheap enough to fan out to many consumers on
  every tick.

<!-- soothfast:claim finance_query::stream_deserialize.perfcnt.instructions < 21000 -->
- Decoding a JSON tick back into a `PriceUpdate` also stays **under 21 thousand CPU instructions**.

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
| `price` | `f64` | Current price |
| `change` | `f64` | Price change from previous close |
| `change_percent` | `f64` | Percent change from previous close |
| `day_high` | `f64` | Day's high price |
| `day_low` | `f64` | Day's low price |
| `day_volume` | `i64` | Day's trading volume |
| `open_price` | `f64` | Opening price |
| `previous_close` | `f64` | Previous close price |
| `short_name` | `String` | Short name/description |
| `currency` | `String` | Currency code (e.g., `"USD"`) |
| `exchange` | `String` | Exchange code (e.g., `"NMS"`) |
| `quote_type` | `QuoteType` | Asset type (Equity, Etf, Cryptocurrency, etc.) |
| `market_hours` | `MarketHoursType` | Session (PreMarket, RegularMarket, PostMarket) |
| `time` | `i64` | Unix timestamp in milliseconds, a `Datetime(us, UTC)` column in frames |

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

## Alerts

`AlertExt::alerts` wraps any `PriceStream` (or any `Stream<Item = PriceUpdate>`) and yields an `AlertEvent` only when a rule fires, so you stop filtering every tick by hand. Each `AlertRule` pairs a symbol with an `AlertCondition`; a rule fires once unless you call `repeating()`, which re-arms it whenever the condition stops holding.

```rust no_run covers=finance_query::streaming::alerts::AlertEvent
use finance_query::streaming::{AlertCondition, AlertExt, AlertRule, PriceStream};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut alerts = PriceStream::subscribe(["AAPL", "NVDA"])
        .await?
        .alerts([
            AlertRule::new("AAPL", AlertCondition::CrossesAbove(200.0)),
            AlertRule::new("NVDA", AlertCondition::PercentChangeBelow(-3.0)).repeating(),
        ]);

    while let Some(alert) = alerts.next().await {
        println!("{} at {}", alert.update.id, alert.update.price);
    }
    Ok(())
}
```

The conditions are `CrossesAbove`, `CrossesBelow`, `PriceAbove`, `PriceBelow`, `PercentChangeAbove`, `PercentChangeBelow` and `VolumeAbove`. The crossing conditions compare against the last price seen for the symbol, so they never fire on a symbol's first tick.

If you already own a stream, `AlertEvaluator` applies the same rules to one tick at a time:

<!-- soothfast:claim finance_query::alerts_evaluate.perfcnt.instructions < 58000 -->
- Evaluating 32 ticks against five rules stays **under 58 thousand CPU instructions**, so the check is cheap enough to run on every tick.

<!-- soothfast:bind finance_query::streaming::alerts::AlertEvent -->

| Field | Type | Description |
|-------|------|-------------|
| `condition` | `AlertCondition` | The condition that fired |
| `previous_price` | `Option<f64>` | Last price seen before this tick, if any |
| `update` | `PriceUpdate` | The full tick that triggered the alert |

<!-- /soothfast:bind -->

## Order Book Streaming

`DepthStream` pushes level-2 order-book depth: the full ladder of price levels on each side, where `PriceUpdate` carries only the top of book. It is backed by Polygon's crypto level-2 feed, so pairs are crypto pairs such as `"BTC-USD"`. It needs the `polygon` feature and `POLYGON_API_KEY` in the environment.

```rust no_run feature=polygon covers=finance_query::streaming::book::OrderBookUpdate
use finance_query::streaming::DepthStream;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut books = DepthStream::subscribe(["BTC-USD"]).await?;

    while let Some(book) = books.next().await {
        println!("{} spread {:?} mid {:?}", book.symbol, book.spread(), book.mid());
    }
    Ok(())
}
```

`DepthStreamBuilder` sets the reconnect delay and attempt limit, and `add_pairs` / `remove_pairs` change the subscription after the stream starts.

<!-- soothfast:bind finance_query::streaming::book::OrderBookUpdate -->

| Field | Type | Description |
|-------|------|-------------|
| `symbol` | `String` | Symbol or pair the book belongs to |
| `bids` | `Vec<BookLevel>` | Bid levels, highest price first |
| `asks` | `Vec<BookLevel>` | Ask levels, lowest price first |
| `exchange` | `Option<i32>` | Exchange identifier, where the venue reports one |
| `time` | `i64` | Update timestamp in milliseconds |

<!-- /soothfast:bind -->

Each `BookLevel` has a `price` and the total `size` resting at it. `best_bid`, `best_ask`, `spread`, `mid` and `depth` read the top of book and the total size per side.

## Trade Streaming

`TradeStream` pushes every executed trade print for the symbols you subscribe to, where `PriceStream` coalesces activity into a last-price tick. Use it for execution-quality and microstructure work. It needs the `polygon` feature and `POLYGON_API_KEY`.

```rust no_run feature=polygon covers=finance_query::streaming::trades::TradeTick
use finance_query::streaming::TradeStream;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut trades = TradeStream::subscribe(["AAPL"]).await?;

    while let Some(trade) = trades.next().await {
        println!("{} {} @ {} (notional {:.2})",
            trade.symbol, trade.size, trade.price, trade.notional());
    }
    Ok(())
}
```

`TradeStreamBuilder::asset_class` switches the feed to another `AssetClass`, and `add_symbols` / `remove_symbols` change the subscription after the stream starts.

<!-- soothfast:bind finance_query::streaming::trades::TradeTick -->

| Field | Type | Description |
|-------|------|-------------|
| `symbol` | `String` | Symbol or pair the trade executed on |
| `price` | `f64` | Execution price |
| `size` | `f64` | Executed size (shares, contracts, or base-asset units) |
| `exchange` | `Option<i32>` | Exchange identifier, where the venue reports one |
| `conditions` | `Vec<i32>` | Trade condition codes |
| `trade_id` | `Option<String>` | Provider trade identifier, where one is supplied |
| `time` | `i64` | Execution timestamp in milliseconds |

<!-- /soothfast:bind -->

## Options Chain Streaming

`OptionsChainStream` follows live options quotes. Subscribe by underlying (`"AAPL"`) to follow the whole chain, or by OCC symbol (`"O:AAPL250117C00150000"`) to follow single contracts. Bid, ask and last come from the WebSocket; greeks and open interest come from a REST snapshot refreshed every 60 seconds by default. It needs the `polygon` feature and `POLYGON_API_KEY`.

```rust no_run feature=polygon covers=finance_query::streaming::options::OptionContractUpdate
use finance_query::streaming::OptionsChainStream;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = OptionsChainStream::subscribe(["AAPL"]).await?;

    while let Some(contract) = stream.next().await {
        println!("{} {:?}/{:?}", contract.contract_symbol, contract.bid, contract.ask);
    }
    Ok(())
}
```

`OptionsChainStreamBuilder::greeks_refresh` changes the snapshot interval, and `None` turns the snapshot off so only WebSocket fields arrive.

<!-- soothfast:bind finance_query::streaming::options::OptionContractUpdate -->

| Field | Type | Description |
|-------|------|-------------|
| `contract_symbol` | `String` | OCC contract symbol |
| `underlying` | `String` | Underlying symbol |
| `expiration` | `Option<i64>` | Expiration as a Unix timestamp in seconds |
| `strike` | `Option<f64>` | Strike price |
| `option_type` | `Option<OptionType>` | Call or put |
| `bid` | `Option<f64>` | Best bid |
| `bid_size` | `Option<f64>` | Size at the best bid |
| `ask` | `Option<f64>` | Best ask |
| `ask_size` | `Option<f64>` | Size at the best ask |
| `last_price` | `Option<f64>` | Last trade price |
| `last_size` | `Option<f64>` | Last trade size |
| `volume` | `Option<i64>` | Day volume |
| `open_interest` | `Option<i64>` | Open interest |
| `implied_volatility` | `Option<f64>` | Implied volatility |
| `greeks` | `Option<Greeks>` | Delta, gamma, theta and vega |
| `time` | `i64` | Update timestamp in milliseconds |

<!-- /soothfast:bind -->

## Economic Series Streaming

`EconomicStream` polls FRED series on an interval (15 minutes by default) and yields a `SeriesUpdate` only when a series' latest observation is new or revised. Macro data has no push transport, so this is a poll loop. It needs the `fred` feature and [`fred::init`](providers/fred.md).

```rust no_run feature=fred covers=finance_query::streaming::economic::SeriesUpdate
use finance_query::streaming::EconomicStream;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let mut stream = EconomicStream::subscribe(["FEDFUNDS", "CPIAUCSL"]).await;

    while let Some(release) = stream.next().await {
        println!("{} = {:?} ({})", release.series_id, release.value, release.date);
    }
}
```

`EconomicStreamBuilder::poll_interval` changes the interval, and `add_series` / `remove_series` change the subscription after the stream starts.

<!-- soothfast:bind finance_query::streaming::economic::SeriesUpdate -->

| Field | Type | Description |
|-------|------|-------------|
| `series_id` | `String` | Series identifier (e.g., `"FEDFUNDS"`) |
| `date` | `String` | Observation date as `YYYY-MM-DD` |
| `value` | `Option<f64>` | Newly published value, or `None` when the source reports a gap |
| `previous_value` | `Option<f64>` | Value this release replaced |
| `revision` | `bool` | `true` when the same observation date was re-published with a new value |
| `observed_at` | `i64` | Unix timestamp in seconds at which the release was observed |

<!-- /soothfast:bind -->

## Next Steps

- [Ticker API](ticker.md) - Fetch snapshot quotes and historical data for the same symbols
- [Feeds](feeds.md) - The one-shot `fetch`/`fetch_all` API that `NewsStream` polls under the hood
- [Finance Module](finance.md) - Market summary, trending tickers, and sector data
- [Configuration](configuration.md) - Proxy and timeout settings that apply to the WebSocket connection
