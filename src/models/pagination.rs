//! Resumable provider pages. Cursors contain no credentials.

use crate::{FinanceError, Operation, Provider, Result};

/// Bump whenever a request's cursor identity changes meaning, so older cursors are rejected.
const CURSOR_VERSION: u8 = 1;
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

impl<T> ProviderPage<T> {
    /// A page from `provider_id`, with no response metadata. `next` is `None` on
    /// the last page; otherwise build it with [`PageCursor::continuation`].
    pub fn new(items: Vec<T>, provider_id: Provider, next: Option<PageCursor>) -> Self {
        Self {
            items,
            next,
            provider_id,
            request_id: None,
            results_count: None,
            query_count: None,
            reported_symbol: None,
            adjusted: None,
        }
    }
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

    /// A continuation for an adapter to return in [`ProviderPage::next`].
    ///
    /// `provider` must be the adapter's own id and `target` is whatever state the
    /// adapter needs to fetch the next page. The routing layer binds the cursor to
    /// the operation and request, and checks both before handing it back.
    pub fn continuation(provider: Provider, target: String) -> Self {
        Self {
            version: CURSOR_VERSION,
            provider,
            operation: String::new(),
            request: String::new(),
            target,
        }
    }

    /// The adapter state passed to [`continuation`](Self::continuation).
    pub fn target(&self) -> &str {
        &self.target
    }

    pub(crate) fn bind(&mut self, operation: Operation, request: &str) {
        self.operation = operation.as_str().into();
        self.request = request.into();
    }

    pub(crate) fn validate(&self, operation: Operation, request: &str) -> Result<()> {
        if self.version != CURSOR_VERSION
            || self.operation != operation.as_str()
            || self.request != request
        {
            return Err(FinanceError::InvalidParameter {
                param: "cursor".into(),
                reason: "cursor version, operation or request does not match".into(),
            });
        }
        Ok(())
    }
}
