/// Typed exchange codes for screener queries.
///
/// Use with [`EquityField::Exchange`](crate::EquityField::Exchange) or
/// [`FundField::Exchange`](crate::FundField::Exchange) and
/// [`ScreenerFieldExt::eq_str`](crate::ScreenerFieldExt::eq_str).
///
/// For mutual fund queries use [`ExchangeCode::Nas`].
///
/// # Example
///
/// ```
/// use finance_query::{EquityField, EquityScreenerQuery, ScreenerFieldExt, ExchangeCode};
///
/// let query = EquityScreenerQuery::new()
///     .add_condition(EquityField::Exchange.eq_str(ExchangeCode::Nms));
/// ```
pub mod exchange_codes;

/// Fundamental timeseries field types for financial statements
///
/// These constants represent field names that must be prefixed with frequency ("annual" or "quarterly")
/// Example: "annualTotalRevenue", "quarterlyTotalRevenue"
#[allow(missing_docs)]
pub mod fundamental_types;

/// World market indices
pub mod indices;

/// Typed industry identifiers shared between the industry endpoint and screener queries.
///
/// Use with [`finance::industry()`](crate::finance::industry) for the data endpoint, and with
/// [`EquityField::Industry`](crate::EquityField::Industry) +
/// [`ScreenerFieldExt::eq_str`](crate::ScreenerFieldExt::eq_str) for screener filtering.
///
/// - `as_slug()` → lowercase hyphenated key for `finance::industry()` (e.g. `"semiconductors"`)
/// - `From<Industry> for String` → screener display name (e.g. `"Semiconductors"`)
///
/// # Example
///
/// ```no_run
/// use finance_query::{finance, Industry, EquityField, EquityScreenerQuery, ScreenerFieldExt};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Industry endpoint
/// let data = finance::industry(Industry::Semiconductors).await?;
///
/// // Screener filter
/// let query = EquityScreenerQuery::new()
///     .add_condition(EquityField::Industry.eq_str(Industry::Semiconductors));
/// # Ok(())
/// # }
/// ```
pub mod industries;

/// Predefined screener selectors for Yahoo Finance
pub mod screeners;

/// Yahoo Finance sector types
///
/// These are the 11 GICS sectors available on Yahoo Finance.
pub mod sectors;

mod enums;

pub use enums::{Frequency, Interval, Region, StatementType, TimeRange, ValueFormat};
