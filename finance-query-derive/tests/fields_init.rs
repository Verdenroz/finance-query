use finance_query_derive::FieldsInit;

#[derive(FieldsInit, Debug, PartialEq)]
#[non_exhaustive]
struct Plain {
    symbol: String,
    price: Option<f64>,
    volume: Option<i64>,
}

#[derive(FieldsInit, Debug, PartialEq)]
struct Generic<T: Clone, U = u8>
where
    U: Copy,
{
    value: T,
    tag: U,
    maybe: Option<T>,
}

#[derive(FieldsInit, Debug, PartialEq)]
struct Cfgd {
    always: u8,
    #[cfg(test)]
    only_test: u8,
    #[cfg(any())]
    never: NotCompiled,
}

#[test]
fn from_init_moves_every_field() {
    let built = Plain::from(PlainFieldsInit {
        symbol: "AAPL".into(),
        price: Some(1.5),
        volume: None,
    });
    assert_eq!(
        built,
        Plain {
            symbol: "AAPL".into(),
            price: Some(1.5),
            volume: None,
        }
    );
}

#[test]
fn generic_struct_keeps_params_and_defaults() {
    let built: Generic<String> = GenericFieldsInit {
        value: "a".to_string(),
        tag: 3,
        maybe: Some("b".to_string()),
    }
    .into();
    assert_eq!(built.value, "a");
    assert_eq!(built.tag, 3u8);
    assert_eq!(built.maybe.as_deref(), Some("b"));
}

#[test]
fn cfg_fields_follow_their_cfg() {
    let built = Cfgd::from(CfgdFieldsInit {
        always: 1,
        only_test: 2,
    });
    assert_eq!(built.only_test, 2);
}
