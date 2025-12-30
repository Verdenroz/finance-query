//! Real-time price streaming from Yahoo Finance WebSocket
//!
//! This module provides a Stream-based API for receiving real-time price updates,
//! similar to Kotlin Flow or Rx observables.
//!
//! # Overview
//!
//! Yahoo Finance provides a WebSocket endpoint that streams real-time price data
//! in protobuf format. This module handles:
//!
//! - WebSocket connection and reconnection
//! - Protobuf message decoding
//! - Subscription management with automatic heartbeats
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
mod pricing;

pub use client::{PriceStream, PriceStreamBuilder, StreamError, StreamResult};
pub use pricing::{MarketHoursType, OptionType, PriceUpdate, QuoteType};
