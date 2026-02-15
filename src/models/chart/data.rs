/// Chart aggregate module
///
/// Contains the fully typed Chart structure for historical data.
use super::{Candle, ChartMeta};
use crate::constants::{Interval, TimeRange};
use serde::{Deserialize, Serialize};

/// Fully typed chart data
///
/// Aggregates chart metadata and candles into a single convenient structure.
/// This is the recommended type for serialization and API responses.
/// Used for both single symbol and batch historical data requests.
///
/// Note: This struct cannot be manually constructed - use `Ticker::chart()` to obtain chart data.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chart {
    /// Stock symbol
    pub symbol: String,

    /// Chart metadata (exchange, currency, 52-week range, etc.)
    pub meta: ChartMeta,

    /// OHLCV candles/bars
    pub candles: Vec<Candle>,

    /// Time interval used (e.g., `Interval::OneDay`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<Interval>,

    /// Time range used (e.g., `TimeRange::OneYear`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<TimeRange>,
}

#[cfg(feature = "dataframe")]
impl Chart {
    /// Converts the candles to a polars DataFrame.
    ///
    /// Each candle becomes a row with columns for timestamp, open, high, low, close, volume.
    pub fn to_dataframe(&self) -> ::polars::prelude::PolarsResult<::polars::prelude::DataFrame> {
        Candle::vec_to_dataframe(&self.candles)
    }
}

#[cfg(feature = "indicators")]
impl Chart {
    /// Extracts close prices from candles as a `Vec<f64>`.
    ///
    /// This is a convenience method for passing price data to indicator functions.
    pub fn close_prices(&self) -> Vec<f64> {
        self.candles.iter().map(|c| c.close).collect()
    }

    /// Extracts high prices from candles as a `Vec<f64>`.
    pub fn high_prices(&self) -> Vec<f64> {
        self.candles.iter().map(|c| c.high).collect()
    }

    /// Extracts low prices from candles as a `Vec<f64>`.
    pub fn low_prices(&self) -> Vec<f64> {
        self.candles.iter().map(|c| c.low).collect()
    }

    /// Extracts open prices from candles as a `Vec<f64>`.
    pub fn open_prices(&self) -> Vec<f64> {
        self.candles.iter().map(|c| c.open).collect()
    }

    /// Extracts volumes from candles as a `Vec<f64>`.
    pub fn volumes(&self) -> Vec<f64> {
        self.candles.iter().map(|c| c.volume as f64).collect()
    }

    /// Calculate Simple Moving Average (SMA) on close prices.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::{Ticker, Interval, TimeRange};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let ticker = Ticker::new("AAPL").await?;
    /// let chart = ticker.chart(Interval::OneDay, TimeRange::OneMonth).await?;
    ///
    /// let sma_20 = chart.sma(20);
    /// # Ok(())
    /// # }
    /// ```
    pub fn sma(&self, period: usize) -> Vec<Option<f64>> {
        crate::indicators::sma(&self.close_prices(), period)
    }

    /// Calculate Exponential Moving Average (EMA) on close prices.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::{Ticker, Interval, TimeRange};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let ticker = Ticker::new("AAPL").await?;
    /// let chart = ticker.chart(Interval::OneDay, TimeRange::OneMonth).await?;
    ///
    /// let ema_12 = chart.ema(12);
    /// # Ok(())
    /// # }
    /// ```
    pub fn ema(&self, period: usize) -> Vec<Option<f64>> {
        crate::indicators::ema(&self.close_prices(), period)
    }

    /// Calculate Relative Strength Index (RSI) on close prices.
    ///
    /// Returns values between 0 and 100. Readings above 70 indicate overbought,
    /// below 30 indicate oversold conditions.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::{Ticker, Interval, TimeRange};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let ticker = Ticker::new("AAPL").await?;
    /// let chart = ticker.chart(Interval::OneDay, TimeRange::OneMonth).await?;
    ///
    /// let rsi = chart.rsi(14)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn rsi(&self, period: usize) -> crate::indicators::Result<Vec<Option<f64>>> {
        crate::indicators::rsi(&self.close_prices(), period)
    }

    /// Calculate Moving Average Convergence Divergence (MACD).
    ///
    /// Standard parameters are (12, 26, 9).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::{Ticker, Interval, TimeRange};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let ticker = Ticker::new("AAPL").await?;
    /// let chart = ticker.chart(Interval::OneDay, TimeRange::OneMonth).await?;
    ///
    /// let macd_result = chart.macd(12, 26, 9)?;
    /// println!("MACD Line: {:?}", macd_result.macd_line);
    /// println!("Signal Line: {:?}", macd_result.signal_line);
    /// println!("Histogram: {:?}", macd_result.histogram);
    /// # Ok(())
    /// # }
    /// ```
    pub fn macd(
        &self,
        fast_period: usize,
        slow_period: usize,
        signal_period: usize,
    ) -> crate::indicators::Result<crate::indicators::MacdResult> {
        crate::indicators::macd(
            &self.close_prices(),
            fast_period,
            slow_period,
            signal_period,
        )
    }

    /// Calculate Bollinger Bands.
    ///
    /// Standard parameters are (20, 2.0) for period and std_dev_multiplier.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::{Ticker, Interval, TimeRange};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let ticker = Ticker::new("AAPL").await?;
    /// let chart = ticker.chart(Interval::OneDay, TimeRange::OneMonth).await?;
    ///
    /// let bb = chart.bollinger_bands(20, 2.0)?;
    /// println!("Upper: {:?}", bb.upper);
    /// println!("Middle: {:?}", bb.middle);
    /// println!("Lower: {:?}", bb.lower);
    /// # Ok(())
    /// # }
    /// ```
    pub fn bollinger_bands(
        &self,
        period: usize,
        std_dev_multiplier: f64,
    ) -> crate::indicators::Result<crate::indicators::BollingerBands> {
        crate::indicators::bollinger_bands(&self.close_prices(), period, std_dev_multiplier)
    }

    /// Calculate Average True Range (ATR).
    ///
    /// ATR measures market volatility. Standard period is 14.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::{Ticker, Interval, TimeRange};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let ticker = Ticker::new("AAPL").await?;
    /// let chart = ticker.chart(Interval::OneDay, TimeRange::OneMonth).await?;
    ///
    /// let atr = chart.atr(14)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn atr(&self, period: usize) -> crate::indicators::Result<Vec<Option<f64>>> {
        crate::indicators::atr(
            &self.high_prices(),
            &self.low_prices(),
            &self.close_prices(),
            period,
        )
    }
}
