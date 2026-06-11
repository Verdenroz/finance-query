//! Fully local machine-translation backend (feature `translation-offline`).
//!
//! Runs the NLLB-200 distilled 600M model (int8, CTranslate2) on CPU via
//! `ct2rs`. Model files are downloaded from the Hugging Face Hub on first
//! use and cached locally (respects `HF_HOME`); every subsequent run is
//! fully offline. No API key is required.

use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use ct2rs::{Config, TranslationOptions, Translator};

/// `Translator::new` auto-detects the tokenizer from the model directory.
type NllbTranslator = Translator<ct2rs::tokenizers::auto::Tokenizer>;
use hf_hub::api::tokio::ApiBuilder;
use hf_hub::{Cache, Repo, RepoType};
use tokio::sync::OnceCell;

use super::backend::TranslationBackend;
use super::lang::Lang;
use super::split::{join_sentences, split_sentences};
use crate::error::{FinanceError, Result};

/// Default CTranslate2 conversion of NLLB-200 distilled 600M (int8).
const DEFAULT_MODEL_REPO: &str = "JustFrederik/nllb-200-distilled-600M-ct2-int8";

/// Environment variable overriding the Hugging Face model repository
/// (any CTranslate2 NLLB conversion with a `tokenizer.json`).
const MODEL_REPO_ENV: &str = "FINANCE_QUERY_TRANSLATION_MODEL";

const MODEL_FILES: [&str; 4] = [
    "model.bin",
    "shared_vocabulary.txt",
    "tokenizer.json",
    "config.json",
];

/// Offline NLLB translation backend.
pub(crate) struct OfflineBackend {
    cell: OnceCell<Arc<NllbTranslator>>,
}

/// Process-wide shared backend instance (model is loaded once, lazily).
pub(crate) fn shared() -> Arc<OfflineBackend> {
    static SHARED: std::sync::OnceLock<Arc<OfflineBackend>> = std::sync::OnceLock::new();
    SHARED
        .get_or_init(|| {
            Arc::new(OfflineBackend {
                cell: OnceCell::new(),
            })
        })
        .clone()
}

/// Download (if needed) and load the offline translation model.
///
/// Optional warm-up for servers and CLIs: the model is otherwise loaded
/// lazily on the first translated request, which incurs a one-time
/// multi-second download/load delay.
pub async fn preload() -> Result<()> {
    shared().translator().await.map(|_| ())
}

fn translation_error(context: impl std::fmt::Display) -> FinanceError {
    FinanceError::TranslationError {
        context: context.to_string(),
    }
}

async fn download_model() -> Result<PathBuf> {
    let repo_id = std::env::var(MODEL_REPO_ENV).unwrap_or_else(|_| DEFAULT_MODEL_REPO.to_string());
    let hf_repo = Repo::new(repo_id.clone(), RepoType::Model);

    // Fast path: every file already in the local HF cache.
    let cache = Cache::default();
    let cached: Vec<_> = MODEL_FILES
        .iter()
        .filter_map(|f| cache.repo(hf_repo.clone()).get(f))
        .collect();
    if cached.len() == MODEL_FILES.len() {
        return dir_of(&cached[0]);
    }

    tracing::info!(repo = %repo_id, "downloading translation model (one-time setup)");
    let api = ApiBuilder::new()
        .build()
        .map_err(|e| translation_error(format!("hf-hub init failed: {e}")))?;
    let repo = api.repo(hf_repo);
    let mut model_path = None;
    for file in MODEL_FILES {
        let path = repo
            .get(file)
            .await
            .map_err(|e| translation_error(format!("model download failed ({file}): {e}")))?;
        if file == "model.bin" {
            model_path = Some(path);
        }
    }
    dir_of(&model_path.expect("model.bin is in MODEL_FILES"))
}

fn dir_of(file: &std::path::Path) -> Result<PathBuf> {
    file.parent()
        .map(PathBuf::from)
        .ok_or_else(|| translation_error("invalid model cache path"))
}

impl OfflineBackend {
    async fn translator(&self) -> Result<Arc<NllbTranslator>> {
        self.cell
            .get_or_try_init(|| async {
                let model_dir = download_model().await?;
                let config = Config {
                    // Cap intra-op threads: NLLB gains little beyond 8 and
                    // this avoids starving the async runtime on big hosts.
                    num_threads_per_replica: std::thread::available_parallelism()
                        .map(|n| n.get().min(8))
                        .unwrap_or(4),
                    ..Config::default()
                };
                let translator =
                    tokio::task::spawn_blocking(move || Translator::new(&model_dir, &config))
                        .await
                        .map_err(|e| translation_error(format!("model load task failed: {e}")))?
                        .map_err(|e| translation_error(format!("model load failed: {e}")))?;
                tracing::info!("offline translation model loaded");
                Ok(Arc::new(translator))
            })
            .await
            .cloned()
    }
}

#[async_trait]
impl TranslationBackend for OfflineBackend {
    fn id(&self) -> &'static str {
        "nllb-offline"
    }

    async fn translate_batch(&self, texts: &[String], target: &Lang) -> Result<Vec<String>> {
        let code = target
            .nllb_code()
            .ok_or_else(|| FinanceError::InvalidParameter {
                param: "lang".into(),
                reason: format!(
                    "language '{}' is not supported by the offline translation model",
                    target.code()
                ),
            })?;
        if texts.is_empty() {
            return Ok(Vec::new());
        }
        let translator = self.translator().await?;

        // Flatten all texts into sentence-level segments for one model call.
        let mut segments: Vec<String> = Vec::new();
        let mut spans: Vec<(usize, usize)> = Vec::with_capacity(texts.len());
        for text in texts {
            let sentences = split_sentences(text);
            let start = segments.len();
            segments.extend(sentences);
            spans.push((start, segments.len()));
        }

        let prefixes = vec![vec![code.to_string()]; segments.len()];
        let results = tokio::task::spawn_blocking(move || {
            let options = TranslationOptions {
                beam_size: 1,
                disable_unk: true,
                ..Default::default()
            };
            translator.translate_batch_with_target_prefix(&segments, &prefixes, &options, None)
        })
        .await
        .map_err(|e| translation_error(format!("translation task failed: {e}")))?
        .map_err(|e| translation_error(format!("translation failed: {e}")))?;

        let translated_segments: Vec<String> = results.into_iter().map(|(text, _)| text).collect();
        let without_space = target.joins_without_space();
        Ok(spans
            .into_iter()
            .map(|(start, end)| join_sentences(&translated_segments[start..end], without_space))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// End-to-end model download + inference. Excluded from default runs:
    /// downloads ~600 MB on first use.
    #[tokio::test]
    #[ignore = "requires network access"]
    async fn offline_backend_translates() {
        let backend = shared();
        let lang = Lang::parse("de").unwrap();
        let texts = vec!["The company designs smartphones.".to_string()];
        let out = backend.translate_batch(&texts, &lang).await.unwrap();
        assert_eq!(out.len(), 1);
        assert_ne!(out[0], texts[0]);
        assert!(!out[0].contains("<unk>"));
    }

    #[tokio::test]
    async fn unsupported_language_errors() {
        let backend = shared();
        let lang = Lang::parse("tlh").unwrap();
        let texts = vec!["Hello".to_string()];
        assert!(backend.translate_batch(&texts, &lang).await.is_err());
    }
}
