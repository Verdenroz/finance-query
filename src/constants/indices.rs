/// Region categories for world indices
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Region {
    /// North and South America
    Americas,
    /// European markets
    Europe,
    /// Asia and Pacific markets
    AsiaPacific,
    /// Middle East and Africa
    MiddleEastAfrica,
    /// Currency indices
    Currencies,
}

impl std::str::FromStr for Region {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace(['-', '_'], "").as_str() {
            "americas" | "america" | "am" => Ok(Region::Americas),
            "europe" | "eu" => Ok(Region::Europe),
            "asiapacific" | "asia" | "apac" => Ok(Region::AsiaPacific),
            "middleeastafrica" | "mea" | "emea" => Ok(Region::MiddleEastAfrica),
            "currencies" | "currency" | "fx" => Ok(Region::Currencies),
            _ => Err(()),
        }
    }
}

impl Region {
    /// Parse from string, returns None on invalid input
    pub fn parse(s: &str) -> Option<Self> {
        s.parse().ok()
    }

    /// Get the symbols for this region
    pub fn symbols(&self) -> &'static [&'static str] {
        match self {
            Region::Americas => AMERICAS,
            Region::Europe => EUROPE,
            Region::AsiaPacific => ASIA_PACIFIC,
            Region::MiddleEastAfrica => MIDDLE_EAST_AFRICA,
            Region::Currencies => CURRENCIES,
        }
    }

    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Region::Americas => "americas",
            Region::Europe => "europe",
            Region::AsiaPacific => "asia-pacific",
            Region::MiddleEastAfrica => "middle-east-africa",
            Region::Currencies => "currencies",
        }
    }

    /// All region variants
    pub fn all() -> &'static [Region] {
        &[
            Region::Americas,
            Region::Europe,
            Region::AsiaPacific,
            Region::MiddleEastAfrica,
            Region::Currencies,
        ]
    }
}

/// Americas indices
pub const AMERICAS: &[&str] = &[
    "^GSPC",   // S&P 500
    "^DJI",    // Dow Jones Industrial Average
    "^IXIC",   // NASDAQ Composite
    "^NYA",    // NYSE Composite Index
    "^XAX",    // NYSE American Composite Index
    "^RUT",    // Russell 2000 Index
    "^VIX",    // CBOE Volatility Index
    "^GSPTSE", // S&P/TSX Composite (Canada)
    "^BVSP",   // IBOVESPA (Brazil)
    "^MXX",    // IPC MEXICO
    "^IPSA",   // S&P IPSA (Chile)
    "^MERV",   // MERVAL (Argentina)
];

/// Europe indices
pub const EUROPE: &[&str] = &[
    "^FTSE",            // FTSE 100 (UK)
    "^GDAXI",           // DAX (Germany)
    "^FCHI",            // CAC 40 (France)
    "^STOXX50E",        // EURO STOXX 50
    "^N100",            // Euronext 100 Index
    "^BFX",             // BEL 20 (Belgium)
    "^BUK100P",         // Cboe UK 100
    "MOEX.ME",          // Moscow Exchange
    "^125904-USD-STRD", // MSCI EUROPE
];

/// Asia Pacific indices
pub const ASIA_PACIFIC: &[&str] = &[
    "^N225",     // Nikkei 225 (Japan)
    "^HSI",      // Hang Seng Index (Hong Kong)
    "000001.SS", // SSE Composite Index (China)
    "^KS11",     // KOSPI (South Korea)
    "^TWII",     // Taiwan Weighted Index
    "^STI",      // STI Index (Singapore)
    "^AXJO",     // S&P/ASX 200 (Australia)
    "^AORD",     // All Ordinaries (Australia)
    "^NZ50",     // S&P/NZX 50 (New Zealand)
    "^BSESN",    // S&P BSE SENSEX (India)
    "^JKSE",     // IDX Composite (Indonesia)
    "^KLSE",     // FTSE Bursa Malaysia KLCI
];

/// Middle East & Africa indices
pub const MIDDLE_EAST_AFRICA: &[&str] = &[
    "^TA125.TA", // TA-125 (Israel)
    "^CASE30",   // EGX 30 (Egypt)
    "^JN0U.JO",  // Top 40 USD Net TRI (South Africa)
];

/// Currency indices
pub const CURRENCIES: &[&str] = &[
    "DX-Y.NYB", // US Dollar Index
    "^XDB",     // British Pound Currency Index
    "^XDE",     // Euro Currency Index
    "^XDN",     // Japanese Yen Currency Index
    "^XDA",     // Australian Dollar Currency Index
];

/// All world indices (all regions combined)
pub fn all_symbols() -> Vec<&'static str> {
    Region::all()
        .iter()
        .flat_map(|r| r.symbols().iter().copied())
        .collect()
}
