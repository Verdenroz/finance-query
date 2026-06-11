//! Sentence segmentation for the offline translation backend.
//!
//! NLLB-class models are trained on sentence-level inputs; feeding a whole
//! multi-sentence business summary degrades quality and risks truncation.
//! This splitter is abbreviation-aware so company suffixes like "Inc." do
//! not produce spurious sentence boundaries.

/// English abbreviations that end with a period but do not end a sentence.
const ABBREVIATIONS: &[&str] = &[
    "Inc.", "Corp.", "Ltd.", "Co.", "Cos.", "S.A.", "N.V.", "A.G.", "plc.", "PLC.", "L.P.", "LLC.",
    "L.L.C.", "U.S.", "U.K.", "U.N.", "No.", "Nos.", "Mr.", "Mrs.", "Ms.", "Dr.", "Jr.", "Sr.",
    "St.", "vs.", "etc.", "approx.", "est.", "Est.", "Bros.", "Hldgs.", "Mfg.",
];

/// Split text into sentence-level segments suitable for machine translation.
///
/// Splits after `.`, `!`, or `?` followed by whitespace, unless the token is a
/// known abbreviation or looks like an initial (e.g. "D." in "Timothy D. Cook").
pub(crate) fn split_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();
    for word in text.split_inclusive(char::is_whitespace) {
        current.push_str(word);
        let token = word.trim_end();
        if (token.ends_with('.') || token.ends_with('!') || token.ends_with('?'))
            && !is_non_terminal(token)
        {
            let sentence = current.trim();
            if !sentence.is_empty() {
                sentences.push(sentence.to_string());
            }
            current = String::new();
        }
    }
    let rest = current.trim();
    if !rest.is_empty() {
        sentences.push(rest.to_string());
    }
    sentences
}

/// True if a period-terminated token should not end a sentence.
fn is_non_terminal(token: &str) -> bool {
    if !token.ends_with('.') {
        return false;
    }
    if ABBREVIATIONS.iter().any(|a| token.ends_with(a)) {
        return true;
    }
    // Single-letter initials like "D." or "J.P."
    let bare = token.trim_start_matches(['(', '"', '\'']);
    bare.len() <= 2 && bare.chars().next().is_some_and(|c| c.is_ascii_uppercase())
}

/// Rejoin translated sentence segments, with or without separating spaces
/// depending on the target script.
pub(crate) fn join_sentences(sentences: &[String], without_space: bool) -> String {
    let sep = if without_space { "" } else { " " };
    sentences.join(sep)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_plain_sentences() {
        let s = split_sentences("First sentence. Second sentence! Third?");
        assert_eq!(s.len(), 3);
        assert_eq!(s[0], "First sentence.");
        assert_eq!(s[2], "Third?");
    }

    #[test]
    fn keeps_company_suffixes_intact() {
        let s =
            split_sentences("Apple Inc. designs smartphones worldwide. It also provides services.");
        assert_eq!(s.len(), 2);
        assert!(s[0].starts_with("Apple Inc. designs"));
    }

    #[test]
    fn keeps_initials_intact() {
        let s = split_sentences("Timothy D. Cook is the CEO. He joined in 1998.");
        assert_eq!(s.len(), 2);
        assert!(s[0].contains("D. Cook"));
    }

    #[test]
    fn handles_empty_and_single() {
        assert!(split_sentences("").is_empty());
        assert_eq!(
            split_sentences("No terminal punctuation"),
            vec!["No terminal punctuation".to_string()]
        );
    }

    #[test]
    fn join_respects_script() {
        let parts = vec!["一文目。".to_string(), "二文目。".to_string()];
        assert_eq!(join_sentences(&parts, true), "一文目。二文目。");
        let parts = vec!["Erster Satz.".to_string(), "Zweiter Satz.".to_string()];
        assert_eq!(join_sentences(&parts, false), "Erster Satz. Zweiter Satz.");
    }
}
