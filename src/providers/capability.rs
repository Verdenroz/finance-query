//! The [`Capability`] bitflags a provider declares.

use super::Provider;

/// Capability bits that a provider can declare.
///
/// Route a capability to specific providers using `.route(Capability::QUOTE, [Provider::Fmp])`.
/// If no route is configured for a capability, all providers declaring that capability are used.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Capability(u32);

impl Capability {
    /// Equity quote data — price, volume, market cap, fundamentals summary.
    pub const QUOTE: Self = Self(1 << 0);
    /// Historical OHLCV chart data across intervals and ranges.
    pub const CHART: Self = Self(1 << 1);
    /// Financial statements — income, balance sheet, cash flow.
    pub const FUNDAMENTALS: Self = Self(1 << 2);
    /// Corporate events — news, recommendations, SEC filings metadata.
    pub const CORPORATE: Self = Self(1 << 3);
    /// Options chains and contract data.
    pub const OPTIONS: Self = Self(1 << 4);
    /// Symbol discovery — search, screeners, exchange and ticker reference data.
    pub const DISCOVERY: Self = Self(1 << 5);

    /// Cryptocurrency quotes and market data.
    pub const CRYPTO: Self = Self(1 << 6);
    /// Macro-economic data series (FRED, GDP, CPI, etc.).
    pub const ECONOMIC: Self = Self(1 << 7);
    /// Market-wide calendars — earnings, IPOs, dividends, splits, economic events.
    pub const CALENDAR: Self = Self(1 << 8);

    /// Foreign exchange currency pair quotes.
    pub const FOREX: Self = Self(1 << 9);
    /// Stock market index quotes (S&P 500, NASDAQ, etc.).
    pub const INDICES: Self = Self(1 << 10);
    /// Futures contract quotes.
    pub const FUTURES: Self = Self(1 << 11);
    /// Commodity price quotes (gold, oil, etc.).
    pub const COMMODITIES: Self = Self(1 << 12);
    /// Market-wide statistics — sector/industry performance and movers.
    pub const MARKET: Self = Self(1 << 13);

    /// SEC EDGAR filing data.
    pub const FILINGS: Self = Self(1 << 14);

    /// The empty capability set — starting point for derived accumulation.
    pub const NONE: Self = Self(0);

    /// Const-context union, for capability-set consts ([`std::ops::BitOr`]
    /// isn't const-callable).
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Every single-bit capability, in declaration order.
    ///
    /// Combined sets are not included: this yields exactly the constants above.
    pub fn all() -> impl Iterator<Item = Self> {
        Self::ALL.iter().map(|(capability, _)| *capability)
    }

    /// Returns `true` if this capability set includes all bits in `other`.
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Every single-bit capability paired with its name — the one place names
    /// and bits meet. `name()`, `Display`, and the bit-uniqueness test all
    /// derive from this table.
    const ALL: [(Self, &'static str); 15] = [
        (Self::QUOTE, "quote"),
        (Self::CHART, "chart"),
        (Self::FUNDAMENTALS, "fundamentals"),
        (Self::CORPORATE, "corporate"),
        (Self::OPTIONS, "options"),
        (Self::DISCOVERY, "discovery"),
        (Self::CRYPTO, "crypto"),
        (Self::ECONOMIC, "economic"),
        (Self::CALENDAR, "calendar"),
        (Self::MARKET, "market"),
        (Self::FOREX, "forex"),
        (Self::INDICES, "indices"),
        (Self::FUTURES, "futures"),
        (Self::COMMODITIES, "commodities"),
        (Self::FILINGS, "filings"),
    ];

    /// Returns a short lowercase name for this capability (e.g., `"quote"`, `"chart"`).
    ///
    /// Returns `"unknown"` for combined capability flags or unrecognised bits;
    /// [`Display`](std::fmt::Display) spells combined sets out instead (e.g.
    /// `"quote|chart"`).
    pub fn name(self) -> &'static str {
        Self::ALL
            .iter()
            .find(|(cap, _)| cap.0 == self.0)
            .map(|(_, name)| *name)
            .unwrap_or("unknown")
    }
}

impl std::fmt::Display for Capability {
    /// Single capabilities print their [`name`](Capability::name); combined
    /// sets are spelled out `|`-separated (e.g. `"quote|chart"`) rather than
    /// collapsing to `"unknown"`.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut remaining = self.0;
        let mut first = true;
        for (cap, name) in Self::ALL {
            if remaining & cap.0 != 0 {
                if !first {
                    f.write_str("|")?;
                }
                f.write_str(name)?;
                first = false;
                remaining &= !cap.0;
            }
        }
        if first || remaining != 0 {
            if !first {
                f.write_str("|")?;
            }
            f.write_str("unknown")?;
        }
        Ok(())
    }
}

impl Capability {
    /// Providers whose `capabilities()` declare this capability, regardless of
    /// which providers are actually configured/feature-enabled in this build.
    ///
    /// Purely informational — used to make [`crate::FinanceError::NotSupported`]/
    /// [`crate::FinanceError::NoProviderAvailable`] point at what would need to
    /// be enabled (feature flag) and/or routed (`Providers::builder().route(...)`).
    ///
    /// Built-ins only. A registered custom provider never appears here, since
    /// [`Provider::capabilities`] cannot see an adapter's accessors.
    pub fn candidate_providers(self) -> Vec<Provider> {
        Provider::all()
            .into_iter()
            .filter(|p| p.capabilities().contains(self))
            .collect()
    }
}

impl std::ops::BitOr for Capability {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capability_bits_are_distinct_single_bits() {
        for (i, (a, name_a)) in Capability::ALL.iter().enumerate() {
            assert_eq!(a.0.count_ones(), 1, "{name_a} is not a single bit");
            for (b, name_b) in &Capability::ALL[i + 1..] {
                assert_ne!(a.0, b.0, "{name_a} and {name_b} share a bit");
            }
        }
    }

    #[test]
    fn display_spells_out_combined_capabilities() {
        assert_eq!(Capability::QUOTE.to_string(), "quote");
        assert_eq!(
            (Capability::QUOTE | Capability::CHART).to_string(),
            "quote|chart"
        );
        assert_eq!(
            (Capability::FILINGS | Capability::CORPORATE).to_string(),
            "corporate|filings"
        );
        assert_eq!(Capability::NONE.to_string(), "unknown");
        // name() keeps its documented single-bit contract.
        assert_eq!((Capability::QUOTE | Capability::CHART).name(), "unknown");
    }
}
