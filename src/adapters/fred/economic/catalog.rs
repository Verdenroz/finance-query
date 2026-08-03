//! FRED series discovery (search, category and release browsing) and ALFRED
//! vintages.
//!
//! Without these, a caller has to already know a series id exists. The vintage
//! parameters matter separately: FRED revises macro data, so backtesting a
//! strategy against today's `GDPC1` is look-ahead bias — `as_of` asks for the
//! series as it stood on a past date.

use serde::Deserialize;

use crate::adapters::fred::build_client;
use crate::error::Result;
use crate::models::economic::{
    EconomicCategory, EconomicRelease, EconomicSeries, EconomicSeriesMatch, MacroObservation,
};

// ============================================================================
// Response types
// ============================================================================

/// `/series/search` and `/series` envelope.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeriesSearchResponseDTO {
    /// Matching series. FRED spells the key `seriess` (sic).
    #[serde(default)]
    pub seriess: Vec<SeriesDTO>,
}

/// One series entry.
#[derive(Debug, Clone, Deserialize)]
pub struct SeriesDTO {
    /// Series id.
    pub id: String,
    /// Title.
    pub title: Option<String>,
    /// Reporting frequency, long form.
    pub frequency: Option<String>,
    /// Units, long form.
    pub units: Option<String>,
    /// Seasonal adjustment, long form.
    pub seasonal_adjustment: Option<String>,
    /// Earliest observation date.
    pub observation_start: Option<String>,
    /// Latest observation date.
    pub observation_end: Option<String>,
    /// FRED popularity score.
    pub popularity: Option<i64>,
    /// Free-text notes.
    pub notes: Option<String>,
}

/// `/category/children` envelope.
#[derive(Debug, Clone, Deserialize)]
pub struct CategoriesResponseDTO {
    /// Child categories.
    #[serde(default)]
    pub categories: Vec<CategoryDTO>,
}

/// One category entry.
#[derive(Debug, Clone, Deserialize)]
pub struct CategoryDTO {
    /// Category id.
    pub id: i64,
    /// Category name.
    pub name: Option<String>,
    /// Parent category id.
    pub parent_id: Option<i64>,
}

/// `/releases` envelope.
#[derive(Debug, Clone, Deserialize)]
pub struct ReleasesResponseDTO {
    /// Releases.
    #[serde(default)]
    pub releases: Vec<ReleaseDTO>,
}

/// One release entry.
#[derive(Debug, Clone, Deserialize)]
pub struct ReleaseDTO {
    /// Release id.
    pub id: i64,
    /// Release name.
    pub name: Option<String>,
    /// Whether a press release accompanies it.
    pub press_release: Option<bool>,
    /// Publisher link.
    pub link: Option<String>,
}

/// `/series/observations` envelope.
#[derive(Debug, Clone, Deserialize)]
pub struct ObservationsResponseDTO {
    /// Observations.
    #[serde(default)]
    pub observations: Vec<ObservationDTO>,
}

/// One observation. FRED sends values as strings, using `"."` for missing.
#[derive(Debug, Clone, Deserialize)]
pub struct ObservationDTO {
    /// Observation date.
    pub date: String,
    /// Observation value as filed.
    pub value: String,
}

// ============================================================================
// Canonical conversions
// ============================================================================

pub(crate) fn to_series_match(dto: SeriesDTO) -> EconomicSeriesMatch {
    EconomicSeriesMatch {
        id: dto.id,
        title: dto.title,
        frequency: dto.frequency,
        units: dto.units,
        seasonal_adjustment: dto.seasonal_adjustment,
        observation_start: dto.observation_start,
        observation_end: dto.observation_end,
        popularity: dto.popularity,
        notes: dto.notes,
    }
}

pub(crate) fn to_category(dto: CategoryDTO) -> EconomicCategory {
    EconomicCategory {
        id: dto.id,
        name: dto.name,
        parent_id: dto.parent_id,
    }
}

pub(crate) fn to_release(dto: ReleaseDTO) -> EconomicRelease {
    EconomicRelease {
        id: dto.id,
        name: dto.name,
        press_release: dto.press_release,
        link: dto.link,
    }
}

pub(crate) fn to_observations(dto: ObservationsResponseDTO) -> Vec<MacroObservation> {
    dto.observations
        .into_iter()
        .map(|o| MacroObservation {
            date: o.date,
            // "." is FRED's missing marker; anything unparseable is treated the
            // same way rather than dropping the observation's date entirely.
            value: (o.value != ".").then(|| o.value.parse().ok()).flatten(),
        })
        .collect()
}

// ============================================================================
// Query functions
// ============================================================================

/// Search the FRED series catalog by free text.
pub async fn search(query: &str, limit: u32) -> Result<Vec<EconomicSeriesMatch>> {
    let limit = limit.clamp(1, 1000).to_string();
    let resp: SeriesSearchResponseDTO = build_client()?
        .get_json(
            "series/search",
            &[
                ("search_text", query),
                ("limit", &limit),
                ("order_by", "popularity"),
                ("sort_order", "desc"),
            ],
        )
        .await?;
    Ok(resp.seriess.into_iter().map(to_series_match).collect())
}

/// List the child categories of `parent_id`. FRED's root category is `0`.
pub async fn categories(parent_id: i64) -> Result<Vec<EconomicCategory>> {
    let id = parent_id.to_string();
    let resp: CategoriesResponseDTO = build_client()?
        .get_json("category/children", &[("category_id", &id)])
        .await?;
    Ok(resp.categories.into_iter().map(to_category).collect())
}

/// List every economic-data release FRED publishes.
pub async fn releases() -> Result<Vec<EconomicRelease>> {
    let resp: ReleasesResponseDTO = build_client()?.get_json("releases", &[]).await?;
    Ok(resp.releases.into_iter().map(to_release).collect())
}

/// Fetch a series as it stood on `date` (`YYYY-MM-DD`) — ALFRED's vintage view.
///
/// Both realtime bounds are pinned to `date` so the response contains exactly
/// the values published as of that day, not a range of revisions.
pub async fn series_as_of(series_id: &str, date: &str) -> Result<EconomicSeries> {
    let resp: ObservationsResponseDTO = build_client()?
        .get_json(
            "series/observations",
            &[
                ("series_id", series_id),
                ("realtime_start", date),
                ("realtime_end", date),
            ],
        )
        .await?;

    Ok(EconomicSeries {
        series_id: series_id.to_string(),
        title: None,
        units: None,
        frequency: None,
        observations: to_observations(resp),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_a_search_hit_including_fmts_misspelled_envelope_key() {
        let resp: SeriesSearchResponseDTO = serde_json::from_value(serde_json::json!({
            "seriess": [{
                "id": "GDPC1",
                "title": "Real Gross Domestic Product",
                "frequency": "Quarterly",
                "units": "Billions of Chained 2017 Dollars",
                "seasonal_adjustment": "Seasonally Adjusted Annual Rate",
                "observation_start": "1947-01-01",
                "observation_end": "2024-07-01",
                "popularity": 93,
                "notes": "BEA Account Code: A191RX"
            }]
        }))
        .unwrap();

        let out = to_series_match(resp.seriess.into_iter().next().unwrap());
        assert_eq!(out.id, "GDPC1");
        assert_eq!(out.title.as_deref(), Some("Real Gross Domestic Product"));
        assert_eq!(out.frequency.as_deref(), Some("Quarterly"));
        assert_eq!(out.observation_start.as_deref(), Some("1947-01-01"));
        assert_eq!(out.popularity, Some(93));
    }

    #[test]
    fn missing_envelope_arrays_yield_empty_results() {
        let search: SeriesSearchResponseDTO =
            serde_json::from_value(serde_json::json!({})).unwrap();
        assert!(search.seriess.is_empty());
        let cats: CategoriesResponseDTO = serde_json::from_value(serde_json::json!({})).unwrap();
        assert!(cats.categories.is_empty());
        let rels: ReleasesResponseDTO = serde_json::from_value(serde_json::json!({})).unwrap();
        assert!(rels.releases.is_empty());
    }

    #[test]
    fn maps_categories_and_releases() {
        let cats: CategoriesResponseDTO = serde_json::from_value(serde_json::json!({
            "categories": [{ "id": 32992, "name": "Money, Banking, & Finance", "parent_id": 0 }]
        }))
        .unwrap();
        let cat = to_category(cats.categories.into_iter().next().unwrap());
        assert_eq!(cat.id, 32992);
        assert_eq!(cat.parent_id, Some(0));

        let rels: ReleasesResponseDTO = serde_json::from_value(serde_json::json!({
            "releases": [{
                "id": 50,
                "name": "Employment Situation",
                "press_release": true,
                "link": "https://www.bls.gov/news.release/empsit.htm"
            }]
        }))
        .unwrap();
        let rel = to_release(rels.releases.into_iter().next().unwrap());
        assert_eq!(rel.name.as_deref(), Some("Employment Situation"));
        assert_eq!(rel.press_release, Some(true));
    }

    /// FRED encodes a missing observation as `"."`; keeping the date with a
    /// `None` value preserves the series' spacing.
    #[test]
    fn missing_observations_keep_their_date() {
        let resp: ObservationsResponseDTO = serde_json::from_value(serde_json::json!({
            "observations": [
                { "date": "2020-01-01", "value": "21538.032" },
                { "date": "2020-04-01", "value": "." },
                { "date": "2020-07-01", "value": "not-a-number" }
            ]
        }))
        .unwrap();

        let obs = to_observations(resp);
        assert_eq!(obs.len(), 3);
        assert_eq!(obs[0].value, Some(21538.032));
        assert_eq!(obs[1].date, "2020-04-01");
        assert_eq!(obs[1].value, None);
        assert_eq!(obs[2].value, None);
    }
}
