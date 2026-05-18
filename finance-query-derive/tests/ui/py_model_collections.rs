//! Tests Vec/Option/HashMap/nested handling in #[derive(PyModel)].

use finance_query_derive::PyModel;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, PyModel)]
pub struct Inner {
    pub label: String,
    pub value: f64,
}

#[derive(Debug, Clone, PartialEq, PyModel)]
pub struct Outer {
    pub name: String,
    pub items: Vec<f64>,
    pub maybe: Option<i64>,
    pub tags: HashMap<String, String>,
    pub nested: Inner,
    pub nested_opt: Option<Inner>,
    pub nested_vec: Vec<Inner>,
    pub nested_map: HashMap<String, Inner>,
}

fn main() {
    let inner = Inner { label: "x".into(), value: 1.0 };
    let outer = Outer {
        name: "outer".into(),
        items: vec![1.0, 2.0, 3.0],
        maybe: Some(42),
        tags: HashMap::from([("k".to_string(), "v".to_string())]),
        nested: inner.clone(),
        nested_opt: Some(inner.clone()),
        nested_vec: vec![inner.clone(), inner.clone()],
        nested_map: HashMap::from([("k".to_string(), inner.clone())]),
    };
    let py: PyOuter = outer.clone().into();
    let back: Outer = py.into();
    assert_eq!(outer, back);
}
