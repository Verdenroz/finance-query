//! GraphQL types for the keyless providers that have no other schema home:
//! CFTC Commitments of Traders. (GDELT news reuses `GqlNews`, since it
//! populates the same canonical `News` model as every other news source.)

use async_graphql::{ComplexObject, Result, SimpleObject};
use serde::Deserialize;

use crate::graphql::pagination::{self, Page};

/// Mirrors `finance_query::cftc::CommitmentsOfTraders`.
// The library model is plain snake_case, so a `#[serde(rename_all)]` here would
// match nothing and `#[serde(default)]` would zero it — see tests/gql_wire_shape.rs.
#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase", complex)]
#[serde(default)]
pub struct GqlCommitmentsOfTraders {
    /// The symbol the series was requested with (`"GC=F"`, or a raw CFTC code).
    pub symbol: String,
    /// CFTC's own market and exchange name.
    pub market_and_exchange_name: String,
    /// CFTC contract market code identifying this market.
    pub cftc_contract_market_code: String,
    /// Weekly observations, oldest first. Paginated via the resolver below.
    #[graphql(skip)]
    pub observations: Vec<GqlCotObservation>,
}

#[ComplexObject(rename_fields = "camelCase")]
impl GqlCommitmentsOfTraders {
    /// Weekly report rows, oldest first.
    async fn observations(
        &self,
        #[graphql(desc = "Max rows to return; omitted = every row in one page")] first: Option<i32>,
        #[graphql(desc = "Opaque continuation cursor from a previous page's endCursor")]
        after: Option<String>,
    ) -> Result<Page<GqlCotObservation>> {
        pagination::paginate(&self.observations, first, after).await
    }
}

/// Mirrors `finance_query::cftc::CotObservation` — one weekly report row,
/// broken down by trader category.
// Same no-serde-rename rule as `GqlCommitmentsOfTraders` above.
#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(default)]
pub struct GqlCotObservation {
    /// Report date (`YYYY-MM-DD`) — the Tuesday the report is as of.
    pub report_date: String,
    pub open_interest: Option<i64>,
    pub producer_merchant_long: Option<i64>,
    pub producer_merchant_short: Option<i64>,
    pub swap_dealer_long: Option<i64>,
    pub swap_dealer_short: Option<i64>,
    pub swap_dealer_spread: Option<i64>,
    pub managed_money_long: Option<i64>,
    pub managed_money_short: Option<i64>,
    pub managed_money_spread: Option<i64>,
    pub other_reportable_long: Option<i64>,
    pub other_reportable_short: Option<i64>,
    pub other_reportable_spread: Option<i64>,
    pub total_reportable_long: Option<i64>,
    pub total_reportable_short: Option<i64>,
    pub nonreportable_long: Option<i64>,
    pub nonreportable_short: Option<i64>,
}
