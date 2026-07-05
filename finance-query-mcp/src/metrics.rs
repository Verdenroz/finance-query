//! Prometheus metrics for monitoring MCP tool invocations.
//!
//! Tracks per-tool call counts, latencies, and in-flight calls. Exported via
//! `/metrics` on the HTTP transport; stdio runs accumulate but never export.

use lazy_static::lazy_static;
use prometheus::{CounterVec, Gauge, HistogramOpts, HistogramVec, Opts, Registry, TextEncoder};
use std::sync::Once;
use std::time::Instant;

static INIT: Once = Once::new();

lazy_static! {
    /// Global metrics registry
    pub static ref REGISTRY: Registry = Registry::new();

    /// Total tool calls by tool name and outcome (status: "ok" | "error")
    pub static ref TOOL_CALLS_TOTAL: CounterVec = CounterVec::new(
        Opts::new("tool_calls_total", "Total number of MCP tool calls")
            .namespace("finance_query_mcp"),
        &["tool", "status"]
    )
    .expect("Failed to create tool calls counter");

    /// Tool call duration in seconds by tool name
    pub static ref TOOL_CALL_DURATION: HistogramVec = HistogramVec::new(
        HistogramOpts::new(
            "tool_call_duration_seconds",
            "MCP tool call latency in seconds"
        )
        .namespace("finance_query_mcp")
        .buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]),
        &["tool"]
    )
    .expect("Failed to create tool call duration histogram");

    /// Tool calls currently being processed
    pub static ref TOOL_CALLS_IN_FLIGHT: Gauge = Gauge::with_opts(
        Opts::new(
            "tool_calls_in_flight",
            "Number of MCP tool calls currently being processed"
        )
        .namespace("finance_query_mcp")
    )
    .expect("Failed to create in-flight tool calls gauge");
}

/// Initialize metrics registry
pub fn init() {
    INIT.call_once(|| {
        REGISTRY
            .register(Box::new(TOOL_CALLS_TOTAL.clone()))
            .expect("Failed to register TOOL_CALLS_TOTAL");
        REGISTRY
            .register(Box::new(TOOL_CALL_DURATION.clone()))
            .expect("Failed to register TOOL_CALL_DURATION");
        REGISTRY
            .register(Box::new(TOOL_CALLS_IN_FLIGHT.clone()))
            .expect("Failed to register TOOL_CALLS_IN_FLIGHT");

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

/// Helper to track tool call timing
pub struct ToolCallTimer {
    start: Instant,
    tool: String,
}

impl ToolCallTimer {
    pub fn new(tool: impl Into<String>) -> Self {
        TOOL_CALLS_IN_FLIGHT.inc();
        Self {
            start: Instant::now(),
            tool: tool.into(),
        }
    }

    pub fn observe(self, ok: bool) {
        let duration = self.start.elapsed().as_secs_f64();

        TOOL_CALL_DURATION
            .with_label_values(&[&self.tool])
            .observe(duration);

        let status = if ok { "ok" } else { "error" };
        TOOL_CALLS_TOTAL
            .with_label_values(&[self.tool.as_str(), status])
            .inc();

        TOOL_CALLS_IN_FLIGHT.dec();
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
        TOOL_CALLS_TOTAL
            .with_label_values(&["get_quote", "ok"])
            .inc();

        // Test that we can observe histograms
        TOOL_CALL_DURATION
            .with_label_values(&["get_quote"])
            .observe(0.001);
    }

    #[test]
    fn test_metrics_gathering() {
        init();

        // Generate some metrics data
        TOOL_CALLS_TOTAL
            .with_label_values(&["get_chart", "error"])
            .inc();

        let output = gather();

        // Should contain metric names (with namespace prefix)
        assert!(output.contains("finance_query_mcp_tool_calls_total"));
        assert!(output.contains("finance_query_mcp_tool_call_duration_seconds"));
    }

    #[test]
    fn test_tool_call_timer() {
        init();
        let timer = ToolCallTimer::new("get_news");
        timer.observe(true);

        // Verify in-flight gauge returned to baseline
        assert_eq!(TOOL_CALLS_IN_FLIGHT.get() as i64, 0);
    }
}
