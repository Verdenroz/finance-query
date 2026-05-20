//! Canonical intermediate types for multi-provider data normalization.
//!
//! These are `pub(crate)` types that each [`Provider`](super::Provider)
//! implementation populates from its native format. They are consumed
//! by conversion methods on public-facing types in [`crate::models`].

use crate::Provider;

// ── Recommendations ───────────────────────────────────────────────

pub(crate) fn recommendation_from_similar(
    symbol: impl Into<String>,
    provider_id: Option<Provider>,
    items: Vec<crate::models::corporate::recommendation::SimilarSymbol>,
    limit: Option<u32>,
) -> crate::models::corporate::recommendation::Recommendation {
    let recommendations = if let Some(limit) = limit {
        items.into_iter().take(limit as usize).collect()
    } else {
        items
    };

    crate::models::corporate::recommendation::Recommendation {
        symbol: symbol.into(),
        recommendations,
        provider_id,
    }
}
