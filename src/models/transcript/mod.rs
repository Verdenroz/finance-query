//! Earnings call transcript models.
//!
//! Typed models for Yahoo Finance earnings call transcripts.

use serde::{Deserialize, Serialize};

/// Full transcript response from Yahoo Finance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transcript {
    /// The transcript content including speakers and paragraphs.
    pub transcript_content: TranscriptContent,
    /// Metadata about the transcript.
    pub transcript_metadata: TranscriptMetadata,
}

impl Transcript {
    /// Get the full transcript text.
    pub fn text(&self) -> &str {
        self.transcript_content
            .transcript
            .as_ref()
            .map(|t| t.text.as_str())
            .unwrap_or("")
    }

    /// Get the fiscal quarter (e.g., "Q4").
    pub fn quarter(&self) -> &str {
        &self.transcript_metadata.fiscal_period
    }

    /// Get the fiscal year.
    pub fn year(&self) -> i32 {
        self.transcript_metadata.fiscal_year
    }

    /// Get speaker name by speaker ID.
    pub fn speaker_name(&self, speaker_id: i32) -> Option<&str> {
        self.transcript_content
            .speaker_mapping
            .iter()
            .find(|s| s.speaker == speaker_id)
            .map(|s| s.speaker_data.name.as_str())
    }

    /// Get all paragraphs with speaker names resolved.
    pub fn paragraphs_with_speakers(&self) -> Vec<(&Paragraph, Option<&str>)> {
        self.transcript_content
            .transcript
            .as_ref()
            .map(|t| {
                t.paragraphs
                    .iter()
                    .map(|p| (p, self.speaker_name(p.speaker)))
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// Transcript content including speakers and full transcript.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptContent {
    /// Company ID (quartrId).
    pub company_id: i64,
    /// Event ID for this transcript.
    pub event_id: i64,
    /// Version of the transcript format.
    #[serde(default)]
    pub version: Option<String>,
    /// Mapping of speaker IDs to speaker information.
    #[serde(default)]
    pub speaker_mapping: Vec<SpeakerMapping>,
    /// The full transcript data.
    #[serde(default)]
    pub transcript: Option<TranscriptData>,
}

/// Mapping of a speaker ID to speaker information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeakerMapping {
    /// Speaker ID (referenced in paragraphs).
    pub speaker: i32,
    /// Speaker details.
    pub speaker_data: SpeakerData,
}

/// Information about a speaker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeakerData {
    /// Company the speaker represents.
    #[serde(default)]
    pub company: Option<String>,
    /// Speaker's name.
    #[serde(default)]
    pub name: String,
    /// Speaker's role/title.
    #[serde(default)]
    pub role: Option<String>,
}

/// Full transcript data with paragraphs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptData {
    /// Number of speakers in the call.
    #[serde(default)]
    pub number_of_speakers: i32,
    /// Full text of the transcript.
    #[serde(default)]
    pub text: String,
    /// Paragraphs (sections spoken by each speaker).
    #[serde(default)]
    pub paragraphs: Vec<Paragraph>,
}

/// A paragraph (section spoken by one speaker).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paragraph {
    /// Speaker ID (use speaker_mapping to get name).
    #[serde(default)]
    pub speaker: i32,
    /// Start time in seconds.
    #[serde(default)]
    pub start: f64,
    /// End time in seconds.
    #[serde(default)]
    pub end: f64,
    /// Full text of this paragraph.
    #[serde(default)]
    pub text: String,
    /// Sentences in this paragraph.
    #[serde(default)]
    pub sentences: Vec<Sentence>,
}

/// A sentence within a paragraph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sentence {
    /// Start time in seconds.
    #[serde(default)]
    pub start: f64,
    /// End time in seconds.
    #[serde(default)]
    pub end: f64,
    /// Text of the sentence.
    #[serde(default)]
    pub text: String,
    /// Individual words with timing and confidence.
    #[serde(default)]
    pub words: Vec<Word>,
}

/// A word with timing and confidence information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Word {
    /// The word (lowercase).
    #[serde(default)]
    pub word: String,
    /// The word with punctuation.
    #[serde(default)]
    pub punctuated_word: String,
    /// Start time in seconds.
    #[serde(default)]
    pub start: f64,
    /// End time in seconds.
    #[serde(default)]
    pub end: f64,
    /// Confidence score (0.0 - 1.0).
    #[serde(default)]
    pub confidence: f64,
}

/// Metadata about the transcript.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptMetadata {
    /// Date of the earnings call (Unix timestamp).
    #[serde(default)]
    pub date: i64,
    /// Event ID.
    #[serde(default)]
    pub event_id: i64,
    /// Type of event (e.g., "Earnings Call").
    #[serde(default)]
    pub event_type: String,
    /// Fiscal period (e.g., "Q4").
    #[serde(default)]
    pub fiscal_period: String,
    /// Fiscal year.
    #[serde(default)]
    pub fiscal_year: i32,
    /// Whether this is the latest transcript.
    #[serde(default)]
    pub is_latest: bool,
    /// S3 URL for the transcript data.
    #[serde(default)]
    pub s3_url: String,
    /// Title (e.g., "Q4 2025").
    #[serde(default)]
    pub title: String,
    /// Transcript ID.
    #[serde(default)]
    pub transcript_id: i64,
    /// Transcript type (e.g., "IN_HOUSE").
    #[serde(default, rename = "type")]
    pub transcript_type: String,
    /// Last updated timestamp.
    #[serde(default)]
    pub updated: i64,
}

/// Transcript with metadata from the earnings call list.
///
/// Used when fetching multiple transcripts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptWithMeta {
    /// Event ID.
    pub event_id: String,
    /// Fiscal quarter (e.g., "Q1", "Q2", "Q3", "Q4").
    pub quarter: Option<String>,
    /// Fiscal year.
    pub year: Option<i32>,
    /// Title of the earnings call.
    pub title: String,
    /// URL to the earnings call page.
    pub url: String,
    /// The full transcript.
    pub transcript: Transcript,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_transcript() {
        let json = r#"{
            "transcriptContent": {
                "company_id": 4742,
                "event_id": 369370,
                "version": "1.0.0",
                "speaker_mapping": [
                    {
                        "speaker": 0,
                        "speaker_data": {
                            "company": "Apple",
                            "name": "Tim Cook",
                            "role": "CEO"
                        }
                    }
                ],
                "transcript": {
                    "number_of_speakers": 15,
                    "text": "Hello everyone...",
                    "paragraphs": []
                }
            },
            "transcriptMetadata": {
                "date": 1761858000,
                "eventId": 369370,
                "eventType": "Earnings Call",
                "fiscalPeriod": "Q4",
                "fiscalYear": 2025,
                "isLatest": true,
                "title": "Q4 2025"
            }
        }"#;

        let transcript: Transcript = serde_json::from_str(json).unwrap();
        assert_eq!(transcript.transcript_content.company_id, 4742);
        assert_eq!(transcript.quarter(), "Q4");
        assert_eq!(transcript.year(), 2025);
        assert_eq!(transcript.speaker_name(0), Some("Tim Cook"));
    }
}
