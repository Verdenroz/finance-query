//! Historical stock-directory requests.

use crate::{FinanceError, Result};
pub(crate) use finance_query_adapters_core::stock_listing::date;
use serde::Serialize;

/// A stock directory snapshot; independent of free-text search.
#[derive(Debug, Clone, Serialize)]
pub struct StockListingRequest {
    pub(crate) date: String,
    pub(crate) active: bool,
    pub(crate) stock_type: Option<String>,
    pub(crate) locale: String,
    pub(crate) limit: u32,
}

impl StockListingRequest {
    /// Request active or inactive stocks on an inclusive calendar date.
    pub fn new(as_of: &str, active: bool) -> Result<Self> {
        date(as_of)?;
        Ok(Self {
            date: as_of.into(),
            active,
            stock_type: None,
            locale: "us".into(),
            limit: 1000,
        })
    }

    /// Filter by a provider stock-type code, such as CS or ADRC.
    pub fn with_stock_type(mut self, code: impl Into<String>) -> Self {
        self.stock_type = Some(code.into());
        self
    }

    /// Select a market locale. Defaults to us.
    pub fn with_locale(mut self, locale: impl Into<String>) -> Self {
        self.locale = locale.into();
        self
    }

    /// Snapshot date (`YYYY-MM-DD`).
    pub fn as_of(&self) -> &str {
        &self.date
    }

    /// Whether active or inactive stocks are requested.
    pub fn active(&self) -> bool {
        self.active
    }

    /// Stock-type filter, if any.
    pub fn stock_type(&self) -> Option<&str> {
        self.stock_type.as_deref()
    }

    /// Requested market locale.
    pub fn locale(&self) -> &str {
        &self.locale
    }

    /// Maximum entries per provider page.
    pub fn page_size(&self) -> u32 {
        self.limit
    }

    /// The fields a continuation must match. Changing them requires a new cursor version.
    pub(crate) fn cursor_identity(&self) -> String {
        serde_json::json!([
            self.date,
            self.active,
            self.stock_type,
            self.locale,
            self.limit
        ])
        .to_string()
    }

    /// Limit a page to 1–1000 directory entries.
    pub fn with_page_size(mut self, limit: u32) -> Result<Self> {
        if !(1..=1000).contains(&limit) {
            return Err(FinanceError::InvalidParameter {
                param: "page_size".into(),
                reason: "expected 1..=1000".into(),
            });
        }
        self.limit = limit;
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_identity_is_pinned_to_the_query_fields() {
        let request = StockListingRequest::new("2020-01-02", false)
            .unwrap()
            .with_stock_type("CS")
            .with_page_size(2)
            .unwrap();
        assert_eq!(
            request.cursor_identity(),
            r#"["2020-01-02",false,"CS","us",2]"#
        );
    }
}
