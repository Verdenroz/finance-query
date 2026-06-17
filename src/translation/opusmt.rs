//! Fully local machine-translation backend (feature `translation-offline`).
//!
//! Runs **opus-mt bilingual** models (one English→target model per language)
//! on CPU via CTranslate2 (`ct2rs`) with int8 weights. Models are distributed
//! as Argos Translate packages (the engine behind LibreTranslate): a small
//! (~80–210 MB) zip bundling a CTranslate2 model directory and a shared
//! SentencePiece tokenizer. The required language's package is downloaded from
//! the Argos package server on first use and cached locally; every subsequent
//! run is fully offline. No API key is required.
//!
//! Bilingual models are an order of magnitude smaller than a single
//! multilingual model, so on a constrained CPU VM they translate a full
//! earnings-call transcript in a few seconds (many times faster) and give
//! far tighter cross-language latency parity, because each language runs its
//! own similarly sized model rather than one shared decoder whose output token
//! count varies widely by language.

use std::collections::HashMap;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock, RwLock};

use async_trait::async_trait;
use ct2rs::tokenizers::sentencepiece::Tokenizer as SpmTokenizer;
use ct2rs::{BatchType, ComputeType, Config, TranslationOptions, Translator};
use tokio::sync::OnceCell;

use super::backend::TranslationBackend;
use super::lang::Lang;
use super::split::{join_sentences, split_sentences};
use crate::error::{FinanceError, Result};

type OpusTranslator = Translator<SpmTokenizer>;

/// Argos package server (version-pinned package stems below).
const ARGOS_BASE_URL: &str = "https://argos-net.com/v1";

/// Environment variable overriding the model cache root (defaults to
/// `$HF_HOME` then `$HOME/.cache/huggingface`, reusing the same persistent
/// volume as other cached model data).
const CACHE_ENV: &str = "FINANCE_QUERY_TRANSLATION_CACHE";

/// Intra-op threads per model (env override). Defaults to `min(cores, 8)`.
const THREADS_ENV: &str = "FINANCE_QUERY_TRANSLATION_THREADS";

/// Languages to warm during [`preload`] (comma-separated primary subtags).
const PRELOAD_ENV: &str = "FINANCE_QUERY_TRANSLATION_PRELOAD";

/// CTranslate2 model config written into each extracted model directory.
///
/// Argos opus-mt packages ship no `config.json`; CTranslate2 needs one to
/// resolve special tokens. All opus-mt models share the Marian convention
/// (`</s>` is both EOS and the decoder start; the SentencePiece tokenizer
/// appends source EOS itself, so `add_source_eos` is false). No `model_type`
/// is set — the architecture lives in `model.bin`, and forcing `marian` would
/// make CTranslate2 demand dual source/target vocabularies (Argos ships one
/// `shared_vocabulary.txt`).
const MARIAN_CONFIG_JSON: &str = r#"{
  "add_source_bos": false,
  "add_source_eos": false,
  "bos_token": "<s>",
  "decoder_start_token": "</s>",
  "eos_token": "</s>",
  "unk_token": "<unk>"
}
"#;

/// English → target-language opus-mt packages (Argos index, version-pinned).
/// Keyed by the primary BCP 47 subtag; `zh` is Simplified, `zt` Traditional.
const PACKAGES: &[(&str, &str)] = &[
    ("ar", "translate-en_ar-1_0"),
    ("az", "translate-en_az-1_5"),
    ("bg", "translate-en_bg-1_9"),
    ("bn", "translate-en_bn-1_9"),
    ("ca", "translate-en_ca-1_9"),
    ("cs", "translate-en_cs-1_9_6"),
    ("da", "translate-en_da-1_9"),
    ("de", "translate-en_de-1_3"),
    ("el", "translate-en_el-1_9"),
    ("eo", "translate-en_eo-1_5"),
    ("es", "translate-en_es-1_0"),
    ("et", "translate-en_et-1_9"),
    ("eu", "translate-en_eu-1_9"),
    ("fa", "translate-en_fa-1_5"),
    ("fi", "translate-en_fi-1_9"),
    ("fr", "translate-en_fr-1_9"),
    ("ga", "translate-en_ga-1_1"),
    ("gl", "translate-en_gl-1_9"),
    ("he", "translate-en_he-1_5"),
    ("hi", "translate-en_hi-1_1"),
    ("hu", "translate-en_hu-1_9"),
    ("id", "translate-en_id-1_9"),
    ("it", "translate-en_it-1_0"),
    ("ja", "translate-en_ja-1_1"),
    ("ko", "translate-en_ko-1_1"),
    ("ky", "translate-en_ky-1_4"),
    ("lt", "translate-en_lt-1_9"),
    ("lv", "translate-en_lv-1_9"),
    ("ms", "translate-en_ms-1_9"),
    ("nb", "translate-en_nb-1_9"),
    ("nl", "translate-en_nl-1_8"),
    ("pl", "translate-en_pl-1_9"),
    ("pt", "translate-en_pt-1_9"),
    ("ro", "translate-en_ro-1_9"),
    ("ru", "translate-en_ru-1_9"),
    ("sk", "translate-en_sk-1_9"),
    ("sl", "translate-en_sl-1_9"),
    ("sq", "translate-en_sq-1_9"),
    ("sv", "translate-en_sv-1_5"),
    ("th", "translate-en_th-1_9"),
    ("tl", "translate-en_tl-1_9"),
    ("tr", "translate-en_tr-1_5"),
    ("uk", "translate-en_uk-1_4"),
    ("ur", "translate-en_ur-1_9"),
    ("vi", "translate-en_vi-1_9"),
    ("zh", "translate-en_zh-1_9"),
    ("zt", "translate-en_zt-1_9"),
];

/// Resolve the opus-mt package stem for a target language, honouring the
/// Traditional/Simplified Chinese distinction.
fn package_for(lang: &Lang) -> Option<&'static str> {
    let key = match (lang.primary(), lang.code().as_str()) {
        ("zh", "zh-Hant") => "zt",
        ("zh", _) => "zh",
        (primary, _) => primary,
    };
    PACKAGES
        .iter()
        .find_map(|(code, stem)| (*code == key).then_some(*stem))
}

fn translation_error(context: impl std::fmt::Display) -> FinanceError {
    FinanceError::TranslationError {
        context: context.to_string(),
    }
}

fn threads() -> usize {
    std::env::var(THREADS_ENV)
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map(|n| n.get().min(8))
                .unwrap_or(4)
        })
        .max(1)
}

fn cache_root() -> PathBuf {
    if let Ok(dir) = std::env::var(CACHE_ENV) {
        return PathBuf::from(dir);
    }
    if let Ok(hf) = std::env::var("HF_HOME") {
        return PathBuf::from(hf).join("argos");
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".cache/huggingface/argos")
}

/// Per-language offline opus-mt backend.
pub(crate) struct OpusMtBackend {
    /// One lazily-initialised translator per target language. The inner
    /// `OnceCell` deduplicates concurrent first-use loads of the same language.
    models: RwLock<HashMap<String, Arc<OnceCell<Arc<OpusTranslator>>>>>,
}

/// Process-wide shared backend instance.
pub(crate) fn shared() -> Arc<OpusMtBackend> {
    static SHARED: OnceLock<Arc<OpusMtBackend>> = OnceLock::new();
    SHARED
        .get_or_init(|| {
            Arc::new(OpusMtBackend {
                models: RwLock::new(HashMap::new()),
            })
        })
        .clone()
}

/// Download (if needed) and warm a set of language models ahead of first use.
///
/// Languages come from `FINANCE_QUERY_TRANSLATION_PRELOAD` (comma-separated
/// primary subtags, e.g. `"es,ja,de"`); unset is a no-op, since each model is
/// otherwise fetched lazily on the first request for that language.
pub async fn preload() -> Result<()> {
    let Ok(list) = std::env::var(PRELOAD_ENV) else {
        return Ok(());
    };
    let backend = shared();
    for code in list.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        let lang = Lang::parse(code)?;
        backend.translator(&lang).await?;
    }
    Ok(())
}

/// Extract the CTranslate2 model directory and SentencePiece model from a
/// downloaded `.argosmodel` zip into `dest`, returning `(model_dir, spm_path)`.
fn extract_package(bytes: &[u8], dest: &Path) -> Result<(PathBuf, PathBuf)> {
    let model_dir = dest.join("model");
    let spm_path = dest.join("sentencepiece.model");
    std::fs::create_dir_all(&model_dir).map_err(translation_error)?;

    // Keep whatever each package vintage ships (reconciled below): older ones use
    // a `.txt` vocab and no config.json; newer ones bundle the `.json` forms.
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).map_err(translation_error)?;
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i).map_err(translation_error)?;
        let name = entry.name().replace('\\', "/");
        let target = if name.ends_with("/sentencepiece.model") {
            spm_path.clone()
        } else if let Some(file) = name
            .rsplit_once("/model/")
            .map(|(_, f)| f)
            .filter(|f| !f.is_empty() && !f.contains('/'))
        {
            model_dir.join(file)
        } else {
            continue;
        };
        let mut out = std::fs::File::create(&target).map_err(translation_error)?;
        std::io::copy(&mut entry, &mut out).map_err(translation_error)?;
    }

    if !model_dir.join("model.bin").exists() || !spm_path.exists() {
        return Err(translation_error(
            "argos package missing model.bin or sentencepiece.model",
        ));
    }

    // ct2rs needs an explicit source/target vocab pair, so split a lone
    // `shared_vocabulary.txt`; the newer `.json` form loads directly.
    let shared_txt = model_dir.join("shared_vocabulary.txt");
    if shared_txt.exists() && !model_dir.join("shared_vocabulary.json").exists() {
        std::fs::copy(&shared_txt, model_dir.join("source_vocabulary.txt"))
            .map_err(translation_error)?;
        std::fs::copy(&shared_txt, model_dir.join("target_vocabulary.txt"))
            .map_err(translation_error)?;
    }

    // Older packages omit config.json; CTranslate2 needs one. Newer packages
    // ship their own (with the correct decoder-start token) — keep it.
    let config = model_dir.join("config.json");
    if !config.exists() {
        std::fs::write(&config, MARIAN_CONFIG_JSON).map_err(translation_error)?;
    }
    Ok((model_dir, spm_path))
}

/// Download + extract (if not cached) and load the translator for `stem`.
fn load_translator(primary: &str, stem: &'static str) -> Result<OpusTranslator> {
    let dest = cache_root().join(primary);
    let model_dir = dest.join("model");
    let spm_path = dest.join("sentencepiece.model");

    if !(model_dir.join("model.bin").exists() && spm_path.exists()) {
        let url = format!("{ARGOS_BASE_URL}/{stem}.argosmodel");
        tracing::info!(%url, "downloading opus-mt model (one-time setup)");
        let bytes = reqwest::blocking::get(&url)
            .and_then(reqwest::blocking::Response::error_for_status)
            .and_then(|r| r.bytes())
            .map_err(|e| translation_error(format!("model download failed ({stem}): {e}")))?;
        extract_package(&bytes, &dest)?;
    }

    // Pin int8 (Argos weights are int8) so every language shares one dnnl GEMM
    // path — uniform precision keeps the cross-language latency spread tight.
    let config = Config {
        compute_type: ComputeType::INT8,
        num_threads_per_replica: threads(),
        ..Config::default()
    };
    let tokenizer = SpmTokenizer::from_file(&spm_path, &spm_path)
        .map_err(|e| translation_error(format!("tokenizer load failed: {e}")))?;
    Translator::with_tokenizer(&model_dir, tokenizer, &config)
        .map_err(|e| translation_error(format!("model load failed: {e}")))
}

impl OpusMtBackend {
    async fn translator(&self, target: &Lang) -> Result<Arc<OpusTranslator>> {
        let stem = package_for(target).ok_or_else(|| FinanceError::InvalidParameter {
            param: "lang".into(),
            reason: format!(
                "language '{}' is not supported by the offline translation model",
                target.code()
            ),
        })?;
        let primary = target.primary().to_string();

        let cell = {
            let mut models = self
                .models
                .write()
                .map_err(|_| translation_error("translation model cache poisoned"))?;
            models.entry(primary.clone()).or_default().clone()
        };

        cell.get_or_try_init(|| async {
            let primary = primary.clone();
            let translator = tokio::task::spawn_blocking(move || load_translator(&primary, stem))
                .await
                .map_err(|e| translation_error(format!("model load task failed: {e}")))??;
            tracing::info!(lang = %target.code(), "opus-mt model loaded");
            Ok(Arc::new(translator))
        })
        .await
        .cloned()
    }
}

#[async_trait]
impl TranslationBackend for OpusMtBackend {
    fn id(&self) -> &'static str {
        "opus-mt-offline"
    }

    /// True when an opus-mt package exists for the target language; the ~48
    /// supported languages cover the major world languages, and unsupported
    /// ones degrade to English free-form text (dictionary terms still resolve).
    fn supports(&self, target: &Lang) -> bool {
        package_for(target).is_some()
    }

    async fn translate_batch(&self, texts: &[String], target: &Lang) -> Result<Vec<String>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }
        let translator = self.translator(target).await?;

        // Flatten every text into sentence-level segments for one model call;
        // opus-mt models are trained on (and capped to) single sentences.
        let mut segments: Vec<String> = Vec::new();
        let mut spans: Vec<(usize, usize)> = Vec::with_capacity(texts.len());
        for text in texts {
            let start = segments.len();
            segments.extend(split_sentences(text));
            spans.push((start, segments.len()));
        }

        let results = tokio::task::spawn_blocking(move || {
            // Length-sort + token-batch so latency tracks real tokens, not
            // padding to the longest sentence — the cross-language spread driver.
            let options = TranslationOptions {
                beam_size: 1,
                disable_unk: true,
                max_batch_size: 2048,
                batch_type: BatchType::Tokens,
                ..Default::default()
            };
            translator
                .translate_batch(&segments, &options, None)
                .map(|out| out.into_iter().map(|(text, _)| text).collect::<Vec<_>>())
        })
        .await
        .map_err(|e| translation_error(format!("translation task failed: {e}")))?
        .map_err(|e| translation_error(format!("translation failed: {e}")))?;

        let without_space = target.joins_without_space();
        Ok(spans
            .into_iter()
            .map(|(start, end)| join_sentences(&results[start..end], without_space))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_lookup_resolves_major_languages() {
        assert_eq!(
            package_for(&Lang::parse("es").unwrap()),
            Some("translate-en_es-1_0")
        );
        assert_eq!(
            package_for(&Lang::parse("ja-JP").unwrap()),
            Some("translate-en_ja-1_1")
        );
        // Simplified vs Traditional Chinese map to distinct packages.
        assert_eq!(
            package_for(&Lang::parse("zh-CN").unwrap()),
            Some("translate-en_zh-1_9")
        );
        assert_eq!(
            package_for(&Lang::parse("zh-TW").unwrap()),
            Some("translate-en_zt-1_9")
        );
        // Unsupported language has no package.
        assert_eq!(package_for(&Lang::parse("tlh").unwrap()), None);
    }

    #[tokio::test]
    async fn unsupported_language_errors() {
        let backend = shared();
        let lang = Lang::parse("tlh").unwrap();
        let texts = vec!["Hello".to_string()];
        assert!(backend.translate_batch(&texts, &lang).await.is_err());
    }

    /// End-to-end model download + inference. Excluded from default runs:
    /// downloads a model on first use.
    #[tokio::test]
    #[ignore = "requires network access"]
    async fn opusmt_backend_translates() {
        let backend = shared();
        let lang = Lang::parse("es").unwrap();
        let texts = vec!["The company designs smartphones.".to_string()];
        let out = backend.translate_batch(&texts, &lang).await.unwrap();
        assert_eq!(out.len(), 1);
        assert_ne!(out[0], texts[0]);
        assert!(!out[0].contains("<unk>"));
    }
}
