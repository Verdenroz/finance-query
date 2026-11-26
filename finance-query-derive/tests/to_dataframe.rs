use std::collections::HashMap;
use std::fmt;

use finance_query::ToDataFrame;
use finance_query::polars::prelude::{CsvWriter, DataFrame, DataType, Field, SerWriter};

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
    n: isize,
}

#[derive(ToDataFrame)]
struct Composite {
    side: Side,
    maybe_side: Option<Side>,
    #[dataframe(list)]
    tags: Vec<String>,
    #[dataframe(list)]
    maybe_tags: Option<Vec<String>>,
    plain_tags: Vec<String>,
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
        n: -13,
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
            ("n", DataType::Int64),
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
fn display_enums_marked_lists_and_flattened_structs_become_columns() {
    let row = Composite {
        side: Side::Buy,
        maybe_side: None,
        tags: vec!["x".into(), "y".into()],
        maybe_tags: None,
        plain_tags: vec!["z".into()],
        inner: inner(),
        maybe_inner: None,
        unflattened: inner(),
        opaque: NotDisplay,
        map: HashMap::new(),
        rows: vec![1.0],
    };
    let rows = [row];
    let df = Composite::vec_to_dataframe(&rows).unwrap();
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
    assert_eq!(rows[0].plain_tags.len(), 1);
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

#[derive(ToDataFrame)]
struct Holding {
    symbol: String,
    weight: Option<f64>,
}

#[derive(ToDataFrame)]
struct Fund {
    name: String,
    #[dataframe(nested)]
    holdings: Vec<Holding>,
    #[dataframe(nested)]
    maybe_holdings: Option<Vec<Holding>>,
    #[dataframe(nested)]
    top: Holding,
    #[dataframe(nested, rename = "benchmark")]
    maybe_top: Option<Holding>,
}

fn holding(symbol: &str) -> Holding {
    Holding {
        symbol: symbol.into(),
        weight: Some(0.5),
    }
}

fn fund(name: &str, holdings: usize) -> Fund {
    Fund {
        name: name.into(),
        holdings: (0..holdings).map(|i| holding(&format!("S{i}"))).collect(),
        maybe_holdings: (holdings > 1).then(|| vec![holding("M")]),
        top: holding("TOP"),
        maybe_top: (holdings > 0).then(|| holding("B")),
    }
}

fn struct_list() -> DataType {
    DataType::List(Box::new(DataType::Struct(vec![
        Field::new("symbol".into(), DataType::String),
        Field::new("weight".into(), DataType::Float64),
    ])))
}

#[test]
fn nested_fields_stay_out_of_the_flat_frame() {
    let df = fund("A", 2).to_dataframe().unwrap();
    assert_eq!(schema(&df), columns(&[("name", DataType::String)]));
}

#[test]
fn child_frames_cover_every_field_shape() {
    let parent = fund("A", 2);
    let holdings = parent.holdings_dataframe().unwrap();
    assert_eq!(holdings.height(), 2);
    assert_eq!(
        schema(&holdings),
        columns(&[("symbol", DataType::String), ("weight", DataType::Float64)])
    );
    assert_eq!(parent.maybe_holdings_dataframe().unwrap().height(), 1);
    assert_eq!(parent.top_dataframe().unwrap().height(), 1);
    assert_eq!(parent.maybe_top_dataframe().unwrap().height(), 1);
}

#[test]
fn empty_and_none_children_keep_the_child_schema() {
    let parent = fund("A", 0);
    for df in [
        parent.holdings_dataframe().unwrap(),
        parent.maybe_holdings_dataframe().unwrap(),
        parent.maybe_top_dataframe().unwrap(),
    ] {
        assert_eq!(df.height(), 0);
        assert_eq!(schema(&df), schema(&holding("x").to_dataframe().unwrap()));
    }
}

#[test]
fn nested_frame_appends_list_struct_columns_in_field_order() {
    let df = fund("A", 2).to_nested_dataframe().unwrap();
    assert_eq!(
        schema(&df),
        columns(&[
            ("name", DataType::String),
            ("holdings", struct_list()),
            ("maybe_holdings", struct_list()),
            ("top", struct_list()),
            ("benchmark", struct_list()),
        ])
    );
    assert_eq!(df.height(), 1);
    let list_len = |name: &str, row: usize| {
        df.column(name)
            .unwrap()
            .list()
            .unwrap()
            .get_as_series(row)
            .unwrap()
            .len()
    };
    assert_eq!(list_len("holdings", 0), 2);
    assert_eq!(list_len("top", 0), 1);
}

#[test]
fn empty_children_are_empty_lists_not_null() {
    let df = fund("A", 0).to_nested_dataframe().unwrap();
    for name in ["holdings", "maybe_holdings", "benchmark"] {
        let column = df.column(name).unwrap();
        assert_eq!(column.null_count(), 0, "{name}");
        assert_eq!(
            column.list().unwrap().get_as_series(0).unwrap().len(),
            0,
            "{name}"
        );
    }
}

#[test]
fn vec_to_nested_dataframe_spans_parents() {
    let parents = [fund("A", 3), fund("B", 0), fund("C", 1)];
    let df = Fund::vec_to_nested_dataframe(&parents).unwrap();
    assert_eq!(df.height(), 3);
    let lens: Vec<usize> = df
        .column("holdings")
        .unwrap()
        .list()
        .unwrap()
        .into_iter()
        .map(|list| list.unwrap().len())
        .collect();
    assert_eq!(lens, [3, 0, 1]);
    assert_eq!(
        Fund::vec_to_nested_dataframe(&[]).unwrap().width(),
        df.width()
    );
}

#[test]
fn csv_export_of_the_flat_frame_succeeds_with_nested_fields() {
    let mut df = fund("A", 2).to_dataframe().unwrap();
    let mut out = Vec::new();
    CsvWriter::new(&mut out).finish(&mut df).unwrap();
    assert!(String::from_utf8(out).unwrap().contains('A'));
}
