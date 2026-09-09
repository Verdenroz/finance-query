//! GraphQL types for US Treasury securities auctions.

use async_graphql::SimpleObject;
use serde::Deserialize;

/// One US Treasury securities auction. Rows exist from announcement onwards,
/// so an auction that has not been held yet carries its terms with every
/// result field `null`. Bills report `highDiscntRate`/`highInvestmentRate`
/// and leave `highYield` unset; notes and bonds do the reverse.
#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(default)]
pub struct GqlTreasuryAuction {
    pub record_date: String,
    pub cusip: String,
    pub security_type: String,
    pub security_term: String,
    pub auction_date: String,
    pub issue_date: String,
    pub maturity_date: String,
    pub reopening: Option<bool>,
    pub auction_format: Option<String>,
    pub int_rate: Option<f64>,
    pub offering_amt: Option<f64>,
    pub total_tendered: Option<f64>,
    pub total_accepted: Option<f64>,
    pub bid_to_cover_ratio: Option<f64>,
    pub high_yield: Option<f64>,
    pub high_discnt_rate: Option<f64>,
    pub high_investment_rate: Option<f64>,
    pub high_price: Option<f64>,
    pub primary_dealer_accepted: Option<f64>,
    pub direct_bidder_accepted: Option<f64>,
    pub indirect_bidder_accepted: Option<f64>,
    pub comp_accepted: Option<f64>,
    pub noncomp_accepted: Option<f64>,
    pub soma_accepted: Option<f64>,
}

/// A US Treasury auction scheduled but not yet held. `offeringAmt` stays
/// `null` until Treasury formally announces the auction's terms.
#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(default)]
pub struct GqlUpcomingAuction {
    pub record_date: String,
    pub security_type: String,
    pub security_term: String,
    pub cusip: String,
    pub reopening: Option<bool>,
    pub offering_amt: Option<f64>,
    pub announcement_date: String,
    pub auction_date: String,
    pub issue_date: String,
}
