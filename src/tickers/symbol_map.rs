//! Insertion-ordered map keyed by symbol, used by the batch responses.

use serde::de::{MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::marker::PhantomData;
use std::ops::Index;

/// A map from symbol to value that iterates in insertion order.
///
/// Batch responses fill it in the order of the symbols passed to
/// [`Tickers`](crate::Tickers), so iteration follows the caller's input.
/// Entries live in a `Vec`, so lookups scan linearly, which suits batch
/// sizes and keeps large values written straight into their slot.
///
/// Serializes as a JSON object, the same shape as a `HashMap<String, V>`.
#[derive(Debug, Clone, PartialEq)]
pub struct SymbolMap<V> {
    entries: Vec<(String, V)>,
}

impl<V> SymbolMap<V> {
    /// Creates an empty map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an empty map with room for `capacity` entries.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
        }
    }

    /// Number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the map has no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns the value for `symbol`.
    pub fn get(&self, symbol: &str) -> Option<&V> {
        self.entries
            .iter()
            .find_map(|(key, value)| (key == symbol).then_some(value))
    }

    /// Returns the value for `symbol` mutably.
    pub fn get_mut(&mut self, symbol: &str) -> Option<&mut V> {
        self.entries
            .iter_mut()
            .find_map(|(key, value)| (key == symbol).then_some(value))
    }

    /// Whether the map has an entry for `symbol`.
    pub fn contains_key(&self, symbol: &str) -> bool {
        self.entries.iter().any(|(key, _)| key == symbol)
    }

    /// Sets the value for `symbol`. An existing entry keeps its position and
    /// returns the value it held; a new entry goes last.
    pub fn insert(&mut self, symbol: String, value: V) -> Option<V> {
        match self.get_mut(&symbol) {
            Some(slot) => Some(std::mem::replace(slot, value)),
            None => {
                self.entries.push((symbol, value));
                None
            }
        }
    }

    /// Removes `symbol` and returns its value. Later entries keep their order.
    pub fn remove(&mut self, symbol: &str) -> Option<V> {
        let index = self.entries.iter().position(|(key, _)| key == symbol)?;
        Some(self.entries.remove(index).1)
    }

    /// Iterates over `(symbol, value)` pairs in insertion order.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = (&String, &V)> {
        self.entries.iter().map(|(key, value)| (key, value))
    }

    /// Iterates over symbols in insertion order.
    pub fn keys(&self) -> impl ExactSizeIterator<Item = &String> {
        self.entries.iter().map(|(key, _)| key)
    }

    /// Iterates over values in insertion order.
    pub fn values(&self) -> impl ExactSizeIterator<Item = &V> {
        self.entries.iter().map(|(_, value)| value)
    }

    /// Iterates over values mutably in insertion order.
    pub fn values_mut(&mut self) -> impl ExactSizeIterator<Item = &mut V> {
        self.entries.iter_mut().map(|(_, value)| value)
    }

    /// Appends an entry without checking for `symbol`. The caller guarantees
    /// the symbol is absent.
    pub(crate) fn push(&mut self, symbol: String, value: V) {
        self.entries.push((symbol, value));
    }

    /// Reorders entries to follow `symbols`; symbols not listed go last in
    /// their current order.
    pub(crate) fn sort_by_symbols(&mut self, symbols: &[std::sync::Arc<str>]) {
        let position: std::collections::HashMap<&str, usize> = symbols
            .iter()
            .enumerate()
            .rev()
            .map(|(index, symbol)| (&**symbol, index))
            .collect();
        self.entries.sort_by_cached_key(|(key, _)| {
            position.get(key.as_str()).copied().unwrap_or(usize::MAX)
        });
    }
}

impl<V> Default for SymbolMap<V> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

/// # Panics
///
/// Panics if `symbol` is not in the map.
impl<V> Index<&str> for SymbolMap<V> {
    type Output = V;

    fn index(&self, symbol: &str) -> &V {
        match self.get(symbol) {
            Some(value) => value,
            None => panic!("no entry for symbol {symbol:?}"),
        }
    }
}

impl<V> FromIterator<(String, V)> for SymbolMap<V> {
    fn from_iter<I: IntoIterator<Item = (String, V)>>(iter: I) -> Self {
        let mut map = Self::new();
        for (symbol, value) in iter {
            map.insert(symbol, value);
        }
        map
    }
}

impl<V> IntoIterator for SymbolMap<V> {
    type Item = (String, V);
    type IntoIter = std::vec::IntoIter<(String, V)>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}

impl<'a, V> IntoIterator for &'a SymbolMap<V> {
    type Item = (&'a String, &'a V);
    type IntoIter = std::iter::Map<
        std::slice::Iter<'a, (String, V)>,
        fn(&'a (String, V)) -> (&'a String, &'a V),
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.iter().map(|(key, value)| (key, value))
    }
}

impl<V: Serialize> Serialize for SymbolMap<V> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.len()))?;
        for (symbol, value) in self {
            map.serialize_entry(symbol, value)?;
        }
        map.end()
    }
}

impl<'de, V: Deserialize<'de>> Deserialize<'de> for SymbolMap<V> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct MapVisitor<V>(PhantomData<V>);

        impl<'de, V: Deserialize<'de>> Visitor<'de> for MapVisitor<V> {
            type Value = SymbolMap<V>;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a map of symbols to values")
            }

            fn visit_map<A: MapAccess<'de>>(self, mut access: A) -> Result<Self::Value, A::Error> {
                let mut map = SymbolMap::with_capacity(access.size_hint().unwrap_or(0));
                while let Some((symbol, value)) = access.next_entry()? {
                    map.insert(symbol, value);
                }
                Ok(map)
            }
        }

        deserializer.deserialize_map(MapVisitor(PhantomData))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iterates_in_insertion_order_and_replaces_in_place() {
        let mut map = SymbolMap::new();
        map.insert("MSFT".to_string(), 1);
        map.insert("AAPL".to_string(), 2);
        assert_eq!(map.insert("MSFT".to_string(), 3), Some(1));

        assert_eq!(map.keys().collect::<Vec<_>>(), ["MSFT", "AAPL"]);
        assert_eq!(map["MSFT"], 3);
        assert_eq!(map.remove("MSFT"), Some(3));
        assert_eq!(map.keys().collect::<Vec<_>>(), ["AAPL"]);
    }

    #[test]
    fn serde_round_trips_as_json_object_in_order() {
        let map: SymbolMap<u32> = serde_json::from_str(r#"{"B":1,"A":2}"#).unwrap();

        assert_eq!(map.keys().collect::<Vec<_>>(), ["B", "A"]);
        assert_eq!(serde_json::to_string(&map).unwrap(), r#"{"B":1,"A":2}"#);
    }

    #[test]
    fn sort_by_symbols_puts_unlisted_last() {
        let mut map: SymbolMap<u32> = [("Z", 0), ("B", 1), ("A", 2)]
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();
        map.sort_by_symbols(&["A".into(), "B".into()]);

        assert_eq!(map.keys().collect::<Vec<_>>(), ["A", "B", "Z"]);
    }
}
