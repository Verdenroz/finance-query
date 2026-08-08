//! Formatted value support for GraphQL responses.
//!
//! Yahoo Finance returns many numeric fields as `{"raw": 182.5, "fmt": "182.50", "longFmt": "182.50"}`
//! objects. Resolvers apply `finance_query::ValueFormat::transform()` on the raw JSON before
//! deserializing into GraphQL types, so formatted fields are exposed as
//! `Json<GqlFormattedValue>` and will contain a scalar (raw mode), string (fmt mode), or the
//! full object (both mode).
//!
//! [`GqlFormattedValue`] models exactly those three shapes plus a catch-all, so the field is
//! documented as a real union instead of an unconstrained JSON blob — while still accepting
//! anything upstream sends, exactly as a bare `serde_json::Value` did.

use serde::{Deserialize, Serialize};

/// The `{raw, fmt, longFmt}` wrapper as it reaches the wire under `format=both`.
///
/// `ValueFormat::Both` performs no transformation, so this is whatever
/// `finance_query::FormattedValue<T>` serialized to: `fmt` and `longFmt` are omitted when
/// absent, and `raw` carries the underlying datum (a number for most fields, occasionally a
/// string, `null` when upstream has no value).
///
/// `raw` is required and unknown keys are rejected, mirroring
/// `ValueFormat::is_formatted_value` — an object that fails either test is not a formatted
/// value and falls through to [`GqlFormattedValue::Other`], where it round-trips untouched.
/// Field order matches `finance_query::FormattedValue`'s so re-serialization is byte-identical.
///
/// Unlike the surrounding `Gql*` structs, serde *is* the wire truth here: this type lives
/// inside an opaque `JSON` scalar, which async-graphql resolves through `Serialize`. The one
/// key needing a rename is spelled out per-field rather than via a container-wide
/// `rename_all` — the library's `FormattedValue` already emits `longFmt`, and a blanket rule
/// would be a standing hazard on a type that also parses library output.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct GqlFormattedObject {
    /// Human-readable rendering, e.g. `"182.50"` or `"3.50T"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fmt: Option<String>,
    /// Fully expanded rendering for large numbers, e.g. `"3,500,000,000,000"`.
    #[serde(rename = "longFmt", default, skip_serializing_if = "Option::is_none")]
    pub long_fmt: Option<String>,
    /// The underlying datum. May be `null` when upstream reports no value.
    pub raw: serde_json::Value,
}

/// One Yahoo Finance formatted value, in whichever shape the request's `format` selected.
///
/// The arms correspond one-to-one with `finance_query::ValueFormat`:
///
/// - `format=raw` (the default) yields the bare `raw` datum — [`Int`] or [`Float`] for a
///   number, [`Display`] on the rare occasion `raw` is a string.
/// - `format=pretty` yields the `fmt` (or `longFmt`) string — [`Display`].
/// - `format=both` yields the untouched wrapper object — [`Both`].
/// - Anything else upstream sends lands in [`Other`], which holds an arbitrary
///   `serde_json::Value`. It is deliberately the **last** variant: serde tries untagged arms in
///   declaration order, so it only ever catches what the modelled shapes rejected, and it keeps
///   this type exactly as permissive as the bare `serde_json::Value` it replaced.
///
/// The numeric split into [`Int`]/[`Float`] is not a fourth shape — it is the same bare scalar,
/// typed as the two numeric flavours JSON actually distinguishes. Collapsing both into `f64`
/// would re-serialize a market cap of `3500000000000` as `3500000000000.0`.
///
/// [`Int`]: GqlFormattedValue::Int
/// [`Float`]: GqlFormattedValue::Float
/// [`Display`]: GqlFormattedValue::Display
/// [`Both`]: GqlFormattedValue::Both
/// [`Other`]: GqlFormattedValue::Other
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum GqlFormattedValue {
    /// `format=raw` with an integral datum (share counts, market caps, epoch seconds).
    Int(i64),
    /// `format=raw` with a fractional datum (prices, ratios, percentages).
    Float(f64),
    /// `format=pretty` — the display string; also covers a string-valued `raw`.
    Display(String),
    /// `format=both` — the full `{raw, fmt, longFmt}` wrapper.
    Both(GqlFormattedObject),
    /// Catch-all for any other shape. Must remain the last variant.
    Other(serde_json::Value),
}

impl Default for GqlFormattedValue {
    /// `null`, matching what a defaulted `serde_json::Value` used to produce.
    fn default() -> Self {
        Self::Other(serde_json::Value::Null)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use finance_query::ValueFormat;
    use serde_json::json;

    /// One field as `finance_query::FormattedValue<T>` serializes it: `fmt`/`longFmt` omitted
    /// when absent, `raw` carrying the datum.
    fn both_payload() -> serde_json::Value {
        json!({
            "marketCap": { "fmt": "3.50T", "longFmt": "3,500,000,000,000", "raw": 3_500_000_000_000i64 },
            "regularMarketPrice": { "fmt": "182.50", "raw": 182.5 },
        })
    }

    fn roundtrip(v: &serde_json::Value) -> serde_json::Value {
        let parsed: GqlFormattedValue = serde_json::from_value(v.clone()).expect("deserializes");
        serde_json::to_value(&parsed).expect("serializes")
    }

    #[test]
    fn raw_mode_lands_on_the_numeric_arms_and_round_trips_exactly() {
        let t = ValueFormat::Raw.transform(both_payload());
        let cap = &t["marketCap"];
        let price = &t["regularMarketPrice"];

        assert_eq!(
            serde_json::from_value::<GqlFormattedValue>(cap.clone()).expect("cap"),
            GqlFormattedValue::Int(3_500_000_000_000)
        );
        assert_eq!(
            serde_json::from_value::<GqlFormattedValue>(price.clone()).expect("price"),
            GqlFormattedValue::Float(182.5)
        );

        // Byte-exact: an integer must not come back as `3500000000000.0`.
        assert_eq!(roundtrip(cap).to_string(), "3500000000000");
        assert_eq!(roundtrip(price).to_string(), "182.5");
    }

    #[test]
    fn pretty_mode_lands_on_the_display_arm() {
        let t = ValueFormat::Pretty.transform(both_payload());
        assert_eq!(
            serde_json::from_value::<GqlFormattedValue>(t["marketCap"].clone()).expect("cap"),
            GqlFormattedValue::Display("3.50T".into())
        );
        assert_eq!(roundtrip(&t["regularMarketPrice"]), json!("182.50"));
    }

    #[test]
    fn both_mode_keeps_the_wrapper_including_long_fmt() {
        let t = ValueFormat::Both.transform(both_payload());
        let cap = &t["marketCap"];

        let parsed: GqlFormattedValue = serde_json::from_value(cap.clone()).expect("cap");
        let GqlFormattedValue::Both(obj) = &parsed else {
            panic!("expected the wrapper arm, got {parsed:?}");
        };
        assert_eq!(obj.fmt.as_deref(), Some("3.50T"));
        // `longFmt` is genuinely emitted by `FormattedValue` and whitelisted by
        // `ValueFormat::is_formatted_value`, so it belongs in the modelled shape.
        assert_eq!(obj.long_fmt.as_deref(), Some("3,500,000,000,000"));
        assert_eq!(obj.raw, json!(3_500_000_000_000i64));

        assert_eq!(roundtrip(cap), *cap);
        // A wrapper without `longFmt` must not gain a `"longFmt": null` key.
        let price = &t["regularMarketPrice"];
        assert_eq!(roundtrip(price), *price);
    }

    #[test]
    fn unexpected_shapes_fall_through_to_the_catch_all_untouched() {
        let odd = [
            json!(true),
            json!(null),
            json!([1, 2, 3]),
            json!({}),                                  // FormattedValue with nothing set
            json!({ "fmt": "N/A" }),                    // wrapper with no `raw`
            json!({ "raw": 1, "surprise": "new key" }), // not a wrapper per is_formatted_value
            json!({ "nested": { "raw": 1 } }),
        ];
        for v in odd {
            let parsed: GqlFormattedValue =
                serde_json::from_value(v.clone()).unwrap_or_else(|e| panic!("{v} failed: {e}"));
            assert!(
                matches!(parsed, GqlFormattedValue::Other(_)),
                "{v} should hit the catch-all, got {parsed:?}"
            );
            assert_eq!(roundtrip(&v), v, "{v} must round-trip untouched");
        }
    }

    #[test]
    fn string_valued_raw_and_null_raw_still_work() {
        // `raw` is typed as an arbitrary value, so a non-numeric datum keeps the wrapper arm.
        for v in [
            json!({ "raw": null }),
            json!({ "fmt": "Q3", "raw": "Q3 2025" }),
        ] {
            let parsed: GqlFormattedValue = serde_json::from_value(v.clone()).expect("wrapper");
            assert!(matches!(parsed, GqlFormattedValue::Both(_)), "{v}");
            assert_eq!(roundtrip(&v), v);
        }
    }

    /// The GraphQL layer must produce the same bytes as before: async-graphql resolves
    /// `Json<T>` through `to_value(&T)`, so this pins the JSON an API consumer receives.
    #[test]
    fn graphql_resolution_matches_the_bare_json_value() {
        let t_raw = ValueFormat::Raw.transform(both_payload());
        let t_both = ValueFormat::Both.transform(both_payload());
        for v in [
            t_raw["marketCap"].clone(),
            t_raw["regularMarketPrice"].clone(),
            t_both["marketCap"].clone(),
            t_both["regularMarketPrice"].clone(),
            json!({ "fmt": "N/A" }),
        ] {
            let typed: GqlFormattedValue = serde_json::from_value(v.clone()).expect("typed");
            assert_eq!(
                async_graphql::to_value(&typed).expect("typed -> ConstValue"),
                async_graphql::to_value(&v).expect("Value -> ConstValue"),
                "GraphQL output drifted for {v}"
            );
        }
    }
}
