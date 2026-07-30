use serde::{Deserialize, Serialize};

/// Supported regions for Yahoo Finance regional APIs
///
/// Each region has predefined language and region codes that work together.
/// Using the Region enum ensures correct lang/region pairing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum Region {
    /// Argentina (es-AR, AR)
    #[serde(rename = "AR")]
    Argentina,
    /// Australia (en-AU, AU)
    #[serde(rename = "AU")]
    Australia,
    /// Brazil (pt-BR, BR)
    #[serde(rename = "BR")]
    Brazil,
    /// Canada (en-CA, CA)
    #[serde(rename = "CA")]
    Canada,
    /// China (zh-CN, CN)
    #[serde(rename = "CN")]
    China,
    /// Denmark (da-DK, DK)
    #[serde(rename = "DK")]
    Denmark,
    /// Finland (fi-FI, FI)
    #[serde(rename = "FI")]
    Finland,
    /// France (fr-FR, FR)
    #[serde(rename = "FR")]
    France,
    /// Germany (de-DE, DE)
    #[serde(rename = "DE")]
    Germany,
    /// Greece (el-GR, GR)
    #[serde(rename = "GR")]
    Greece,
    /// Hong Kong (zh-Hant-HK, HK)
    #[serde(rename = "HK")]
    HongKong,
    /// India (en-IN, IN)
    #[serde(rename = "IN")]
    India,
    /// Israel (he-IL, IL)
    #[serde(rename = "IL")]
    Israel,
    /// Italy (it-IT, IT)
    #[serde(rename = "IT")]
    Italy,
    /// Japan (ja-JP, JP)
    #[serde(rename = "JP")]
    Japan,
    /// South Korea (ko-KR, KR)
    #[serde(rename = "KR")]
    Korea,
    /// Malaysia (ms-MY, MY)
    #[serde(rename = "MY")]
    Malaysia,
    /// Mexico (es-MX, MX)
    #[serde(rename = "MX")]
    Mexico,
    /// New Zealand (en-NZ, NZ)
    #[serde(rename = "NZ")]
    NewZealand,
    /// Norway (nb-NO, NO)
    #[serde(rename = "NO")]
    Norway,
    /// Portugal (pt-PT, PT)
    #[serde(rename = "PT")]
    Portugal,
    /// Qatar (ar-QA, QA)
    #[serde(rename = "QA")]
    Qatar,
    /// Russia (ru-RU, RU)
    #[serde(rename = "RU")]
    Russia,
    /// Singapore (en-SG, SG)
    #[serde(rename = "SG")]
    Singapore,
    /// Spain (es-ES, ES)
    #[serde(rename = "ES")]
    Spain,
    /// Sweden (sv-SE, SE)
    #[serde(rename = "SE")]
    Sweden,
    /// Taiwan (zh-TW, TW)
    #[serde(rename = "TW")]
    Taiwan,
    /// Thailand (th-TH, TH)
    #[serde(rename = "TH")]
    Thailand,
    /// Turkey (tr-TR, TR)
    #[serde(rename = "TR")]
    Turkey,
    /// United Kingdom (en-GB, GB)
    #[serde(rename = "GB", alias = "UK")]
    UnitedKingdom,
    /// United States (en-US, US) - Default
    #[default]
    #[serde(rename = "US")]
    UnitedStates,
    /// Vietnam (vi-VN, VN)
    #[serde(rename = "VN")]
    Vietnam,
}

impl Region {
    /// Get the language code for this region
    ///
    /// # Example
    ///
    /// ```
    /// use finance_query::Region;
    ///
    /// assert_eq!(Region::France.lang(), "fr-FR");
    /// assert_eq!(Region::UnitedStates.lang(), "en-US");
    /// ```
    pub fn lang(&self) -> &'static str {
        match self {
            Region::Argentina => "es-AR",
            Region::Australia => "en-AU",
            Region::Brazil => "pt-BR",
            Region::Canada => "en-CA",
            Region::China => "zh-CN",
            Region::Denmark => "da-DK",
            Region::Finland => "fi-FI",
            Region::France => "fr-FR",
            Region::Germany => "de-DE",
            Region::Greece => "el-GR",
            Region::HongKong => "zh-Hant-HK",
            Region::India => "en-IN",
            Region::Israel => "he-IL",
            Region::Italy => "it-IT",
            Region::Japan => "ja-JP",
            Region::Korea => "ko-KR",
            Region::Malaysia => "ms-MY",
            Region::Mexico => "es-MX",
            Region::NewZealand => "en-NZ",
            Region::Norway => "nb-NO",
            Region::Portugal => "pt-PT",
            Region::Qatar => "ar-QA",
            Region::Russia => "ru-RU",
            Region::Singapore => "en-SG",
            Region::Spain => "es-ES",
            Region::Sweden => "sv-SE",
            Region::Taiwan => "zh-TW",
            Region::Thailand => "th-TH",
            Region::Turkey => "tr-TR",
            Region::UnitedKingdom => "en-GB",
            Region::UnitedStates => "en-US",
            Region::Vietnam => "vi-VN",
        }
    }

    /// Get the region code for this region
    ///
    /// # Example
    ///
    /// ```
    /// use finance_query::Region;
    ///
    /// assert_eq!(Region::France.region(), "FR");
    /// assert_eq!(Region::UnitedStates.region(), "US");
    /// ```
    pub fn region(&self) -> &'static str {
        match self {
            Region::Argentina => "AR",
            Region::Australia => "AU",
            Region::Brazil => "BR",
            Region::Canada => "CA",
            Region::China => "CN",
            Region::Denmark => "DK",
            Region::Finland => "FI",
            Region::France => "FR",
            Region::Germany => "DE",
            Region::Greece => "GR",
            Region::HongKong => "HK",
            Region::India => "IN",
            Region::Israel => "IL",
            Region::Italy => "IT",
            Region::Japan => "JP",
            Region::Korea => "KR",
            Region::Malaysia => "MY",
            Region::Mexico => "MX",
            Region::NewZealand => "NZ",
            Region::Norway => "NO",
            Region::Portugal => "PT",
            Region::Qatar => "QA",
            Region::Russia => "RU",
            Region::Singapore => "SG",
            Region::Spain => "ES",
            Region::Sweden => "SE",
            Region::Taiwan => "TW",
            Region::Thailand => "TH",
            Region::Turkey => "TR",
            Region::UnitedKingdom => "GB",
            Region::UnitedStates => "US",
            Region::Vietnam => "VN",
        }
    }

    /// UTC offset in seconds for the region's primary exchange.
    ///
    /// Returns the standard-time (non-DST) UTC offset of each country's main
    /// exchange. This is used by the backtesting engine to align higher-timeframe
    /// resampling bucket boundaries to local calendar weeks and months, preventing
    /// APAC and other non-UTC exchanges from having bars mis-bucketed into the
    /// prior week due to UTC midnight falling inside their local trading day.
    ///
    /// # Note
    ///
    /// DST transitions are not modelled. For exchanges in regions with DST
    /// (e.g. NYSE, LSE) the boundary shift is at most ±1 hour and affects only
    /// the transition candles. This is a deliberate simplification — exact DST
    /// handling would require a timezone database dependency.
    pub const fn utc_offset_secs(&self) -> i64 {
        match self {
            // UTC-5 (NYSE/TSX winter)
            Region::UnitedStates | Region::Canada => -18_000,
            // UTC-6 (BMV — Mexico abolished DST for most of the country in 2022)
            Region::Mexico => -21_600,
            // UTC-3 (BYMA / B3 winter)
            Region::Argentina | Region::Brazil => -10_800,
            // UTC+0 (LSE / Euronext Lisbon)
            Region::UnitedKingdom | Region::Portugal => 0,
            // UTC+1 (Euronext Paris/Amsterdam/Milan/Madrid, Oslo, Stockholm, Copenhagen, Helsinki)
            Region::France
            | Region::Germany
            | Region::Italy
            | Region::Spain
            | Region::Norway
            | Region::Sweden
            | Region::Denmark
            | Region::Finland => 3_600,
            // UTC+2 (Athens, Tel Aviv, Moscow — note Russia stays UTC+3 year-round)
            Region::Greece | Region::Israel => 7_200,
            // UTC+3 (MOEX — no DST since 2014; Qatar/AST has no DST)
            Region::Turkey | Region::Russia | Region::Qatar => 10_800,
            // UTC+5:30 (BSE/NSE — India has no DST)
            Region::India => 19_800,
            // UTC+7 (SET Bangkok, HSX Hanoi)
            Region::Thailand | Region::Vietnam => 25_200,
            // UTC+8 (SSE/SZSE, HKEX, SGX, Bursa Malaysia, TWSE)
            Region::China
            | Region::HongKong
            | Region::Singapore
            | Region::Malaysia
            | Region::Taiwan => 28_800,
            // UTC+9 (TSE, KRX — neither observes DST)
            Region::Japan | Region::Korea => 32_400,
            // UTC+10 (ASX — AEST winter)
            Region::Australia => 36_000,
            // UTC+12 (NZX — NZST winter)
            Region::NewZealand => 43_200,
        }
    }
}

impl std::str::FromStr for Region {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "AR" => Ok(Region::Argentina),
            "AU" => Ok(Region::Australia),
            "BR" => Ok(Region::Brazil),
            "CA" => Ok(Region::Canada),
            "CN" => Ok(Region::China),
            "DK" => Ok(Region::Denmark),
            "FI" => Ok(Region::Finland),
            "FR" => Ok(Region::France),
            "DE" => Ok(Region::Germany),
            "GR" => Ok(Region::Greece),
            "HK" => Ok(Region::HongKong),
            "IN" => Ok(Region::India),
            "IL" => Ok(Region::Israel),
            "IT" => Ok(Region::Italy),
            "JP" => Ok(Region::Japan),
            "KR" => Ok(Region::Korea),
            "MY" => Ok(Region::Malaysia),
            "MX" => Ok(Region::Mexico),
            "NZ" => Ok(Region::NewZealand),
            "NO" => Ok(Region::Norway),
            "PT" => Ok(Region::Portugal),
            "QA" => Ok(Region::Qatar),
            "RU" => Ok(Region::Russia),
            "SG" => Ok(Region::Singapore),
            "ES" => Ok(Region::Spain),
            "SE" => Ok(Region::Sweden),
            "TW" => Ok(Region::Taiwan),
            "TH" => Ok(Region::Thailand),
            "TR" => Ok(Region::Turkey),
            "GB" | "UK" => Ok(Region::UnitedKingdom),
            "US" => Ok(Region::UnitedStates),
            "VN" => Ok(Region::Vietnam),
            _ => Err(()),
        }
    }
}

impl From<Region> for String {
    /// Returns the lowercase two-letter country code used by the Yahoo Finance screener
    /// (e.g. `"us"`, `"gb"`).
    fn from(v: Region) -> Self {
        v.region().to_lowercase()
    }
}
