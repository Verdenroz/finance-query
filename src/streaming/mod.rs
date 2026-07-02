//! Real-time price streaming with pluggable provider backends.
//!
//! This module provides a Stream-based API for receiving real-time price updates,
//! similar to Kotlin Flow or Rx observables.
//!
//! # Overview
//!
//! A [`StreamSource`](source::StreamSource) trait abstracts the provider-specific
//! transport and wire protocol. Yahoo ([`YahooStreamSource`](yahoo::YahooStreamSource))
//! is the reference implementation, with additional providers (e.g. Polygon)
//! supported through the same abstraction.
//!
//! This module handles:
//!
//! - Provider-agnostic reconnection logic
//! - Subscription management with automatic heartbeats
//! - Protobuf message decoding (Yahoo)
//! - A clean Stream API for consuming updates
//!
//! # Example
//!
//! ```no_run
//! use finance_query::streaming::PriceStream;
//! use futures::StreamExt;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Subscribe to symbols
//! let mut stream = PriceStream::subscribe(&["AAPL", "NVDA", "TSLA"]).await?;
//!
//! // Process updates as they arrive
//! while let Some(price) = stream.next().await {
//!     println!("{}: ${:.2} ({:+.2}%)",
//!         price.id,
//!         price.price,
//!         price.change_percent
//!     );
//! }
//! # Ok(())
//! # }
//! ```

mod client;
mod news;
mod pricing;
mod source;
mod subscription;
mod yahoo;

pub use client::{PriceStream, PriceStreamBuilder, StreamError, StreamResult};
pub use news::{NewsStream, NewsStreamBuilder};
pub use pricing::{MarketHoursType, OptionType, PriceUpdate, QuoteType};
