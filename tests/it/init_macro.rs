use finance_query::{CalendarDetail, ForexQuote, init};

#[derive(Debug, Default, PartialEq)]
struct Wrapper<T> {
    inner: Option<T>,
    count: u32,
}

#[derive(Debug)]
struct NeedsBase {
    id: u8,
    name: String,
}

#[test]
fn literal_form_defaults_missing_fields() {
    let quote = init!(ForexQuote {
        symbol: "EURUSD".to_string(),
        bid: Some(1.08),
    });
    assert_eq!(quote.symbol, "EURUSD");
    assert_eq!(quote.bid, Some(1.08));
    assert_eq!(quote.ask, None);
}

#[test]
fn base_form_overrides_listed_fields() {
    let base = init!(ForexQuote {
        symbol: "EURUSD".to_string(),
        bid: Some(1.08),
    });
    let updated = init!(ForexQuote {
        price: Some(1.1),
        ..base
    });
    assert_eq!(updated.symbol, "EURUSD");
    assert_eq!(updated.bid, Some(1.08));
    assert_eq!(updated.price, Some(1.1));
}

#[test]
fn base_form_accepts_non_default_base() {
    let built = init!(NeedsBase {
        name: "x".to_string(),
        ..NeedsBase {
            id: 7,
            name: String::new()
        }
    });
    assert_eq!((built.id, built.name.as_str()), (7, "x"));
}

#[test]
fn shorthand_fields_read_locals() {
    let symbol = "GBPUSD".to_string();
    let price = Some(1.3);
    let quote = init!(ForexQuote { symbol, price });
    assert_eq!(quote.symbol, "GBPUSD");
    assert_eq!(quote.price, Some(1.3));
}

#[test]
fn empty_and_trailing_comma_forms() {
    assert_eq!(init!(ForexQuote {}).symbol, "");
    assert_eq!(init!(ForexQuote { bid: Some(1.0) }).bid, Some(1.0));
    let base = ForexQuote::default();
    assert_eq!(init!(ForexQuote { ..base }).bid, None);
}

#[test]
fn generic_struct() {
    let built = init!(Wrapper<u8> {
        inner: Some(1),
        count: 2
    });
    assert_eq!(
        built,
        Wrapper {
            inner: Some(1),
            count: 2
        }
    );
    let turbofish = init!(Wrapper::<u8> { inner: Some(3) });
    assert_eq!(turbofish.count, 0);
    let inferred = init!(Wrapper {
        count: 4,
        ..turbofish
    });
    assert_eq!((inferred.inner, inferred.count), (Some(3), 4));
}

#[test]
fn cfg_gated_fields() {
    let quote = init!(ForexQuote {
        #[cfg(any())]
        not_a_field: 1,
        #[cfg(all())]
        bid: Some(1.0),
        symbol: "X".to_string(),
    });
    assert_eq!(quote.bid, Some(1.0));

    let base = ForexQuote::default();
    let quote = init!(ForexQuote {
        #[cfg(any())]
        not_a_field: 1,
        #[cfg(all())]
        ask: Some(2.0),
        ..base
    });
    assert_eq!(quote.ask, Some(2.0));
}

#[test]
fn non_exhaustive_variant_from_blank_base() {
    let company = Some("ACME".to_string());
    let detail = init!(CalendarDetail::Earnings {
        eps: Some(1.2),
        company,
        ..CalendarDetail::earnings()
    });
    let CalendarDetail::Earnings {
        eps,
        company,
        revenue,
        ..
    } = detail
    else {
        panic!("not an earnings detail");
    };
    assert_eq!(eps, Some(1.2));
    assert_eq!(company.as_deref(), Some("ACME"));
    assert_eq!(revenue, None);
}
