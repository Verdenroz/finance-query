//! Corporate data models.
//!
//! Company profiles, officers, ownership, insider activity, and related data.

#[cfg(feature = "dataframe")]
macro_rules! rows_to_dataframe {
    ($wrapper:ty, $field:ident, $row:ty) => {
        impl $wrapper {
            /// Converts the rows to a polars DataFrame, one row per entry.
            ///
            /// Columns are those of the row type's `to_dataframe`; an empty list
            /// yields a zero-row frame with the full schema.
            pub fn to_dataframe(
                &self,
            ) -> ::polars::prelude::PolarsResult<::polars::prelude::DataFrame> {
                <$row>::vec_to_dataframe(&self.$field)
            }
        }

        impl crate::ToDataFrame for $wrapper {
            fn to_dataframe(
                &self,
            ) -> ::polars::prelude::PolarsResult<::polars::prelude::DataFrame> {
                <$wrapper>::to_dataframe(self)
            }

            fn vec_to_dataframe(
                items: &[Self],
            ) -> ::polars::prelude::PolarsResult<::polars::prelude::DataFrame> {
                crate::__dataframe::stack_rows(
                    <$row>::vec_to_dataframe(&[]),
                    items,
                    <$wrapper>::to_dataframe,
                )
            }

            fn to_nested_dataframe(
                &self,
            ) -> ::polars::prelude::PolarsResult<::polars::prelude::DataFrame> {
                <$row as crate::ToDataFrame>::vec_to_nested_dataframe(&self.$field)
            }

            fn vec_to_nested_dataframe(
                items: &[Self],
            ) -> ::polars::prelude::PolarsResult<::polars::prelude::DataFrame> {
                crate::__dataframe::stack_rows(
                    <$row as crate::ToDataFrame>::vec_to_nested_dataframe(&[]),
                    items,
                    <$wrapper as crate::ToDataFrame>::to_nested_dataframe,
                )
            }
        }
    };
}

// Sub-capability directories
/// Executive compensation and employee headcount.
pub mod governance;
/// News article models.
pub mod news;

/// Provider-neutral earnings call transcript.
pub mod earnings_transcript;
pub mod press_release;
/// Recommendation/similar symbol models.
pub mod recommendation;
/// Earnings call transcripts.
pub mod transcript;

// quoteSummary modules (canonical home, re-exported from quote/ for backward compat)
pub mod asset_profile;
pub mod calendar_events;
pub mod earnings;
pub mod earnings_history;
pub mod earnings_trend;
pub mod equity_performance;
pub mod fund_ownership;
pub mod fund_performance;
pub mod fund_profile;
pub mod insider_holders;
pub mod insider_transactions;
pub mod institution_ownership;
pub mod major_holders_breakdown;
pub mod net_share_purchase_activity;
pub mod recommendation_trend;
pub mod sec_filings;
pub mod summary_profile;
pub mod top_holdings;
pub mod upgrade_downgrade_history;

pub use asset_profile::{AssetProfile, CompanyOfficer};
pub use calendar_events::CalendarEvents;
pub use earnings::Earnings;
pub use earnings_history::EarningsHistory;
pub use earnings_trend::EarningsTrend;
pub use equity_performance::EquityPerformance;
pub use fund_ownership::FundOwnership;
pub use fund_performance::FundPerformance;
pub use fund_profile::FundProfile;
pub use insider_holders::InsiderHolders;
pub use insider_transactions::InsiderTransactions;
pub use institution_ownership::InstitutionOwnership;
pub use major_holders_breakdown::MajorHoldersBreakdown;
pub use net_share_purchase_activity::NetSharePurchaseActivity;
pub use recommendation_trend::RecommendationTrend;
pub use sec_filings::SecFilings;
pub use summary_profile::SummaryProfile;
pub use top_holdings::TopHoldings;
pub use upgrade_downgrade_history::UpgradeDowngradeHistory;

#[cfg(all(test, feature = "dataframe"))]
mod dataframe_tests;
