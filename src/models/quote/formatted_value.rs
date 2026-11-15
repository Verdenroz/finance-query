/// Formatted value type used throughout Yahoo Finance API responses
///
/// Many numeric fields in Yahoo Finance responses follow this pattern:
/// - `raw`: The actual numeric value
/// - `fmt`: Human-readable formatted string (e.g., "276.97")
/// - `longFmt`: Long format for large numbers (e.g., "14,776,353,000")
///
/// # Examples
///
/// ```json
/// {
///   "fmt": "276.97",
///   "raw": 276.97
/// }
/// ```
///
/// ```json
/// {
///   "fmt": "14.78B",
///   "longFmt": "14,776,353,000",
///   "raw": 14776353000
/// }
/// ```
/// A generic type representing Yahoo Finance's formatted value pattern
///
/// Contains the raw numeric value along with optional formatted representations.
/// Note: `raw` is optional because Yahoo sometimes returns empty objects `{}` for unavailable data.
///
/// A value with no `raw`, `fmt` or `longFmt` serialises as `null`, not `{}`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FormattedValue<T> {
    /// Human-readable formatted string
    pub fmt: Option<String>,

    /// Long format (for large numbers with full precision)
    pub long_fmt: Option<String>,

    /// Raw numeric value (None if data is unavailable)
    pub raw: Option<T>,
}

impl<T: serde::Serialize> serde::Serialize for FormattedValue<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;

        if self.fmt.is_none() && self.long_fmt.is_none() && self.raw.is_none() {
            return serializer.serialize_none();
        }
        let len = usize::from(self.fmt.is_some())
            + usize::from(self.long_fmt.is_some())
            + usize::from(self.raw.is_some());
        let mut map = serializer.serialize_map(Some(len))?;
        if let Some(fmt) = &self.fmt {
            map.serialize_entry("fmt", fmt)?;
        }
        if let Some(long_fmt) = &self.long_fmt {
            map.serialize_entry("longFmt", long_fmt)?;
        }
        if let Some(raw) = &self.raw {
            map.serialize_entry("raw", raw)?;
        }
        map.end()
    }
}

// Custom Deserialize that accepts both full objects AND bare values.
// Bare values (from ValueFormat::Raw transform) become FormattedValue { raw: Some(v), .. }.
impl<'de, T: serde::Deserialize<'de>> serde::Deserialize<'de> for FormattedValue<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Full<T> {
            fmt: Option<String>,
            long_fmt: Option<String>,
            raw: Option<T>,
        }

        #[derive(serde::Deserialize)]
        #[serde(untagged)]
        // `Null` goes last: an untagged enum pays for every failed variant,
        // and nearly every value is `Full` or `Bare`.
        enum Helper<T> {
            Full(Full<T>),
            Bare(T),
            Null(()),
        }

        match Helper::<T>::deserialize(deserializer)? {
            Helper::Null(()) => Ok(FormattedValue {
                fmt: None,
                long_fmt: None,
                raw: None,
            }),
            Helper::Full(f) => Ok(FormattedValue {
                fmt: f.fmt,
                long_fmt: f.long_fmt,
                raw: f.raw,
            }),
            Helper::Bare(v) => Ok(FormattedValue {
                fmt: None,
                long_fmt: None,
                raw: Some(v),
            }),
        }
    }
}

impl FormattedValue<f64> {
    /// Rescales `raw` from a fraction to percent, leaving `fmt` and `long_fmt` untouched.
    pub(crate) fn fraction_to_percent(self) -> Self {
        Self {
            raw: self.raw.map(|v| v * 100.0),
            ..self
        }
    }
}

impl<T> FormattedValue<T> {
    /// Create a new FormattedValue with just a raw value
    pub fn new(raw: T) -> Self {
        Self {
            fmt: None,
            long_fmt: None,
            raw: Some(raw),
        }
    }

    /// Create a FormattedValue with raw and formatted values
    pub fn with_fmt(raw: T, fmt: String) -> Self {
        Self {
            fmt: Some(fmt),
            long_fmt: None,
            raw: Some(raw),
        }
    }

    /// Create a FormattedValue with all fields
    pub fn with_all(raw: T, fmt: String, long_fmt: String) -> Self {
        Self {
            fmt: Some(fmt),
            long_fmt: Some(long_fmt),
            raw: Some(raw),
        }
    }

    /// Get the raw value
    pub fn value(&self) -> Option<&T> {
        self.raw.as_ref()
    }

    /// Get the formatted string, falling back to long format, then None
    pub fn formatted(&self) -> Option<&str> {
        self.fmt.as_deref().or(self.long_fmt.as_deref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_simple() {
        let json = r#"{"fmt": "276.97", "raw": 276.97}"#;
        let value: FormattedValue<f64> = serde_json::from_str(json).unwrap();
        assert_eq!(value.raw, Some(276.97));
        assert_eq!(value.fmt.as_deref(), Some("276.97"));
        assert_eq!(value.long_fmt, None);
    }

    #[test]
    fn test_deserialize_with_long_fmt() {
        let json = r#"{"fmt": "14.78B", "longFmt": "14,776,353,000", "raw": 14776353000}"#;
        let value: FormattedValue<i64> = serde_json::from_str(json).unwrap();
        assert_eq!(value.raw, Some(14776353000));
        assert_eq!(value.fmt.as_deref(), Some("14.78B"));
        assert_eq!(value.long_fmt.as_deref(), Some("14,776,353,000"));
    }

    #[test]
    fn test_formatted_helper() {
        let value = FormattedValue::with_fmt(100.5, "100.50".to_string());
        assert_eq!(value.formatted(), Some("100.50"));

        let value = FormattedValue::new(100.5);
        assert_eq!(value.formatted(), None);
    }
}

#[cfg(test)]
mod test_empty_object {
    use super::*;

    #[test]
    fn test_empty_object_deserializes() {
        let json = "{}";
        let result: Result<FormattedValue<f64>, _> = serde_json::from_str(json);
        assert!(
            result.is_ok(),
            "Empty object should deserialize: {:?}",
            result.err()
        );

        let fv = result.unwrap();
        assert_eq!(fv.raw, None);
        assert_eq!(fv.fmt, None);
        assert_eq!(fv.long_fmt, None);
    }
}

#[cfg(test)]
mod test_empty_serialization {
    use super::*;

    #[test]
    fn empty_value_serializes_as_null() {
        let empty: FormattedValue<f64> = serde_json::from_str("{}").unwrap();
        assert_eq!(serde_json::to_string(&empty).unwrap(), "null");
        assert_eq!(serde_json::to_string(&Some(empty.clone())).unwrap(), "null");
    }

    #[test]
    fn null_deserializes_to_empty_value() {
        let value: FormattedValue<f64> = serde_json::from_str("null").unwrap();
        assert_eq!(value, FormattedValue::default());
    }

    #[test]
    fn populated_value_serializes_unchanged() {
        let value = FormattedValue::with_all(1.5, "1.50".to_string(), "1.5".to_string());
        assert_eq!(
            serde_json::to_string(&value).unwrap(),
            r#"{"fmt":"1.50","longFmt":"1.5","raw":1.5}"#
        );
        let raw_only = FormattedValue::new(2.0);
        assert_eq!(serde_json::to_string(&raw_only).unwrap(), r#"{"raw":2.0}"#);
    }
}
