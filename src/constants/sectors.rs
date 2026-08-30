use serde::{Deserialize, Serialize};

/// Market sector types available on Yahoo Finance
///
/// The `alias`es mirror the shorthands [`FromStr`](std::str::FromStr) accepts, so
/// deserializing (axum path extraction, JSON) takes the same spellings parsing does.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Sector {
    /// Technology sector (software, semiconductors, hardware)
    #[serde(alias = "tech")]
    Technology,
    /// Financial Services sector (banks, insurance, asset management)
    #[serde(alias = "financials", alias = "financial")]
    FinancialServices,
    /// Consumer Cyclical sector (retail, automotive, leisure)
    ConsumerCyclical,
    /// Communication Services sector (telecom, media, entertainment)
    #[serde(alias = "communication")]
    CommunicationServices,
    /// Healthcare sector (pharma, biotech, medical devices)
    #[serde(alias = "health")]
    Healthcare,
    /// Industrials sector (aerospace, machinery, construction)
    #[serde(alias = "industrial")]
    Industrials,
    /// Consumer Defensive sector (food, beverages, household products)
    ConsumerDefensive,
    /// Energy sector (oil, gas, renewable energy)
    Energy,
    /// Basic Materials sector (chemicals, metals, mining)
    #[serde(alias = "materials")]
    BasicMaterials,
    /// Real Estate sector (REITs, property management)
    #[serde(alias = "realestate")]
    RealEstate,
    /// Utilities sector (electric, gas, water utilities)
    #[serde(alias = "utility")]
    Utilities,
}

impl Sector {
    /// Convert to Yahoo Finance API path segment (lowercase with hyphens)
    pub fn as_api_path(&self) -> &'static str {
        match self {
            Sector::Technology => "technology",
            Sector::FinancialServices => "financial-services",
            Sector::ConsumerCyclical => "consumer-cyclical",
            Sector::CommunicationServices => "communication-services",
            Sector::Healthcare => "healthcare",
            Sector::Industrials => "industrials",
            Sector::ConsumerDefensive => "consumer-defensive",
            Sector::Energy => "energy",
            Sector::BasicMaterials => "basic-materials",
            Sector::RealEstate => "real-estate",
            Sector::Utilities => "utilities",
        }
    }

    /// Get human-readable display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Sector::Technology => "Technology",
            Sector::FinancialServices => "Financial Services",
            Sector::ConsumerCyclical => "Consumer Cyclical",
            Sector::CommunicationServices => "Communication Services",
            Sector::Healthcare => "Healthcare",
            Sector::Industrials => "Industrials",
            Sector::ConsumerDefensive => "Consumer Defensive",
            Sector::Energy => "Energy",
            Sector::BasicMaterials => "Basic Materials",
            Sector::RealEstate => "Real Estate",
            Sector::Utilities => "Utilities",
        }
    }

    /// SPDR Select Sector ETF tracking this GICS sector, used to derive
    /// sector performance history from ETF price action when no provider
    /// serves it directly.
    pub fn spdr_etf(&self) -> &'static str {
        match self {
            Sector::Technology => "XLK",
            Sector::FinancialServices => "XLF",
            Sector::ConsumerCyclical => "XLY",
            Sector::CommunicationServices => "XLC",
            Sector::Healthcare => "XLV",
            Sector::Industrials => "XLI",
            Sector::ConsumerDefensive => "XLP",
            Sector::Energy => "XLE",
            Sector::BasicMaterials => "XLB",
            Sector::RealEstate => "XLRE",
            Sector::Utilities => "XLU",
        }
    }

    /// List all valid sector types for error messages
    pub fn valid_types() -> &'static str {
        "technology, financial-services, consumer-cyclical, communication-services, \
         healthcare, industrials, consumer-defensive, energy, basic-materials, \
         real-estate, utilities"
    }

    /// Get all sector types as an array
    pub fn all() -> &'static [Sector] {
        &[
            Sector::Technology,
            Sector::FinancialServices,
            Sector::ConsumerCyclical,
            Sector::CommunicationServices,
            Sector::Healthcare,
            Sector::Industrials,
            Sector::ConsumerDefensive,
            Sector::Energy,
            Sector::BasicMaterials,
            Sector::RealEstate,
            Sector::Utilities,
        ]
    }
}

impl std::str::FromStr for Sector {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace('_', "-").as_str() {
            "technology" | "tech" => Ok(Sector::Technology),
            "financial-services" | "financials" | "financial" => Ok(Sector::FinancialServices),
            "consumer-cyclical" => Ok(Sector::ConsumerCyclical),
            "communication-services" | "communication" => Ok(Sector::CommunicationServices),
            "healthcare" | "health" => Ok(Sector::Healthcare),
            "industrials" | "industrial" => Ok(Sector::Industrials),
            "consumer-defensive" => Ok(Sector::ConsumerDefensive),
            "energy" => Ok(Sector::Energy),
            "basic-materials" | "materials" => Ok(Sector::BasicMaterials),
            "real-estate" | "realestate" => Ok(Sector::RealEstate),
            "utilities" | "utility" => Ok(Sector::Utilities),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for Sector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl From<Sector> for String {
    /// Returns the display name used by Yahoo Finance screener (e.g. `"Technology"`).
    fn from(v: Sector) -> Self {
        v.display_name().to_string()
    }
}
