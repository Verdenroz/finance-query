/// Predefined Yahoo Finance screener selector
///
/// Passed to `finance::screener()` or `client.get_screener()` to select one of the
/// 15 built-in Yahoo Finance screeners (equity or fund).
///
/// The `alias`es mirror the shorthands [`FromStr`](std::str::FromStr) accepts, so
/// deserializing (axum path extraction, JSON) takes the same spellings parsing does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Screener {
    // Equity screeners
    /// Small caps with high EPS growth, sorted by volume
    AggressiveSmallCaps,
    /// Top gaining stocks (>3% change, >$2B market cap)
    #[serde(alias = "gainers")]
    DayGainers,
    /// Top losing stocks (<-2.5% change, >$2B market cap)
    #[serde(alias = "losers")]
    DayLosers,
    /// Tech stocks with 25%+ revenue and EPS growth
    #[serde(alias = "growth-tech")]
    GrowthTechnologyStocks,
    /// Most actively traded stocks by volume
    #[serde(alias = "actives")]
    MostActives,
    /// Stocks with highest short interest percentage
    #[serde(alias = "most-shorted")]
    MostShortedStocks,
    /// Small cap gainers (<$2B market cap)
    SmallCapGainers,
    /// Low P/E (<20), low PEG (<1), high EPS growth (25%+)
    #[serde(alias = "undervalued-growth")]
    UndervaluedGrowthStocks,
    /// Large caps ($10B-$100B) with low P/E and PEG
    #[serde(alias = "undervalued-large")]
    UndervaluedLargeCaps,
    // Fund screeners
    /// Low-risk foreign large cap funds (4-5 star rated)
    ConservativeForeignFunds,
    /// High yield bond funds (4-5 star rated)
    HighYieldBond,
    /// Large blend core funds (4-5 star rated)
    PortfolioAnchors,
    /// Large growth funds (4-5 star rated)
    SolidLargeGrowthFunds,
    /// Mid-cap growth funds (4-5 star rated)
    SolidMidcapGrowthFunds,
    /// Top performing mutual funds by percent change
    TopMutualFunds,
}

impl Screener {
    /// Convert to Yahoo Finance scrId parameter value (SCREAMING_SNAKE_CASE)
    pub fn as_scr_id(&self) -> &'static str {
        match self {
            Screener::AggressiveSmallCaps => "aggressive_small_caps",
            Screener::DayGainers => "day_gainers",
            Screener::DayLosers => "day_losers",
            Screener::GrowthTechnologyStocks => "growth_technology_stocks",
            Screener::MostActives => "most_actives",
            Screener::MostShortedStocks => "most_shorted_stocks",
            Screener::SmallCapGainers => "small_cap_gainers",
            Screener::UndervaluedGrowthStocks => "undervalued_growth_stocks",
            Screener::UndervaluedLargeCaps => "undervalued_large_caps",
            Screener::ConservativeForeignFunds => "conservative_foreign_funds",
            Screener::HighYieldBond => "high_yield_bond",
            Screener::PortfolioAnchors => "portfolio_anchors",
            Screener::SolidLargeGrowthFunds => "solid_large_growth_funds",
            Screener::SolidMidcapGrowthFunds => "solid_midcap_growth_funds",
            Screener::TopMutualFunds => "top_mutual_funds",
        }
    }

    /// Parse from string, returns None on invalid input
    ///
    /// # Example
    /// ```
    /// use finance_query::Screener;
    ///
    /// assert_eq!(Screener::parse("most-actives"), Some(Screener::MostActives));
    /// assert_eq!(Screener::parse("day-gainers"), Some(Screener::DayGainers));
    /// ```
    pub fn parse(s: &str) -> Option<Self> {
        s.parse().ok()
    }

    /// List all valid screener types for error messages
    pub fn valid_types() -> &'static str {
        "aggressive-small-caps, day-gainers, day-losers, growth-technology-stocks, \
         most-actives, most-shorted-stocks, small-cap-gainers, undervalued-growth-stocks, \
         undervalued-large-caps, conservative-foreign-funds, high-yield-bond, \
         portfolio-anchors, solid-large-growth-funds, solid-midcap-growth-funds, \
         top-mutual-funds"
    }

    /// Get all screener types as an array
    pub fn all() -> &'static [Screener] {
        &[
            Screener::AggressiveSmallCaps,
            Screener::DayGainers,
            Screener::DayLosers,
            Screener::GrowthTechnologyStocks,
            Screener::MostActives,
            Screener::MostShortedStocks,
            Screener::SmallCapGainers,
            Screener::UndervaluedGrowthStocks,
            Screener::UndervaluedLargeCaps,
            Screener::ConservativeForeignFunds,
            Screener::HighYieldBond,
            Screener::PortfolioAnchors,
            Screener::SolidLargeGrowthFunds,
            Screener::SolidMidcapGrowthFunds,
            Screener::TopMutualFunds,
        ]
    }
}

impl std::str::FromStr for Screener {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace('_', "-").as_str() {
            "aggressive-small-caps" => Ok(Screener::AggressiveSmallCaps),
            "day-gainers" | "gainers" => Ok(Screener::DayGainers),
            "day-losers" | "losers" => Ok(Screener::DayLosers),
            "growth-technology-stocks" | "growth-tech" => Ok(Screener::GrowthTechnologyStocks),
            "most-actives" | "actives" => Ok(Screener::MostActives),
            "most-shorted-stocks" | "most-shorted" => Ok(Screener::MostShortedStocks),
            "small-cap-gainers" => Ok(Screener::SmallCapGainers),
            "undervalued-growth-stocks" | "undervalued-growth" => {
                Ok(Screener::UndervaluedGrowthStocks)
            }
            "undervalued-large-caps" | "undervalued-large" => Ok(Screener::UndervaluedLargeCaps),
            "conservative-foreign-funds" => Ok(Screener::ConservativeForeignFunds),
            "high-yield-bond" => Ok(Screener::HighYieldBond),
            "portfolio-anchors" => Ok(Screener::PortfolioAnchors),
            "solid-large-growth-funds" => Ok(Screener::SolidLargeGrowthFunds),
            "solid-midcap-growth-funds" => Ok(Screener::SolidMidcapGrowthFunds),
            "top-mutual-funds" => Ok(Screener::TopMutualFunds),
            _ => Err(()),
        }
    }
}
