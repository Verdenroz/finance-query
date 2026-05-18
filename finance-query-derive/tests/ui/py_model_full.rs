//! Tests __eq__ and to_dict emitted by #[derive(PyModel)].
//! Note: the to_dataframe path requires polars/pyo3-polars which aren't deps
//! of finance-query-derive — that's exercised in finance-query-python tests instead.

use finance_query_derive::PyModel;

#[derive(Debug, Clone, PartialEq, PyModel)]
#[py_model(eq)]
pub struct WithEq {
    pub symbol: String,
    pub price: f64,
}

#[derive(Debug, Clone, PartialEq, PyModel)]
#[py_model(rename = "MyCustomName")]
pub struct RenamedThing {
    pub value: i64,
}

fn main() {
    let a = WithEq { symbol: "AAPL".into(), price: 150.0 };
    let b = WithEq { symbol: "AAPL".into(), price: 150.0 };
    let c = WithEq { symbol: "MSFT".into(), price: 400.0 };
    let pa: PyWithEq = a.into();
    let pb: PyWithEq = b.into();
    let pc: PyWithEq = c.into();
    assert_eq!(pa, pb);
    assert_ne!(pa, pc);

    let r = RenamedThing { value: 42 };
    let _: PyRenamedThing = r.into();
}
