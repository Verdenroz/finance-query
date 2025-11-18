//! # finance-query
//!
//! A Rust library for fetching financial data from Yahoo Finance.
//!
//! This library provides both a high-level client API and a builder pattern for advanced queries.
//!
//! ## Features
//!
//! - Fetch stock quotes (detailed and simple)
//! - Retrieve historical chart data
//! - Search for securities
//! - Automatic authentication handling
//! - Async/await support with Tokio
//!
//! ## Quick Start
//!
//! ```no_run
//! use finance_query::{YahooClient, ClientConfig, endpoints::fetch_quote_summary};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create client with default configuration
//!     let client = YahooClient::new(ClientConfig::default()).await?;
//!
//!     // Fetch quote summary data
//!     let quote = fetch_quote_summary(&client, "AAPL").await?;
//!     println!("{}", serde_json::to_string_pretty(&quote)?);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Advanced Usage
//!
//! The library exposes the underlying HTTP client for custom requests:
//!
//! ```no_run
//! use finance_query::{YahooClient, ClientConfig};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = YahooClient::new(ClientConfig::default()).await?;
//!
//! // Access the underlying HTTP client for custom requests
//! let http = client.http_client().await;
//!
//! // Refresh authentication if needed
//! client.refresh_auth_if_needed().await?;
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

/// Yahoo Finance authentication handling
pub mod auth;
/// HTTP client for Yahoo Finance API
pub mod client;
/// Constants for API endpoints, headers, and configuration
pub mod constants;
/// API endpoint implementations for fetching financial data
pub mod endpoints;
/// Error types and result definitions
pub mod error;

// Re-export main types
pub use client::{ClientConfig, YahooClient};
pub use constants::{Interval, TimeRange};
pub use error::{Result, YahooError};

// Re-export endpoint functions for convenience
pub use endpoints::fetch_quote_summary;
