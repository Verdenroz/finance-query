//! Pluggable machine-translation backend.

use std::sync::{Arc, OnceLock, RwLock};

use async_trait::async_trait;

use super::lang::Lang;
use crate::error::Result;

/// A machine-translation backend for free-form text.
///
/// Implement this to plug a custom engine (e.g. a hosted translation API)
/// into the translation pipeline via [`set_backend`](super::set_backend).
/// The built-in offline backend (feature `translation-offline`) is used by
/// default when no custom backend is registered.
///
/// Inputs are English; implementations must return one translated string per
/// input, preserving order. Inputs may contain multiple sentences.
#[async_trait]
pub trait TranslationBackend: Send + Sync {
    /// Identifier used in logs and errors.
    fn id(&self) -> &'static str {
        "custom"
    }

    /// Translate every English text into the target language, preserving order.
    async fn translate_batch(&self, texts: &[String], target: &Lang) -> Result<Vec<String>>;
}

fn registry() -> &'static RwLock<Option<Arc<dyn TranslationBackend>>> {
    static REGISTRY: OnceLock<RwLock<Option<Arc<dyn TranslationBackend>>>> = OnceLock::new();
    REGISTRY.get_or_init(|| RwLock::new(None))
}

/// Register a custom translation backend for the whole process.
///
/// Takes precedence over the built-in offline backend. Free-form text fields
/// are left untranslated when no backend is available (built-in dictionary
/// terms are still translated).
pub fn set_backend(backend: Arc<dyn TranslationBackend>) {
    if let Ok(mut slot) = registry().write() {
        *slot = Some(backend);
    }
}

/// Resolve the active backend: custom first, then the built-in offline
/// backend when the `translation-offline` feature is enabled.
pub(crate) fn active_backend() -> Option<Arc<dyn TranslationBackend>> {
    if let Ok(slot) = registry().read()
        && let Some(backend) = slot.as_ref()
    {
        return Some(backend.clone());
    }
    #[cfg(feature = "translation-offline")]
    {
        return Some(super::offline::shared());
    }
    #[cfg(not(feature = "translation-offline"))]
    None
}
