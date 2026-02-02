//! Prometheus metrics for monitoring server performance.
//!
//! Tracks request counts, latencies, cache performance, and WebSocket connections.

use lazy_static::lazy_static;
use prometheus::{
    Counter, CounterVec, Gauge, HistogramOpts, HistogramVec, Opts, Registry, TextEncoder,
};
use std::sync::Once;
use std::time::Instant;

static INIT: Once = Once::new();

lazy_static! {
    /// Global metrics registry
    pub static ref REGISTRY: Registry = Registry::new();

    // === HTTP Request Metrics ===

    /// Total HTTP requests by endpoint and status code
    pub static ref HTTP_REQUESTS_TOTAL: CounterVec = CounterVec::new(
        Opts::new("http_requests_total", "Total number of HTTP requests")
            .namespace("finance_query"),
        &["method", "endpoint", "status"]
    )
    .expect("Failed to create HTTP requests counter");

    /// HTTP request duration in seconds
    pub static ref HTTP_REQUEST_DURATION: HistogramVec = HistogramVec::new(
        HistogramOpts::new(
            "http_request_duration_seconds",
            "HTTP request latency in seconds"
        )
        .namespace("finance_query")
        .buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]),
        &["method", "endpoint"]
    )
    .expect("Failed to create HTTP request duration histogram");

    /// Active HTTP requests currently being processed
    pub static ref HTTP_REQUESTS_IN_FLIGHT: Gauge = Gauge::new(
        "http_requests_in_flight",
        "Number of HTTP requests currently being processed"
    )
    .expect("Failed to create in-flight requests gauge");

    // === Cache Metrics ===

    /// Cache hits
    pub static ref CACHE_HITS: Counter = Counter::new(
        "cache_hits_total",
        "Total number of cache hits"
    )
    .expect("Failed to create cache hits counter");

    /// Cache misses
    pub static ref CACHE_MISSES: Counter = Counter::new(
        "cache_misses_total",
        "Total number of cache misses"
    )
    .expect("Failed to create cache misses counter");

    /// Cache operation duration
    pub static ref CACHE_OPERATION_DURATION: HistogramVec = HistogramVec::new(
        HistogramOpts::new(
            "cache_operation_duration_seconds",
            "Cache operation duration in seconds"
        )
        .namespace("finance_query")
        .buckets(vec![0.0001, 0.0005, 0.001, 0.005, 0.01, 0.05, 0.1]),
        &["operation"]
    )
    .expect("Failed to create cache operation duration histogram");

    // === WebSocket Metrics ===

    /// Active WebSocket connections
    pub static ref WEBSOCKET_CONNECTIONS: Gauge = Gauge::new(
        "websocket_connections_active",
        "Number of active WebSocket connections"
    )
    .expect("Failed to create WebSocket connections gauge");

    /// Total WebSocket messages sent
    pub static ref WEBSOCKET_MESSAGES_SENT: Counter = Counter::new(
        "websocket_messages_sent_total",
        "Total number of WebSocket messages sent"
    )
    .expect("Failed to create WebSocket messages sent counter");

    /// Total WebSocket messages received
    pub static ref WEBSOCKET_MESSAGES_RECEIVED: Counter = Counter::new(
        "websocket_messages_received_total",
        "Total number of WebSocket messages received"
    )
    .expect("Failed to create WebSocket messages received counter");

    /// WebSocket symbol subscriptions
    pub static ref WEBSOCKET_SYMBOLS_SUBSCRIBED: Gauge = Gauge::new(
        "websocket_symbols_subscribed",
        "Number of unique symbols currently subscribed to"
    )
    .expect("Failed to create WebSocket symbols gauge");

    // === Error Metrics ===

    /// Total errors by type
    pub static ref ERRORS_TOTAL: CounterVec = CounterVec::new(
        Opts::new("errors_total", "Total number of errors by type")
            .namespace("finance_query"),
        &["error_type"]
    )
    .expect("Failed to create errors counter");

    // === Rate Limiting Metrics ===

    /// Total rate limit rejections
    pub static ref RATE_LIMIT_REJECTIONS: Counter = Counter::new(
        "rate_limit_rejections_total",
        "Total number of requests rejected due to rate limiting"
    )
    .expect("Failed to create rate limit rejections counter");
}

/// Initialize metrics registry
pub fn init() {
    INIT.call_once(|| {
        REGISTRY
            .register(Box::new(HTTP_REQUESTS_TOTAL.clone()))
            .expect("Failed to register HTTP_REQUESTS_TOTAL");
        REGISTRY
            .register(Box::new(HTTP_REQUEST_DURATION.clone()))
            .expect("Failed to register HTTP_REQUEST_DURATION");
        REGISTRY
            .register(Box::new(HTTP_REQUESTS_IN_FLIGHT.clone()))
            .expect("Failed to register HTTP_REQUESTS_IN_FLIGHT");
        REGISTRY
            .register(Box::new(CACHE_HITS.clone()))
            .expect("Failed to register CACHE_HITS");
        REGISTRY
            .register(Box::new(CACHE_MISSES.clone()))
            .expect("Failed to register CACHE_MISSES");
        REGISTRY
            .register(Box::new(CACHE_OPERATION_DURATION.clone()))
            .expect("Failed to register CACHE_OPERATION_DURATION");
        REGISTRY
            .register(Box::new(WEBSOCKET_CONNECTIONS.clone()))
            .expect("Failed to register WEBSOCKET_CONNECTIONS");
        REGISTRY
            .register(Box::new(WEBSOCKET_MESSAGES_SENT.clone()))
            .expect("Failed to register WEBSOCKET_MESSAGES_SENT");
        REGISTRY
            .register(Box::new(WEBSOCKET_MESSAGES_RECEIVED.clone()))
            .expect("Failed to register WEBSOCKET_MESSAGES_RECEIVED");
        REGISTRY
            .register(Box::new(WEBSOCKET_SYMBOLS_SUBSCRIBED.clone()))
            .expect("Failed to register WEBSOCKET_SYMBOLS_SUBSCRIBED");
        REGISTRY
            .register(Box::new(ERRORS_TOTAL.clone()))
            .expect("Failed to register ERRORS_TOTAL");
        REGISTRY
            .register(Box::new(RATE_LIMIT_REJECTIONS.clone()))
            .expect("Failed to register RATE_LIMIT_REJECTIONS");

        tracing::info!("Prometheus metrics initialized");
    });
}

/// Gather and encode metrics in Prometheus text format
pub fn gather() -> String {
    use prometheus::Encoder;

    let encoder = TextEncoder::new();
    let metric_families = REGISTRY.gather();
    let mut buffer = Vec::new();

    encoder
        .encode(&metric_families, &mut buffer)
        .expect("Failed to encode metrics");

    String::from_utf8(buffer).expect("Failed to convert metrics to string")
}

/// Helper to track request timing
pub struct RequestTimer {
    start: Instant,
    method: String,
    endpoint: String,
}

impl RequestTimer {
    pub fn new(method: impl Into<String>, endpoint: impl Into<String>) -> Self {
        HTTP_REQUESTS_IN_FLIGHT.inc();
        Self {
            start: Instant::now(),
            method: method.into(),
            endpoint: endpoint.into(),
        }
    }

    pub fn observe(self, status: u16) {
        let duration = self.start.elapsed().as_secs_f64();

        HTTP_REQUEST_DURATION
            .with_label_values(&[&self.method, &self.endpoint])
            .observe(duration);

        HTTP_REQUESTS_TOTAL
            .with_label_values(&[&self.method, &self.endpoint, &status.to_string()])
            .inc();

        HTTP_REQUESTS_IN_FLIGHT.dec();
    }
}

/// Helper to track cache timing
pub struct CacheTimer {
    start: Instant,
    operation: String,
}

impl CacheTimer {
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            start: Instant::now(),
            operation: operation.into(),
        }
    }

    pub fn observe(self) {
        let duration = self.start.elapsed().as_secs_f64();
        CACHE_OPERATION_DURATION
            .with_label_values(&[&self.operation])
            .observe(duration);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_initialization() {
        // Test that metrics can be initialized without panicking
        init();

        // Test that we can increment counters
        CACHE_HITS.inc();
        CACHE_MISSES.inc();

        // Test that we can observe histograms
        HTTP_REQUEST_DURATION
            .with_label_values(&["GET", "/v2/health"])
            .observe(0.001);
    }

    #[test]
    fn test_metrics_gathering() {
        init();

        // Generate some metrics data
        HTTP_REQUESTS_TOTAL
            .with_label_values(&["GET", "/v2/health", "200"])
            .inc();
        CACHE_HITS.inc();

        let output = gather();

        // Should contain metric names (with namespace prefix)
        assert!(output.contains("finance_query_http_requests_total"));
        assert!(output.contains("cache_hits_total"));
    }

    #[test]
    fn test_request_timer() {
        init();
        let timer = RequestTimer::new("GET", "/v2/test");
        timer.observe(200);

        // Verify in-flight gauge returned to baseline
        assert_eq!(HTTP_REQUESTS_IN_FLIGHT.get() as i64, 0);
    }
}
