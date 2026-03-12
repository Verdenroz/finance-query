//! Formatted value support for GraphQL responses.
//!
//! Yahoo Finance returns many numeric fields as `{"raw": 182.5, "fmt": "182.50", "longFmt": "182.50"}`
//! objects. Resolvers apply `finance_query::ValueFormat::transform()` on the raw JSON before
//! deserializing into GraphQL types, so formatted fields are exposed as `Json<serde_json::Value>`
//! and will contain a scalar (raw mode), string (fmt mode), or the full object (both mode).
