use std::collections::HashMap;
use std::fmt;

use finance_query::ToDataFrame;
use finance_query::polars::prelude::{DataFrame, DataType};

#[derive(Clone, Copy)]
enum Side {
    Buy,
}

impl fmt::Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Side::Buy => "buy",
        })
    }
}

struct NotDisplay;

#[derive(ToDataFrame)]
struct Inner {
    price: f64,
    note: Option<String>,
}

#[derive(ToDataFrame)]
struct Scalars {
    name: String,
    maybe_name: Option<String>,
    a: i8,
    b: i16,
    c: i32,
    d: i64,
    e: u8,
    f: u16,
    g: u32,
    h: u64,
    i: f32,
    j: f64,
    k: bool,
    l: Option<u16>,
    m: usize,
}

#[derive(ToDataFrame)]
struct Composite {
    side: Side,
    maybe_side: Option<Side>,
    tags: Vec<String>,
    maybe_tags: Option<Vec<String>>,
    #[dataframe(flatten)]
    inner: Inner,
    #[dataframe(flatten)]
    maybe_inner: Option<Inner>,
    unflattened: Inner,
    opaque: NotDisplay,
    map: HashMap<String, f64>,
    rows: Vec<f64>,
}

#[derive(ToDataFrame)]
struct Json {
    id: i64,
    json: serde_json::Value,
    maybe_json: Option<serde_json::Value>,
}

#[derive(ToDataFrame)]
struct Attributes {
    #[dataframe(rename = "ticker")]
    symbol: String,
    #[dataframe(skip)]
    side: Side,
    #[dataframe(flatten, rename = "quote")]
    inner: Inner,
}

mod renamed {
    use finance_query as fq;

    #[derive(fq::ToDataFrame)]
    #[dataframe(crate = "fq")]
    pub struct Row {
        pub value: i64,
    }
}

fn schema(df: &DataFrame) -> Vec<(String, DataType)> {
    df.schema()
        .iter()
        .map(|(name, dtype)| (name.to_string(), dtype.clone()))
        .collect()
}

fn columns(expected: &[(&str, DataType)]) -> Vec<(String, DataType)> {
    expected
        .iter()
        .map(|(name, dtype)| (name.to_string(), dtype.clone()))
        .collect()
}

fn inner() -> Inner {
    Inner {
        price: 1.5,
        note: None,
    }
}

#[test]
fn scalar_fields_map_to_matching_dtypes() {
    let row = Scalars {
        name: "a".into(),
        maybe_name: None,
        a: 1,
        b: 2,
        c: 3,
        d: 4,
        e: 5,
        f: 6,
        g: 7,
        h: 8,
        i: 9.0,
        j: 10.0,
        k: true,
        l: Some(11),
        m: 12,
    };
    let df = row.to_dataframe().unwrap();
    assert_eq!(
        schema(&df),
        columns(&[
            ("name", DataType::String),
            ("maybe_name", DataType::String),
            ("a", DataType::Int32),
            ("b", DataType::Int32),
            ("c", DataType::Int32),
            ("d", DataType::Int64),
            ("e", DataType::UInt32),
            ("f", DataType::UInt32),
            ("g", DataType::UInt32),
            ("h", DataType::UInt64),
            ("i", DataType::Float32),
            ("j", DataType::Float64),
            ("k", DataType::Boolean),
            ("l", DataType::UInt32),
            ("m", DataType::UInt64),
        ])
    );
    assert_eq!(df.height(), 1);
    assert!(
        df.columns()
            .iter()
            .all(|column| column.as_scalar_column().is_some())
    );
}

#[test]
fn display_enums_string_lists_and_flattened_structs_become_columns() {
    let row = Composite {
        side: Side::Buy,
        maybe_side: None,
        tags: vec!["x".into(), "y".into()],
        maybe_tags: None,
        inner: inner(),
        maybe_inner: None,
        unflattened: inner(),
        opaque: NotDisplay,
        map: HashMap::new(),
        rows: vec![1.0],
    };
    let df = Composite::vec_to_dataframe(&[row]).unwrap();
    let list = DataType::List(Box::new(DataType::String));
    assert_eq!(
        schema(&df),
        columns(&[
            ("side", DataType::String),
            ("maybe_side", DataType::String),
            ("tags", list.clone()),
            ("maybe_tags", list),
            ("inner_price", DataType::Float64),
            ("inner_note", DataType::String),
            ("maybe_inner_price", DataType::Float64),
            ("maybe_inner_note", DataType::String),
        ])
    );
    assert_eq!(
        df.column("side").unwrap().str().unwrap().get(0),
        Some("buy")
    );
    assert_eq!(df.column("maybe_inner_price").unwrap().null_count(), 1);
    assert_eq!(
        df.column("tags")
            .unwrap()
            .list()
            .unwrap()
            .get_as_series(0)
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn json_values_are_skipped() {
    let row = Json {
        id: 1,
        json: serde_json::json!({"a": 1}),
        maybe_json: None,
    };
    let df = row.to_dataframe().unwrap();
    assert_eq!(schema(&df), columns(&[("id", DataType::Int64)]));
    assert!(row.json.is_object() && row.maybe_json.is_none());
}

#[test]
fn skip_and_rename_attributes_apply() {
    let row = Attributes {
        symbol: "AAPL".into(),
        side: Side::Buy,
        inner: inner(),
    };
    let df = row.to_dataframe().unwrap();
    assert_eq!(
        schema(&df),
        columns(&[
            ("ticker", DataType::String),
            ("quote_price", DataType::Float64),
            ("quote_note", DataType::String),
        ])
    );
    assert!(matches!(row.side, Side::Buy));
}

#[test]
fn crate_attribute_overrides_the_facade_path() {
    let df = renamed::Row { value: 1 }.to_dataframe().unwrap();
    assert_eq!(schema(&df), columns(&[("value", DataType::Int64)]));
}

#[test]
fn trait_impl_matches_inherent_output() {
    let rows = [inner(), inner()];
    let via_trait = <Inner as ToDataFrame>::vec_to_dataframe(&rows).unwrap();
    assert!(via_trait.equals_missing(&Inner::vec_to_dataframe(&rows).unwrap()));
    assert_eq!(via_trait.height(), 2);
}

#[test]
fn empty_slice_keeps_the_schema() {
    let df = Inner::vec_to_dataframe(&[]).unwrap();
    assert_eq!(df.height(), 0);
    assert_eq!(
        schema(&df),
        columns(&[("price", DataType::Float64), ("note", DataType::String)])
    );
}
