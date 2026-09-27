//! Resumable provider pages. Cursors contain no credentials.

use crate::{FinanceError, Provider, Result};
use serde::{Deserialize, Serialize};

/// One provider response, without collecting subsequent pages.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ProviderPage<T> {
    /// Records returned on this page.
    pub items: Vec<T>,
    /// Persist with the records before requesting the next page.
    pub next: Option<PageCursor>,
    /// Provider that owns this page and its continuation.
    pub provider_id: Provider,
    /// Provider request identifier.
    pub request_id: Option<String>,
    /// Provider-reported result count, when supplied.
    pub results_count: Option<usize>,
    /// Provider query count; this need not equal the number of bars.
    pub query_count: Option<usize>,
    /// Symbol reported by the response.
    pub reported_symbol: Option<String>,
    /// Adjustment reported by the response.
    pub adjusted: Option<bool>,
}

/// A serializable continuation bound to the original operation and request.
///
/// Treat persisted cursors as opaque values. Validation runs again when used.
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PageCursor {
    pub(crate) version: u8,
    pub(crate) provider: Provider,
    pub(crate) operation: String,
    pub(crate) request: String,
    pub(crate) target: String,
}

impl std::fmt::Debug for PageCursor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PageCursor")
            .field("version", &self.version)
            .field("provider", &self.provider)
            .finish_non_exhaustive()
    }
}

impl PageCursor {
    /// Provider to which this continuation belongs.
    pub fn provider(&self) -> Provider {
        self.provider
    }

    pub(crate) fn validate(&self, operation: &str, request: &str) -> Result<()> {
        if self.version != 1 || self.operation != operation || self.request != request {
            return Err(FinanceError::InvalidParameter {
                param: "cursor".into(),
                reason: "cursor version, operation or request does not match".into(),
            });
        }
        Ok(())
    }
}
