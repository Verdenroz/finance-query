//! Compile tests for #[derive(PyModel)].

#[test]
#[cfg(feature = "python")]
fn ui_simple() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/py_model_simple.rs");
}

#[test]
#[cfg(feature = "python")]
fn ui_collections() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/py_model_collections.rs");
}

#[test]
#[cfg(feature = "python")]
fn ui_full() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/py_model_full.rs");
}
