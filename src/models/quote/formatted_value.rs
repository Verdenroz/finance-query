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
use serde::{Deserialize, Serialize};

/// A generic type representing Yahoo Finance's formatted value pattern
///
/// Contains the raw numeric value along with optional formatted representations.
/// Note: `raw` is optional because Yahoo sometimes returns empty objects `{}` for unavailable data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FormattedValue<T> {
    /// Human-readable formatted string
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fmt: Option<String>,

    /// Long format (for large numbers with full precision)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub long_fmt: Option<String>,

    /// Raw numeric value (None if data is unavailable)
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw: Option<T>,
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
