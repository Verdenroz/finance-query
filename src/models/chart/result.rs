use super::{Candle, Chart, ChartIndicators, ChartMeta};
/// Chart Result module
///
/// Contains the main ChartResult type and conversion methods.
use serde::{Deserialize, Serialize};

/// Chart result for a single symbol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartResult {
    /// Metadata about the chart
    pub meta: ChartMeta,
    /// Timestamps for each data point
    pub timestamp: Option<Vec<i64>>,
    /// Price indicators (OHLCV)
    pub indicators: ChartIndicators,
}

impl ChartResult {
    /// Convert chart result to a vector of candles
    pub fn to_candles(&self) -> Vec<Candle> {
        let timestamps = match &self.timestamp {
            Some(ts) => ts,
            None => return vec![],
        };

        let quote = match self.indicators.quote.first() {
            Some(q) => q,
            None => return vec![],
        };

        let opens = quote.open.as_ref();
        let highs = quote.high.as_ref();
        let lows = quote.low.as_ref();
        let closes = quote.close.as_ref();
        let volumes = quote.volume.as_ref();
        let adj_closes = self
            .indicators
            .adj_close
            .as_ref()
            .and_then(|ac| ac.first())
            .and_then(|ac| ac.adj_close.as_ref());

        timestamps
            .iter()
            .enumerate()
            .filter_map(|(i, &ts)| {
                let open = opens.and_then(|o| o.get(i)).and_then(|v| *v)?;
                let high = highs.and_then(|h| h.get(i)).and_then(|v| *v)?;
                let low = lows.and_then(|l| l.get(i)).and_then(|v| *v)?;
                let close = closes.and_then(|c| c.get(i)).and_then(|v| *v)?;
                let volume = volumes.and_then(|v| v.get(i)).and_then(|v| *v).unwrap_or(0);
                let adj_close = adj_closes.and_then(|ac| ac.get(i)).and_then(|v| *v);

                Some(Candle {
                    timestamp: ts,
                    open,
                    high,
                    low,
                    close,
                    volume,
                    adj_close,
                })
            })
            .collect()
    }

    /// Converts this chart result into a Chart aggregate
    ///
    /// Extracts metadata and candles into a clean, serializable structure.
    pub fn to_chart(&self) -> Chart {
        Chart {
            symbol: self.meta.symbol.clone(),
            meta: self.meta.clone(),
            candles: self.to_candles(),
            interval: self.meta.data_granularity.clone(),
            range: self.meta.range.clone(),
        }
    }
}
