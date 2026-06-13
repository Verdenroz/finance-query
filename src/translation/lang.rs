//! BCP 47 language tag parsing and mapping to translation targets.

use crate::error::{FinanceError, Result};

/// A parsed, normalized target language for translation.
///
/// Accepts BCP 47 language tags (e.g. `"ja"`, `"de-DE"`, `"zh-Hans"`, `"pt_BR"`).
/// Underscores are accepted as subtag separators for convenience.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct Lang {
    primary: String,
    script: Option<String>,
    region: Option<String>,
}

impl Lang {
    /// Parse a BCP 47 language tag into a normalized [`Lang`].
    ///
    /// Returns an error if the tag is structurally invalid (the primary
    /// subtag must be 2–3 ASCII letters).
    pub fn parse(tag: &str) -> Result<Self> {
        let mut primary = None;
        let mut script = None;
        let mut region = None;
        for (i, part) in tag.split(['-', '_']).enumerate() {
            if i == 0 {
                if !(2..=3).contains(&part.len()) || !part.chars().all(|c| c.is_ascii_alphabetic())
                {
                    break;
                }
                primary = Some(part.to_ascii_lowercase());
            } else if part.len() == 4 && part.chars().all(|c| c.is_ascii_alphabetic()) {
                let mut s = part.to_ascii_lowercase();
                s[..1].make_ascii_uppercase();
                script = Some(s);
            } else if part.len() == 2 && part.chars().all(|c| c.is_ascii_alphabetic()) {
                region = Some(part.to_ascii_uppercase());
            }
        }
        let primary = primary.ok_or_else(|| FinanceError::InvalidParameter {
            param: "lang".into(),
            reason: format!("'{tag}' is not a valid BCP 47 language tag"),
        })?;
        Ok(Self {
            primary,
            script,
            region,
        })
    }

    /// The lowercase primary language subtag (e.g. `"ja"`).
    pub fn primary(&self) -> &str {
        &self.primary
    }

    /// The normalized BCP 47 tag (primary + script when relevant), used as a cache key.
    pub fn code(&self) -> String {
        match self.effective_script() {
            Some(script) => format!("{}-{}", self.primary, script),
            None => self.primary.clone(),
        }
    }

    /// True if the target language is English (translation is a no-op).
    pub fn is_english(&self) -> bool {
        self.primary == "en"
    }

    /// Script subtag, defaulted for Chinese from the region when absent
    /// (TW/HK/MO imply Traditional script).
    fn effective_script(&self) -> Option<&str> {
        if let Some(s) = &self.script {
            return Some(s.as_str());
        }
        if self.primary == "zh" {
            return match self.region.as_deref() {
                Some("TW") | Some("HK") | Some("MO") => Some("Hant"),
                _ => Some("Hans"),
            };
        }
        None
    }

    /// True for languages written without spaces between words, so translated
    /// sentence segments are rejoined without a separator.
    #[cfg(any(feature = "translation-offline", test))]
    pub(crate) fn joins_without_space(&self) -> bool {
        matches!(self.primary.as_str(), "ja" | "zh" | "th" | "km" | "lo")
    }

    /// The NLLB / FLORES-200 language code for the built-in offline backend.
    ///
    /// Returns `None` for languages the bundled model does not cover.
    pub fn nllb_code(&self) -> Option<&'static str> {
        let code = match (self.primary.as_str(), self.effective_script()) {
            ("ar", _) => "arb_Arab",
            ("bg", _) => "bul_Cyrl",
            ("bn", _) => "ben_Beng",
            ("ca", _) => "cat_Latn",
            ("cs", _) => "ces_Latn",
            ("da", _) => "dan_Latn",
            ("de", _) => "deu_Latn",
            ("el", _) => "ell_Grek",
            ("en", _) => "eng_Latn",
            ("es", _) => "spa_Latn",
            ("et", _) => "est_Latn",
            ("fa", _) => "pes_Arab",
            ("fi", _) => "fin_Latn",
            ("fr", _) => "fra_Latn",
            ("he", _) | ("iw", _) => "heb_Hebr",
            ("hi", _) => "hin_Deva",
            ("hr", _) => "hrv_Latn",
            ("hu", _) => "hun_Latn",
            ("id", _) | ("in", _) => "ind_Latn",
            ("it", _) => "ita_Latn",
            ("ja", _) => "jpn_Jpan",
            ("ko", _) => "kor_Hang",
            ("lt", _) => "lit_Latn",
            ("lv", _) => "lvs_Latn",
            ("ms", _) => "zsm_Latn",
            ("nb", _) | ("no", _) => "nob_Latn",
            ("nl", _) => "nld_Latn",
            ("nn", _) => "nno_Latn",
            ("pl", _) => "pol_Latn",
            ("pt", _) => "por_Latn",
            ("ro", _) => "ron_Latn",
            ("ru", _) => "rus_Cyrl",
            ("sk", _) => "slk_Latn",
            ("sl", _) => "slv_Latn",
            ("sv", _) => "swe_Latn",
            ("sw", _) => "swh_Latn",
            ("ta", _) => "tam_Taml",
            ("th", _) => "tha_Thai",
            ("tl", _) => "tgl_Latn",
            ("tr", _) => "tur_Latn",
            ("uk", _) => "ukr_Cyrl",
            ("ur", _) => "urd_Arab",
            ("vi", _) => "vie_Latn",
            ("zh", Some("Hant")) => "zho_Hant",
            ("zh", _) => "zho_Hans",
            _ => return None,
        };
        Some(code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_tags() {
        let l = Lang::parse("ja").unwrap();
        assert_eq!(l.primary(), "ja");
        assert_eq!(l.code(), "ja");
        assert!(!l.is_english());
    }

    #[test]
    fn parses_region_and_underscore() {
        assert_eq!(Lang::parse("de-DE").unwrap().code(), "de");
        assert_eq!(Lang::parse("pt_BR").unwrap().code(), "pt");
        assert_eq!(Lang::parse("ja-JP").unwrap().nllb_code(), Some("jpn_Jpan"));
    }

    #[test]
    fn english_variants_are_english() {
        for tag in ["en", "en-US", "en-GB", "en_AU"] {
            assert!(Lang::parse(tag).unwrap().is_english());
        }
    }

    #[test]
    fn chinese_script_resolution() {
        assert_eq!(Lang::parse("zh").unwrap().code(), "zh-Hans");
        assert_eq!(Lang::parse("zh-CN").unwrap().nllb_code(), Some("zho_Hans"));
        assert_eq!(Lang::parse("zh-TW").unwrap().nllb_code(), Some("zho_Hant"));
        assert_eq!(Lang::parse("zh-HK").unwrap().nllb_code(), Some("zho_Hant"));
        assert_eq!(
            Lang::parse("zh-Hant-HK").unwrap().nllb_code(),
            Some("zho_Hant")
        );
        assert_eq!(Lang::parse("zh-Hans").unwrap().code(), "zh-Hans");
    }

    #[test]
    fn invalid_tags_error() {
        assert!(Lang::parse("").is_err());
        assert!(Lang::parse("x").is_err());
        assert!(Lang::parse("1234").is_err());
    }

    #[test]
    fn unknown_language_has_no_nllb_code() {
        assert_eq!(Lang::parse("tlh").unwrap().nllb_code(), None);
    }

    #[test]
    fn cjk_join_detection() {
        assert!(Lang::parse("ja").unwrap().joins_without_space());
        assert!(Lang::parse("zh-TW").unwrap().joins_without_space());
        assert!(!Lang::parse("ko").unwrap().joins_without_space());
        assert!(!Lang::parse("de").unwrap().joins_without_space());
    }
}
