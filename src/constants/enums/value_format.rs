use serde::{Deserialize, Serialize};

/// Value format for API responses
///
/// Controls how `FormattedValue<T>` fields are serialized in responses.
/// This allows API consumers to choose between raw numeric values,
/// human-readable formatted strings, or both.
/// The `alias`es mirror the shorthands [`FromStr`](std::str::FromStr) accepts, so
/// deserializing (axum query extraction, JSON) takes the same spellings parsing does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ValueFormat {
    /// Return only raw numeric values (e.g., `123.45`) - default
    /// Best for programmatic use, calculations, charts
    #[default]
    Raw,
    /// Return only formatted strings (e.g., `"$123.45"`, `"1.2B"`)
    /// Best for display purposes
    #[serde(alias = "fmt")]
    Pretty,
    /// Return both raw and formatted values
    /// Returns the full `{raw, fmt, longFmt}` object
    #[serde(alias = "full")]
    Both,
}

impl std::str::FromStr for ValueFormat {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "raw" => Ok(ValueFormat::Raw),
            "pretty" | "fmt" => Ok(ValueFormat::Pretty),
            "both" | "full" => Ok(ValueFormat::Both),
            _ => Err(()),
        }
    }
}

impl ValueFormat {
    /// Parse from string (case-insensitive), returns None on invalid input
    pub fn parse(s: &str) -> Option<Self> {
        s.parse().ok()
    }

    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            ValueFormat::Raw => "raw",
            ValueFormat::Pretty => "pretty",
            ValueFormat::Both => "both",
        }
    }

    /// Transform a JSON value based on this format
    ///
    /// Recursively processes the JSON, detecting FormattedValue objects
    /// (objects with `raw` key and optionally `fmt`/`longFmt`) and
    /// transforming them according to the format setting.
    ///
    /// # Example
    ///
    /// ```
    /// use finance_query::ValueFormat;
    /// use serde_json::json;
    ///
    /// let data = json!({"price": {"raw": 123.45, "fmt": "$123.45"}});
    ///
    /// // Raw format extracts just the raw value (default)
    /// let raw = ValueFormat::default().transform(data.clone());
    /// assert_eq!(raw, json!({"price": 123.45}));
    ///
    /// // Pretty extracts just the formatted string
    /// let pretty = ValueFormat::Pretty.transform(data.clone());
    /// assert_eq!(pretty, json!({"price": "$123.45"}));
    ///
    /// // Both keeps the full object
    /// let both = ValueFormat::Both.transform(data);
    /// assert_eq!(both, json!({"price": {"raw": 123.45, "fmt": "$123.45"}}));
    /// ```
    pub fn transform(&self, value: serde_json::Value) -> serde_json::Value {
        match self {
            ValueFormat::Both => value, // No transformation needed
            _ => self.transform_recursive(value),
        }
    }

    fn transform_recursive(&self, value: serde_json::Value) -> serde_json::Value {
        use serde_json::Value;

        match value {
            Value::Object(map) => {
                // Check if this looks like a FormattedValue (has 'raw' key)
                if self.is_formatted_value(&map) {
                    return self.extract_value(&map);
                }

                // Otherwise, recursively transform all values
                let transformed: serde_json::Map<String, Value> = map
                    .into_iter()
                    .map(|(k, v)| (k, self.transform_recursive(v)))
                    .collect();
                Value::Object(transformed)
            }
            Value::Array(arr) => Value::Array(
                arr.into_iter()
                    .map(|v| self.transform_recursive(v))
                    .collect(),
            ),
            // Primitives pass through unchanged
            other => other,
        }
    }

    /// Check if an object looks like a FormattedValue
    fn is_formatted_value(&self, map: &serde_json::Map<String, serde_json::Value>) -> bool {
        // Must have 'raw' key (can be null)
        // May have 'fmt' and/or 'longFmt'
        // Should not have many other keys (FormattedValue only has these 3)
        if !map.contains_key("raw") {
            return false;
        }

        let known_keys = ["raw", "fmt", "longFmt"];
        let unknown_keys = map
            .keys()
            .filter(|k| !known_keys.contains(&k.as_str()))
            .count();

        // If there are unknown keys, it's probably not a FormattedValue
        unknown_keys == 0
    }

    /// Extract the appropriate value based on format
    fn extract_value(&self, map: &serde_json::Map<String, serde_json::Value>) -> serde_json::Value {
        match self {
            ValueFormat::Raw => {
                // Return raw value directly (or null if not present)
                map.get("raw").cloned().unwrap_or(serde_json::Value::Null)
            }
            ValueFormat::Pretty => {
                // Prefer fmt, fall back to longFmt, then null
                map.get("fmt")
                    .or_else(|| map.get("longFmt"))
                    .cloned()
                    .unwrap_or(serde_json::Value::Null)
            }
            ValueFormat::Both => {
                // Keep as-is (shouldn't reach here, but handle anyway)
                serde_json::Value::Object(map.clone())
            }
        }
    }
}
