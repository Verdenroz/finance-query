/// Typed exchange code for screener queries.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExchangeCode {
    // ── US Equity ─────────────────────────────────────────────────────────
    /// NYSE American / AMEX ("ASE")
    Ase,
    /// OTC Bulletin Board ("BTS")
    Bts,
    /// NASDAQ Capital Market ("NCM")
    Ncm,
    /// NASDAQ Global Market ("NGM")
    Ngm,
    /// NASDAQ Global Select Market ("NMS") — primary NASDAQ tier
    Nms,
    /// New York Stock Exchange ("NYQ")
    Nyq,
    /// NYSE Arca ("PCX")
    Pcx,
    /// OTC Pink Sheets / OTC Markets ("PNK")
    Pnk,
    // ── US Funds ──────────────────────────────────────────────────────────
    /// NASDAQ — used for US mutual funds ("NAS")
    Nas,
    // ── International ─────────────────────────────────────────────────────
    /// Australian Securities Exchange ("ASX")
    Asx,
    /// Bombay Stock Exchange ("BSE")
    Bse,
    /// Hong Kong Stock Exchange ("HKG")
    Hkg,
    /// Korea Exchange ("KRX")
    Krx,
    /// London Stock Exchange ("LSE")
    Lse,
    /// National Stock Exchange of India ("NSI")
    Nsi,
    /// Shanghai Stock Exchange ("SHH")
    Shh,
    /// Shenzhen Stock Exchange ("SHZ")
    Shz,
    /// Tokyo Stock Exchange ("TYO")
    Tyo,
    /// Toronto Stock Exchange ("TOR")
    Tor,
    /// XETRA / Deutsche Börse ("GER")
    Ger,
}

impl ExchangeCode {
    /// Returns the exchange code string used by Yahoo Finance.
    pub fn as_str(self) -> &'static str {
        match self {
            ExchangeCode::Ase => "ASE",
            ExchangeCode::Bts => "BTS",
            ExchangeCode::Ncm => "NCM",
            ExchangeCode::Ngm => "NGM",
            ExchangeCode::Nms => "NMS",
            ExchangeCode::Nyq => "NYQ",
            ExchangeCode::Pcx => "PCX",
            ExchangeCode::Pnk => "PNK",
            ExchangeCode::Nas => "NAS",
            ExchangeCode::Asx => "ASX",
            ExchangeCode::Bse => "BSE",
            ExchangeCode::Hkg => "HKG",
            ExchangeCode::Krx => "KRX",
            ExchangeCode::Lse => "LSE",
            ExchangeCode::Nsi => "NSI",
            ExchangeCode::Shh => "SHH",
            ExchangeCode::Shz => "SHZ",
            ExchangeCode::Tyo => "TYO",
            ExchangeCode::Tor => "TOR",
            ExchangeCode::Ger => "GER",
        }
    }
}

impl From<ExchangeCode> for String {
    fn from(v: ExchangeCode) -> Self {
        v.as_str().to_string()
    }
}
