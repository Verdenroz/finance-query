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
        assert_eq!(Lang::parse("ja-JP").unwrap().primary(), "ja");
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
        assert_eq!(Lang::parse("zh-CN").unwrap().code(), "zh-Hans");
        assert_eq!(Lang::parse("zh-TW").unwrap().code(), "zh-Hant");
        assert_eq!(Lang::parse("zh-HK").unwrap().code(), "zh-Hant");
        assert_eq!(Lang::parse("zh-Hant-HK").unwrap().code(), "zh-Hant");
        assert_eq!(Lang::parse("zh-Hans").unwrap().code(), "zh-Hans");
    }

    #[test]
    fn invalid_tags_error() {
        assert!(Lang::parse("").is_err());
        assert!(Lang::parse("x").is_err());
        assert!(Lang::parse("1234").is_err());
    }

    #[test]
    fn cjk_join_detection() {
        assert!(Lang::parse("ja").unwrap().joins_without_space());
        assert!(Lang::parse("zh-TW").unwrap().joins_without_space());
        assert!(!Lang::parse("ko").unwrap().joins_without_space());
        assert!(!Lang::parse("de").unwrap().joins_without_space());
    }
}
