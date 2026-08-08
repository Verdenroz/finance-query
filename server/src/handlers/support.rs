//! Shared REST helpers used across multiple `handlers/*` domain modules:
//! query-param parsing.

use finance_query::ValueFormat;

/// Resolve an omitted (or unrecognized) `format` query parameter to the default.
///
/// The lenient string parsing now lives in `params::lenient_value_format`, which
/// is where an unrecognized value becomes `None` rather than a rejection.
pub(crate) fn parse_format(f: Option<ValueFormat>) -> ValueFormat {
    f.unwrap_or_default()
}
