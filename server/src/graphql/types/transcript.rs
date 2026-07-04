//! GraphQL types for earnings call transcripts.

use crate::graphql::pagination::{self, Page};
use async_graphql::{ComplexObject, Result, SimpleObject};
use serde::Deserialize;

// ── Top-level wrapper (from get_transcripts call) ──────────────────────────

#[derive(SimpleObject, Deserialize, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct GqlTranscriptWithMeta {
    pub event_id: String,
    pub quarter: Option<String>,
    pub year: Option<i32>,
    pub title: String,
    pub url: String,
    pub transcript: GqlTranscript,
}

// ── Transcript ─────────────────────────────────────────────────────────────
// Mirrors `finance_query::models::corporate::transcript`'s pervasive `#[serde(default)]` (scraped
// data, see CLAUDE.md) — every nested type must stay tolerant since selection applies post-deserialize.

#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlTranscript {
    pub transcript_content: GqlTranscriptContent,
    pub transcript_metadata: GqlTranscriptMetadata,
}

#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlTranscriptContent {
    pub company_id: i64,
    pub event_id: i64,
    pub version: Option<String>,
    pub speaker_mapping: Vec<GqlSpeakerMapping>,
    pub transcript: Option<GqlTranscriptData>,
}

#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct GqlSpeakerMapping {
    pub speaker: i32,
    pub speaker_data: GqlSpeakerData,
}

#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct GqlSpeakerData {
    pub company: Option<String>,
    pub name: String,
    pub role: Option<String>,
}

#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase", complex)]
#[serde(rename_all = "camelCase", default)]
pub struct GqlTranscriptData {
    pub number_of_speakers: i32,
    pub text: String,
    #[graphql(skip)]
    pub paragraphs: Vec<GqlParagraph>,
}

#[ComplexObject(rename_fields = "camelCase")]
impl GqlTranscriptData {
    /// Per-paragraph breakdown (speaker, start/end timestamp, text). A full
    /// call's `text` blob can be tens of thousands of tokens — paginate
    /// through paragraphs instead when only part of the call is needed.
    async fn paragraphs(
        &self,
        #[graphql(desc = "Max paragraphs to return; omitted = every paragraph in one page")]
        first: Option<i32>,
        #[graphql(desc = "Opaque continuation cursor from a previous page's endCursor")]
        after: Option<String>,
    ) -> Result<Page<GqlParagraph>> {
        pagination::paginate(self.paragraphs.clone(), first, after).await
    }
}

#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct GqlParagraph {
    pub speaker: i32,
    pub start: f64,
    pub end: f64,
    pub text: String,
    pub sentences: Vec<GqlSentence>,
}

#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct GqlSentence {
    pub start: f64,
    pub end: f64,
    pub text: String,
    pub words: Vec<GqlWord>,
}

#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlWord {
    pub word: String,
    pub punctuated_word: String,
    pub start: f64,
    pub end: f64,
    pub confidence: f64,
}

// ── Metadata ────────────────────────────────────────────────────────────────

#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlTranscriptMetadata {
    pub date: i64,
    pub event_id: i64,
    pub event_type: String,
    pub fiscal_period: String,
    pub fiscal_year: i32,
    pub is_latest: bool,
    pub s3_url: String,
    pub title: String,
    pub transcript_id: i64,
    #[serde(rename = "type")]
    pub transcript_type: String,
    pub updated: i64,
}
