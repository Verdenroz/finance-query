//! US Treasury securities auction models.
//!
//! Served through the [`Capability::ECONOMIC`](crate::Capability::ECONOMIC)
//! route from US Treasury FiscalData. [`TreasuryAuction`] covers auctions that
//! have been announced or settled; [`UpcomingAuction`] covers the announced
//! schedule ahead of the auction date.

use serde::{Deserialize, Serialize};

/// One US Treasury securities auction.
///
/// Rows exist from announcement onwards, so an auction whose date has not
/// passed carries its terms (offering amount, maturity) with every result
/// field still `None`.
///
/// Bills and coupon securities report their price differently: a bill prices
/// off [`high_discnt_rate`](Self::high_discnt_rate) and
/// [`high_investment_rate`](Self::high_investment_rate) and leaves
/// [`high_yield`](Self::high_yield) unset, while notes and bonds do the
/// reverse.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TreasuryAuction {
    /// Date the row was published (`YYYY-MM-DD`).
    pub record_date: String,
    /// CUSIP of the security offered.
    pub cusip: String,
    /// Security class (`"Bill"`, `"Note"`, `"Bond"`, …).
    pub security_type: String,
    /// Term as Treasury states it (e.g. `"13-Week"`, `"29-Year 11-Month"`).
    pub security_term: String,
    /// Date the auction was held (`YYYY-MM-DD`).
    pub auction_date: String,
    /// Date the security settles (`YYYY-MM-DD`).
    pub issue_date: String,
    /// Date the security matures (`YYYY-MM-DD`).
    pub maturity_date: String,
    /// Whether the auction reopens an existing security rather than issuing a
    /// new one.
    pub reopening: Option<bool>,
    /// Auction pricing method (e.g. `"Single-Price"`).
    pub auction_format: Option<String>,
    /// Coupon rate (%), for coupon-bearing securities.
    pub int_rate: Option<f64>,
    /// Par amount offered (US dollars).
    pub offering_amt: Option<f64>,
    /// Par amount bid across all bidders (US dollars).
    pub total_tendered: Option<f64>,
    /// Par amount awarded across all bidders (US dollars).
    pub total_accepted: Option<f64>,
    /// Total tendered divided by total accepted, the standard demand gauge.
    pub bid_to_cover_ratio: Option<f64>,
    /// Highest accepted yield (%), for notes and bonds.
    pub high_yield: Option<f64>,
    /// Highest accepted discount rate (%), for bills.
    pub high_discnt_rate: Option<f64>,
    /// Coupon-equivalent yield of the high discount rate (%), for bills.
    pub high_investment_rate: Option<f64>,
    /// Price per $100 par at the highest accepted bid.
    pub high_price: Option<f64>,
    /// Par amount awarded to primary dealers (US dollars).
    pub primary_dealer_accepted: Option<f64>,
    /// Par amount awarded to direct bidders (US dollars).
    pub direct_bidder_accepted: Option<f64>,
    /// Par amount awarded to indirect bidders (US dollars).
    pub indirect_bidder_accepted: Option<f64>,
    /// Par amount awarded on competitive bids (US dollars).
    pub comp_accepted: Option<f64>,
    /// Par amount awarded on non-competitive bids (US dollars).
    pub noncomp_accepted: Option<f64>,
    /// Par amount awarded to the Federal Reserve's System Open Market Account
    /// (US dollars).
    pub soma_accepted: Option<f64>,
}

/// A US Treasury auction that has been scheduled but not yet held.
///
/// Treasury announces terms in stages, so [`offering_amt`](Self::offering_amt)
/// is unset until the formal announcement lands.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct UpcomingAuction {
    /// Date the schedule was published (`YYYY-MM-DD`).
    pub record_date: String,
    /// Security class (`"Bill"`, `"Note"`, `"Bond"`, …).
    pub security_type: String,
    /// Term as Treasury states it (e.g. `"13-Week"`).
    pub security_term: String,
    /// CUSIP of the security to be offered.
    pub cusip: String,
    /// Whether the auction reopens an existing security.
    pub reopening: Option<bool>,
    /// Par amount to be offered (US dollars), once announced.
    pub offering_amt: Option<f64>,
    /// Date Treasury announces the auction's terms (`YYYY-MM-DD`).
    pub announcement_date: String,
    /// Date the auction will be held (`YYYY-MM-DD`).
    pub auction_date: String,
    /// Date the security will settle (`YYYY-MM-DD`).
    pub issue_date: String,
}

/// Which auctions [`EconomicCatalog::treasury_auctions`] should return.
///
/// Every field is optional; the default asks for the most recent auctions of
/// every type. Build one with [`TreasuryAuctionQuery::new`] and the chainable
/// setters.
///
/// [`EconomicCatalog::treasury_auctions`]: crate::domains::EconomicCatalog::treasury_auctions
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct TreasuryAuctionQuery {
    /// Restrict to one security class (`"Bill"`, `"Note"`, `"Bond"`, …).
    pub security_type: Option<String>,
    /// Restrict to one term, spelled as Treasury spells it (e.g. `"13-Week"`).
    pub security_term: Option<String>,
    /// Earliest auction date to include (`YYYY-MM-DD`).
    pub from: Option<String>,
    /// Latest auction date to include (`YYYY-MM-DD`).
    pub to: Option<String>,
    /// Maximum auctions to return, newest first.
    pub limit: Option<u32>,
}

impl TreasuryAuctionQuery {
    /// An unrestricted query — the most recent auctions of every type.
    pub fn new() -> Self {
        Self::default()
    }

    /// Restrict to one security class.
    pub fn security_type(mut self, security_type: impl Into<String>) -> Self {
        self.security_type = Some(security_type.into());
        self
    }

    /// Restrict to one security term.
    pub fn security_term(mut self, security_term: impl Into<String>) -> Self {
        self.security_term = Some(security_term.into());
        self
    }

    /// Restrict auction dates to `[from, to]` (`YYYY-MM-DD`).
    pub fn dates(mut self, from: Option<&str>, to: Option<&str>) -> Self {
        self.from = from.map(str::to_string);
        self.to = to.map(str::to_string);
        self
    }

    /// Return at most `limit` auctions, newest first.
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
}
