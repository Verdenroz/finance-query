// unrouted: awaiting a capability route; see #264.
#![allow(dead_code)]

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::adapters::fmp::build_client;
use crate::error::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TechnicalIndicator {
    Sma,
    Ema,
    Wma,
    Dema,
    Tema,
    Rsi,
    StandardDeviation,
    Williams,
    Adx,
}

impl TechnicalIndicator {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sma => "sma",
            Self::Ema => "ema",
            Self::Wma => "wma",
            Self::Dema => "dema",
            Self::Tema => "tema",
            Self::Rsi => "rsi",
            Self::StandardDeviation => "standarddeviation",
            Self::Williams => "williams",
            Self::Adx => "adx",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TechnicalIndicatorDTO {
    pub date: Option<String>,
    pub open: Option<f64>,
    pub high: Option<f64>,
    pub low: Option<f64>,
    pub close: Option<f64>,
    pub volume: Option<u64>,
    #[serde(flatten)]
    pub values: HashMap<String, Value>,
}

impl TechnicalIndicatorDTO {
    pub fn indicator_value(&self, indicator: TechnicalIndicator) -> Option<f64> {
        self.values.get(indicator.as_str()).and_then(Value::as_f64)
    }
}

pub async fn technical_indicator(
    indicator: TechnicalIndicator,
    symbol: &str,
    period_length: u32,
    timeframe: &str,
    from: Option<&str>,
    to: Option<&str>,
) -> Result<Vec<TechnicalIndicatorDTO>> {
    let period_length = period_length.to_string();
    let mut params = vec![
        ("symbol", symbol),
        ("periodLength", period_length.as_str()),
        ("timeframe", timeframe),
    ];
    if let Some(from) = from {
        params.push(("from", from));
    }
    if let Some(to) = to {
        params.push(("to", to));
    }
    let path = format!("/stable/technical-indicators/{}", indicator.as_str());
    build_client()?.get(&path, &params).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn technical_payload_exposes_selected_value() {
        let row: TechnicalIndicatorDTO =
            serde_json::from_str(r#"{"date":"2026-08-06","close":200.0,"volume":10,"sma":195.5}"#)
                .unwrap();
        assert_eq!(row.indicator_value(TechnicalIndicator::Sma), Some(195.5));
        assert_eq!(
            TechnicalIndicator::StandardDeviation.as_str(),
            "standarddeviation"
        );
    }
}
