//! A simple struct with primitive fields only.

use finance_query_derive::PyModel;

#[derive(Debug, Clone, PartialEq, PyModel)]
pub struct SimpleQuote {
    pub symbol: String,
    pub price: f64,
    pub volume: i64,
}

fn main() {
    let q = SimpleQuote { symbol: "AAPL".into(), price: 150.0, volume: 1_000_000 };
    let py: PySimpleQuote = q.clone().into();
    let back: SimpleQuote = py.into();
    assert_eq!(q, back);
}
