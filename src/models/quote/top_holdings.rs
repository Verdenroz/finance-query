use serde::{Deserialize, Serialize};

use super::FormattedValue;

/// Fund holdings including asset allocation, top holdings, and sector weightings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopHoldings {
    /// Maximum age of the data in seconds
    #[serde(default)]
    pub max_age: Option<i64>,

    /// Cash position as a percentage of portfolio
    #[serde(default)]
    pub cash_position: Option<FormattedValue<f64>>,

    /// Stock position as a percentage of portfolio
    #[serde(default)]
    pub stock_position: Option<FormattedValue<f64>>,

    /// Bond position as a percentage of portfolio
    #[serde(default)]
    pub bond_position: Option<FormattedValue<f64>>,

    /// Other assets position as a percentage of portfolio
    #[serde(default)]
    pub other_position: Option<FormattedValue<f64>>,

    /// Preferred stock position as a percentage of portfolio
    #[serde(default)]
    pub preferred_position: Option<FormattedValue<f64>>,

    /// Convertible securities position as a percentage of portfolio
    #[serde(default)]
    pub convertible_position: Option<FormattedValue<f64>>,

    /// Top holdings in the fund
    #[serde(default)]
    pub holdings: Option<Vec<Holding>>,

    /// Equity holdings valuation metrics
    #[serde(default)]
    pub equity_holdings: Option<EquityHoldings>,

    /// Bond holdings metrics
    #[serde(default)]
    pub bond_holdings: Option<serde_json::Value>,

    /// Bond credit ratings distribution
    #[serde(default)]
    pub bond_ratings: Option<Vec<BondRating>>,

    /// Sector weightings distribution
    #[serde(default)]
    pub sector_weightings: Option<Vec<SectorWeighting>>,
}

/// Individual holding in the fund
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Holding {
    /// Stock symbol
    #[serde(default)]
    pub symbol: Option<String>,

    /// Holding name/description
    #[serde(default)]
    pub holding_name: Option<String>,

    /// Percentage of portfolio in this holding
    #[serde(default)]
    pub holding_percent: Option<FormattedValue<f64>>,
}

/// Equity holdings valuation metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EquityHoldings {
    /// Price to earnings ratio
    #[serde(default)]
    pub price_to_earnings: Option<FormattedValue<f64>>,

    /// Price to book ratio
    #[serde(default)]
    pub price_to_book: Option<FormattedValue<f64>>,

    /// Price to sales ratio
    #[serde(default)]
    pub price_to_sales: Option<FormattedValue<f64>>,

    /// Price to cash flow ratio
    #[serde(default)]
    pub price_to_cashflow: Option<FormattedValue<f64>>,
}

/// Bond rating distribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BondRating {
    /// US Government bonds percentage
    #[serde(default)]
    pub us_government: Option<FormattedValue<f64>>,

    /// AAA rated bonds percentage
    #[serde(default)]
    pub aaa: Option<FormattedValue<f64>>,

    /// AA rated bonds percentage
    #[serde(default)]
    pub aa: Option<FormattedValue<f64>>,

    /// A rated bonds percentage
    #[serde(default)]
    pub a: Option<FormattedValue<f64>>,

    /// BBB rated bonds percentage
    #[serde(default)]
    pub bbb: Option<FormattedValue<f64>>,

    /// BB rated bonds percentage
    #[serde(default)]
    pub bb: Option<FormattedValue<f64>>,

    /// B rated bonds percentage
    #[serde(default)]
    pub b: Option<FormattedValue<f64>>,

    /// Below B rated bonds percentage
    #[serde(default)]
    pub below_b: Option<FormattedValue<f64>>,

    /// Not rated bonds percentage
    #[serde(default)]
    pub not_rated: Option<FormattedValue<f64>>,

    /// Other bonds percentage
    #[serde(default)]
    pub other: Option<FormattedValue<f64>>,
}

/// Sector weighting distribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectorWeighting {
    /// Real estate sector percentage
    #[serde(default)]
    pub realestate: Option<FormattedValue<f64>>,

    /// Consumer cyclical sector percentage
    #[serde(default)]
    pub consumer_cyclical: Option<FormattedValue<f64>>,

    /// Basic materials sector percentage
    #[serde(default)]
    pub basic_materials: Option<FormattedValue<f64>>,

    /// Consumer defensive sector percentage
    #[serde(default)]
    pub consumer_defensive: Option<FormattedValue<f64>>,

    /// Technology sector percentage
    #[serde(default)]
    pub technology: Option<FormattedValue<f64>>,

    /// Communication services sector percentage
    #[serde(default)]
    pub communication_services: Option<FormattedValue<f64>>,

    /// Financial services sector percentage
    #[serde(default)]
    pub financial_services: Option<FormattedValue<f64>>,

    /// Utilities sector percentage
    #[serde(default)]
    pub utilities: Option<FormattedValue<f64>>,

    /// Industrials sector percentage
    #[serde(default)]
    pub industrials: Option<FormattedValue<f64>>,

    /// Energy sector percentage
    #[serde(default)]
    pub energy: Option<FormattedValue<f64>>,

    /// Healthcare sector percentage
    #[serde(default)]
    pub healthcare: Option<FormattedValue<f64>>,
}
