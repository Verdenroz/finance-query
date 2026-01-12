use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::str::FromStr;

/// Alert types supporting various financial metrics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AlertType {
    PriceAbove,
    PriceBelow,
    ChangeAbove,
    ChangeBelow,
    VolumeSpike,
    Week52High,
    Week52Low,
    MarketCapAbove,
    MarketCapBelow,
    DividendYieldAbove,
    PeRatioAbove,
    PeRatioBelow,
}

impl AlertType {
    pub fn display(&self) -> &'static str {
        match self {
            AlertType::PriceAbove => "Price Above",
            AlertType::PriceBelow => "Price Below",
            AlertType::ChangeAbove => "Change % Above",
            AlertType::ChangeBelow => "Change % Below",
            AlertType::VolumeSpike => "Volume Spike",
            AlertType::Week52High => "Near 52W High",
            AlertType::Week52Low => "Near 52W Low",
            AlertType::MarketCapAbove => "Market Cap Above",
            AlertType::MarketCapBelow => "Market Cap Below",
            AlertType::DividendYieldAbove => "Dividend Yield Above",
            AlertType::PeRatioAbove => "P/E Ratio Above",
            AlertType::PeRatioBelow => "P/E Ratio Below",
        }
    }

    pub fn short_display(&self) -> &'static str {
        match self {
            AlertType::PriceAbove => "Price >",
            AlertType::PriceBelow => "Price <",
            AlertType::ChangeAbove => "Chg% >",
            AlertType::ChangeBelow => "Chg% <",
            AlertType::VolumeSpike => "Vol spike",
            AlertType::Week52High => "52W High",
            AlertType::Week52Low => "52W Low",
            AlertType::MarketCapAbove => "MCap >",
            AlertType::MarketCapBelow => "MCap <",
            AlertType::DividendYieldAbove => "DivYld >",
            AlertType::PeRatioAbove => "P/E >",
            AlertType::PeRatioBelow => "P/E <",
        }
    }

    pub fn as_db_str(self) -> &'static str {
        match self {
            AlertType::PriceAbove => "price-above",
            AlertType::PriceBelow => "price-below",
            AlertType::ChangeAbove => "change-above",
            AlertType::ChangeBelow => "change-below",
            AlertType::VolumeSpike => "volume-spike",
            AlertType::Week52High => "52w-high",
            AlertType::Week52Low => "52w-low",
            AlertType::MarketCapAbove => "mcap-above",
            AlertType::MarketCapBelow => "mcap-below",
            AlertType::DividendYieldAbove => "div-yield-above",
            AlertType::PeRatioAbove => "pe-above",
            AlertType::PeRatioBelow => "pe-below",
        }
    }

    /// Returns all available alert types for UI selection
    pub fn all() -> &'static [AlertType] {
        &[
            AlertType::PriceAbove,
            AlertType::PriceBelow,
            AlertType::ChangeAbove,
            AlertType::ChangeBelow,
            AlertType::VolumeSpike,
            AlertType::Week52High,
            AlertType::Week52Low,
            AlertType::MarketCapAbove,
            AlertType::MarketCapBelow,
            AlertType::DividendYieldAbove,
            AlertType::PeRatioAbove,
            AlertType::PeRatioBelow,
        ]
    }

    /// Format the threshold value for display
    pub fn format_threshold(&self, threshold: f64) -> String {
        match self {
            AlertType::PriceAbove | AlertType::PriceBelow => format!("${:.2}", threshold),
            AlertType::ChangeAbove | AlertType::ChangeBelow => format!("{:.1}%", threshold),
            AlertType::VolumeSpike => format!("{}x avg", threshold),
            AlertType::Week52High | AlertType::Week52Low => format!("within {:.1}%", threshold),
            AlertType::MarketCapAbove | AlertType::MarketCapBelow => format!("${:.1}B", threshold),
            AlertType::DividendYieldAbove => format!("{:.2}%", threshold),
            AlertType::PeRatioAbove | AlertType::PeRatioBelow => format!("{:.1}", threshold),
        }
    }

    /// Format a current value for display in notifications
    pub fn format_current_value(&self, value: f64) -> String {
        match self {
            AlertType::PriceAbove | AlertType::PriceBelow => format!("${:.2}", value),
            AlertType::ChangeAbove | AlertType::ChangeBelow => format!("{:+.2}%", value),
            AlertType::VolumeSpike => format!("{:.1}x", value),
            AlertType::Week52High | AlertType::Week52Low => format!("{:.1}% away", value),
            AlertType::MarketCapAbove | AlertType::MarketCapBelow => format!("${:.1}B", value),
            AlertType::DividendYieldAbove => format!("{:.2}%", value),
            AlertType::PeRatioAbove | AlertType::PeRatioBelow => format!("{:.1}", value),
        }
    }
}

impl FromStr for AlertType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Handle both old simple format ("above"/"below") and new format
        match s.to_lowercase().as_str() {
            "above" | "price-above" => Ok(AlertType::PriceAbove),
            "below" | "price-below" => Ok(AlertType::PriceBelow),
            "change-above" => Ok(AlertType::ChangeAbove),
            "change-below" => Ok(AlertType::ChangeBelow),
            "volume-spike" => Ok(AlertType::VolumeSpike),
            "52w-high" => Ok(AlertType::Week52High),
            "52w-low" => Ok(AlertType::Week52Low),
            "mcap-above" => Ok(AlertType::MarketCapAbove),
            "mcap-below" => Ok(AlertType::MarketCapBelow),
            "div-yield-above" => Ok(AlertType::DividendYieldAbove),
            "pe-above" => Ok(AlertType::PeRatioAbove),
            "pe-below" => Ok(AlertType::PeRatioBelow),
            _ => anyhow::bail!("Unknown alert type: {}", s),
        }
    }
}

impl std::fmt::Display for AlertType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_db_str())
    }
}

/// A financial alert with trigger conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: i64,
    pub symbol: String,
    pub alert_type: AlertType,
    pub threshold: f64,
    pub label: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_triggered: Option<DateTime<Utc>>,
    pub trigger_count: u32,
    pub enabled: bool,
}

impl Alert {
    /// Check if this alert is triggered given a quote
    pub fn check(&self, quote: &finance_query::Quote) -> bool {
        if !self.enabled {
            return false;
        }

        match self.alert_type {
            AlertType::PriceAbove => quote
                .regular_market_price
                .as_ref()
                .and_then(|v| v.raw)
                .map(|price| price > self.threshold)
                .unwrap_or(false),
            AlertType::PriceBelow => quote
                .regular_market_price
                .as_ref()
                .and_then(|v| v.raw)
                .map(|price| price < self.threshold)
                .unwrap_or(false),
            AlertType::ChangeAbove => quote
                .regular_market_change_percent
                .as_ref()
                .and_then(|v| v.raw)
                .map(|change| change > self.threshold)
                .unwrap_or(false),
            AlertType::ChangeBelow => quote
                .regular_market_change_percent
                .as_ref()
                .and_then(|v| v.raw)
                .map(|change| change < self.threshold)
                .unwrap_or(false),
            AlertType::VolumeSpike => {
                let volume = quote.regular_market_volume.as_ref().and_then(|v| v.raw);
                let avg_volume = quote.average_volume.as_ref().and_then(|v| v.raw);
                match (volume, avg_volume) {
                    (Some(vol), Some(avg)) if avg > 0 => (vol as f64 / avg as f64) > self.threshold,
                    _ => false,
                }
            }
            AlertType::Week52High => {
                let price = quote.regular_market_price.as_ref().and_then(|v| v.raw);
                let high = quote.fifty_two_week_high.as_ref().and_then(|v| v.raw);
                match (price, high) {
                    (Some(p), Some(h)) if h > 0.0 => {
                        let percent_from_high = ((h - p) / h) * 100.0;
                        percent_from_high <= self.threshold
                    }
                    _ => false,
                }
            }
            AlertType::Week52Low => {
                let price = quote.regular_market_price.as_ref().and_then(|v| v.raw);
                let low = quote.fifty_two_week_low.as_ref().and_then(|v| v.raw);
                match (price, low) {
                    (Some(p), Some(l)) if l > 0.0 => {
                        let percent_from_low = ((p - l) / l) * 100.0;
                        percent_from_low <= self.threshold
                    }
                    _ => false,
                }
            }
            AlertType::MarketCapAbove => quote
                .market_cap
                .as_ref()
                .and_then(|v| v.raw)
                .map(|mcap| (mcap as f64 / 1_000_000_000.0) > self.threshold)
                .unwrap_or(false),
            AlertType::MarketCapBelow => quote
                .market_cap
                .as_ref()
                .and_then(|v| v.raw)
                .map(|mcap| (mcap as f64 / 1_000_000_000.0) < self.threshold)
                .unwrap_or(false),
            AlertType::DividendYieldAbove => quote
                .dividend_yield
                .as_ref()
                .and_then(|v| v.raw)
                .map(|dy| (dy * 100.0) > self.threshold)
                .unwrap_or(false),
            AlertType::PeRatioAbove => quote
                .trailing_pe
                .as_ref()
                .and_then(|v| v.raw)
                .map(|pe| pe > self.threshold)
                .unwrap_or(false),
            AlertType::PeRatioBelow => quote
                .trailing_pe
                .as_ref()
                .and_then(|v| v.raw)
                .map(|pe| pe < self.threshold)
                .unwrap_or(false),
        }
    }

    /// Get the current value from a quote for this alert type
    pub fn get_current_value(&self, quote: &finance_query::Quote) -> Option<f64> {
        match self.alert_type {
            AlertType::PriceAbove | AlertType::PriceBelow => {
                quote.regular_market_price.as_ref().and_then(|v| v.raw)
            }
            AlertType::ChangeAbove | AlertType::ChangeBelow => quote
                .regular_market_change_percent
                .as_ref()
                .and_then(|v| v.raw),
            AlertType::VolumeSpike => {
                let volume = quote.regular_market_volume.as_ref().and_then(|v| v.raw)?;
                let avg_volume = quote.average_volume.as_ref().and_then(|v| v.raw)?;
                if avg_volume > 0 {
                    Some(volume as f64 / avg_volume as f64)
                } else {
                    None
                }
            }
            AlertType::Week52High => {
                let price = quote.regular_market_price.as_ref().and_then(|v| v.raw)?;
                let high = quote.fifty_two_week_high.as_ref().and_then(|v| v.raw)?;
                if high > 0.0 {
                    Some(((high - price) / high) * 100.0)
                } else {
                    None
                }
            }
            AlertType::Week52Low => {
                let price = quote.regular_market_price.as_ref().and_then(|v| v.raw)?;
                let low = quote.fifty_two_week_low.as_ref().and_then(|v| v.raw)?;
                if low > 0.0 {
                    Some(((price - low) / low) * 100.0)
                } else {
                    None
                }
            }
            AlertType::MarketCapAbove | AlertType::MarketCapBelow => quote
                .market_cap
                .as_ref()
                .and_then(|v| v.raw)
                .map(|mcap| mcap as f64 / 1_000_000_000.0),
            AlertType::DividendYieldAbove => quote
                .dividend_yield
                .as_ref()
                .and_then(|v| v.raw)
                .map(|dy| dy * 100.0),
            AlertType::PeRatioAbove | AlertType::PeRatioBelow => {
                quote.trailing_pe.as_ref().and_then(|v| v.raw)
            }
        }
    }

    /// Format the current value for display
    pub fn format_current_value(&self, quote: &finance_query::Quote) -> String {
        match self.get_current_value(quote) {
            Some(val) => match self.alert_type {
                AlertType::PriceAbove | AlertType::PriceBelow => format!("${:.2}", val),
                AlertType::ChangeAbove | AlertType::ChangeBelow => {
                    if val >= 0.0 {
                        format!("+{:.2}%", val)
                    } else {
                        format!("{:.2}%", val)
                    }
                }
                AlertType::VolumeSpike => format!("{:.2}x", val),
                AlertType::Week52High => format!("{:.1}% from high", val),
                AlertType::Week52Low => format!("{:.1}% from low", val),
                AlertType::MarketCapAbove | AlertType::MarketCapBelow => format!("${:.1}B", val),
                AlertType::DividendYieldAbove => format!("{:.2}%", val),
                AlertType::PeRatioAbove | AlertType::PeRatioBelow => format!("{:.2}", val),
            },
            None => "N/A".to_string(),
        }
    }
}

/// Unified alert storage using SQLite
pub struct AlertStore {
    conn: Connection,
}

impl AlertStore {
    pub fn new() -> Result<Self> {
        let path = Self::get_db_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&path).context("Failed to open alerts database")?;
        let store = Self { conn };
        store.init_schema()?;
        Ok(store)
    }

    fn get_db_path() -> Result<PathBuf> {
        let data_dir = dirs::data_local_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine local data directory"))?;
        Ok(data_dir.join("fq").join("alerts.db"))
    }

    fn init_schema(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS alerts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                symbol TEXT NOT NULL,
                alert_type TEXT NOT NULL,
                threshold REAL NOT NULL,
                label TEXT,
                created_at TEXT NOT NULL,
                last_triggered TEXT,
                trigger_count INTEGER NOT NULL DEFAULT 0,
                enabled INTEGER NOT NULL DEFAULT 1
            )",
            [],
        )?;

        // Create index for faster lookups by symbol
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_alerts_symbol ON alerts(symbol)",
            [],
        )?;

        Ok(())
    }

    /// Get all alerts
    pub fn get_alerts(&self) -> Result<Vec<Alert>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, symbol, alert_type, threshold, label, created_at, last_triggered, trigger_count, enabled
             FROM alerts ORDER BY created_at DESC",
        )?;

        let alerts = stmt
            .query_map([], |row| {
                let alert_type_str: String = row.get(2)?;
                let created_at_str: String = row.get(5)?;
                let last_triggered_str: Option<String> = row.get(6)?;

                Ok(Alert {
                    id: row.get(0)?,
                    symbol: row.get(1)?,
                    alert_type: alert_type_str.parse().unwrap_or(AlertType::PriceAbove),
                    threshold: row.get(3)?,
                    label: row.get(4)?,
                    created_at: DateTime::parse_from_rfc3339(&created_at_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    last_triggered: last_triggered_str.and_then(|s| {
                        DateTime::parse_from_rfc3339(&s)
                            .map(|dt| dt.with_timezone(&Utc))
                            .ok()
                    }),
                    trigger_count: row.get(7)?,
                    enabled: row.get::<_, i32>(8)? == 1,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(alerts)
    }

    /// Get alerts for a specific symbol
    pub fn get_alerts_for_symbol(&self, symbol: &str) -> Result<Vec<Alert>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, symbol, alert_type, threshold, label, created_at, last_triggered, trigger_count, enabled
             FROM alerts WHERE UPPER(symbol) = UPPER(?) ORDER BY created_at DESC",
        )?;

        let alerts = stmt
            .query_map(params![symbol], |row| {
                let alert_type_str: String = row.get(2)?;
                let created_at_str: String = row.get(5)?;
                let last_triggered_str: Option<String> = row.get(6)?;

                Ok(Alert {
                    id: row.get(0)?,
                    symbol: row.get(1)?,
                    alert_type: alert_type_str.parse().unwrap_or(AlertType::PriceAbove),
                    threshold: row.get(3)?,
                    label: row.get(4)?,
                    created_at: DateTime::parse_from_rfc3339(&created_at_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    last_triggered: last_triggered_str.and_then(|s| {
                        DateTime::parse_from_rfc3339(&s)
                            .map(|dt| dt.with_timezone(&Utc))
                            .ok()
                    }),
                    trigger_count: row.get(7)?,
                    enabled: row.get::<_, i32>(8)? == 1,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(alerts)
    }

    /// Get enabled alerts only (for checking)
    pub fn get_enabled_alerts(&self) -> Result<Vec<Alert>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, symbol, alert_type, threshold, label, created_at, last_triggered, trigger_count, enabled
             FROM alerts WHERE enabled = 1 ORDER BY created_at DESC",
        )?;

        let alerts = stmt
            .query_map([], |row| {
                let alert_type_str: String = row.get(2)?;
                let created_at_str: String = row.get(5)?;
                let last_triggered_str: Option<String> = row.get(6)?;

                Ok(Alert {
                    id: row.get(0)?,
                    symbol: row.get(1)?,
                    alert_type: alert_type_str.parse().unwrap_or(AlertType::PriceAbove),
                    threshold: row.get(3)?,
                    label: row.get(4)?,
                    created_at: DateTime::parse_from_rfc3339(&created_at_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    last_triggered: last_triggered_str.and_then(|s| {
                        DateTime::parse_from_rfc3339(&s)
                            .map(|dt| dt.with_timezone(&Utc))
                            .ok()
                    }),
                    trigger_count: row.get(7)?,
                    enabled: row.get::<_, i32>(8)? == 1,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(alerts)
    }

    /// Create a new alert
    pub fn create_alert(
        &self,
        symbol: &str,
        alert_type: AlertType,
        threshold: f64,
        label: Option<&str>,
    ) -> Result<i64> {
        let now = Utc::now().to_rfc3339();

        self.conn.execute(
            "INSERT INTO alerts (symbol, alert_type, threshold, label, created_at, trigger_count, enabled)
             VALUES (?, ?, ?, ?, ?, 0, 1)",
            params![
                symbol.to_uppercase(),
                alert_type.as_db_str(),
                threshold,
                label,
                now
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Delete an alert by ID
    pub fn delete_alert(&self, id: i64) -> Result<bool> {
        let rows = self
            .conn
            .execute("DELETE FROM alerts WHERE id = ?", params![id])?;
        Ok(rows > 0)
    }

    /// Update alert triggered status
    pub fn mark_triggered(&self, id: i64) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE alerts SET last_triggered = ?, trigger_count = trigger_count + 1 WHERE id = ?",
            params![now, id],
        )?;
        Ok(())
    }

    /// Enable or disable an alert
    pub fn set_enabled(&self, id: i64, enabled: bool) -> Result<()> {
        self.conn.execute(
            "UPDATE alerts SET enabled = ? WHERE id = ?",
            params![if enabled { 1 } else { 0 }, id],
        )?;
        Ok(())
    }

    /// Clear all alerts
    pub fn clear_all(&self) -> Result<usize> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM alerts", [], |row| row.get(0))?;

        self.conn.execute("DELETE FROM alerts", [])?;

        Ok(count as usize)
    }
}
