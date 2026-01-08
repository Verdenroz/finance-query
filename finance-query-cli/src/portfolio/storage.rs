use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// A single position in the portfolio
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    /// Stock symbol
    pub symbol: String,
    /// Number of shares owned
    pub shares: f64,
    /// Cost basis per share (average purchase price)
    pub cost_basis: f64,
    /// Purchase date (ISO 8601 format)
    pub purchase_date: String,
    /// Optional notes
    #[serde(default)]
    pub notes: String,
}

impl Position {
    pub fn new(symbol: String, shares: f64, cost_basis: f64) -> Self {
        Self {
            symbol,
            shares,
            cost_basis,
            purchase_date: chrono::Local::now().format("%Y-%m-%d").to_string(),
            notes: String::new(),
        }
    }

    /// Total cost of this position
    pub fn total_cost(&self) -> f64 {
        self.shares * self.cost_basis
    }

    /// Current value given current price
    pub fn current_value(&self, current_price: f64) -> f64 {
        self.shares * current_price
    }

    /// Profit/loss given current price
    pub fn profit_loss(&self, current_price: f64) -> f64 {
        self.current_value(current_price) - self.total_cost()
    }

    /// Profit/loss percentage
    pub fn profit_loss_percent(&self, current_price: f64) -> f64 {
        (self.profit_loss(current_price) / self.total_cost()) * 100.0
    }
}

/// Portfolio containing multiple positions
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Portfolio {
    pub positions: Vec<Position>,
}

impl Portfolio {
    pub fn new() -> Self {
        Self {
            positions: Vec::new(),
        }
    }

    /// Add a new position
    pub fn add_position(&mut self, position: Position) {
        self.positions.push(position);
    }

    /// Remove a position by symbol
    pub fn remove_position(&mut self, symbol: &str) -> Option<Position> {
        if let Some(index) = self.positions.iter().position(|p| p.symbol == symbol) {
            Some(self.positions.remove(index))
        } else {
            None
        }
    }

    /// Get all symbols in the portfolio
    pub fn symbols(&self) -> Vec<String> {
        self.positions.iter().map(|p| p.symbol.clone()).collect()
    }

    /// Total cost basis of the portfolio
    pub fn total_cost(&self) -> f64 {
        self.positions.iter().map(|p| p.total_cost()).sum()
    }

    /// Check if portfolio is empty
    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }
}

/// Portfolio storage manager
pub struct PortfolioStorage {
    file_path: PathBuf,
}

impl PortfolioStorage {
    /// Create a new portfolio storage with default path
    pub fn new() -> Result<Self> {
        let file_path = Self::default_path()?;
        Ok(Self { file_path })
    }

    /// Get the default portfolio file path (~/.config/fq/portfolio.json)
    fn default_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Failed to get config directory")?
            .join("fq");

        // Create config directory if it doesn't exist
        fs::create_dir_all(&config_dir)?;

        Ok(config_dir.join("portfolio.json"))
    }

    /// Load portfolio from disk
    pub fn load(&self) -> Result<Portfolio> {
        if !self.file_path.exists() {
            return Ok(Portfolio::new());
        }

        let content =
            fs::read_to_string(&self.file_path).context("Failed to read portfolio file")?;

        let portfolio: Portfolio =
            serde_json::from_str(&content).context("Failed to parse portfolio file")?;

        Ok(portfolio)
    }

    /// Save portfolio to disk
    pub fn save(&self, portfolio: &Portfolio) -> Result<()> {
        let content =
            serde_json::to_string_pretty(portfolio).context("Failed to serialize portfolio")?;

        fs::write(&self.file_path, content).context("Failed to write portfolio file")?;

        Ok(())
    }
}

impl Default for PortfolioStorage {
    fn default() -> Self {
        Self::new().expect("Failed to create portfolio storage")
    }
}
