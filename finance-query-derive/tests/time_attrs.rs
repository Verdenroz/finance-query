use finance_query::polars::prelude::{AnyValue, DataType, TimeUnit, TimeZone};
use finance_query::{FormattedValue, ToDataFrame};

fn utc() -> DataType {
    DataType::Datetime(TimeUnit::Microseconds, Some(TimeZone::UTC))
}

fn formatted(raw: Option<i64>) -> FormattedValue<i64> {
    FormattedValue {
        fmt: None,
        long_fmt: None,
        raw,
    }
}

fn micros(df: &finance_query::polars::prelude::DataFrame, name: &str) -> Vec<Option<i64>> {
    df.column(name)
        .unwrap()
        .datetime()
        .unwrap()
        .phys
        .iter()
        .collect()
}

fn days(df: &finance_query::polars::prelude::DataFrame, name: &str) -> Vec<Option<i32>> {
    df.column(name)
        .unwrap()
        .date()
        .unwrap()
        .phys
        .iter()
        .collect()
}

#[derive(ToDataFrame)]
struct Epochs {
    #[dataframe(datetime = "s")]
    seconds: i64,
    #[dataframe(datetime = "ms")]
    millis: i64,
    #[dataframe(datetime = "us")]
    micros: i64,
    #[dataframe(datetime = "ns")]
    nanos: i64,
    #[dataframe(datetime = "s")]
    maybe: Option<i64>,
    #[dataframe(datetime = "s")]
    wrapped: FormattedValue<i64>,
    #[dataframe(datetime = "s")]
    maybe_wrapped: Option<FormattedValue<i64>>,
}

fn epochs() -> Epochs {
    Epochs {
        seconds: 1_704_067_200,
        millis: 1_704_067_200_000,
        micros: 1_704_067_200_000_000,
        nanos: 1_704_067_200_000_000_000,
        maybe: Some(1),
        wrapped: formatted(Some(2)),
        maybe_wrapped: Some(formatted(None)),
    }
}

#[test]
fn every_unit_lands_on_utc_microseconds() {
    let df = epochs().to_dataframe().unwrap();
    for name in ["seconds", "millis", "micros", "nanos", "maybe", "wrapped"] {
        assert_eq!(df.column(name).unwrap().dtype(), &utc(), "{name}");
    }
    for name in ["seconds", "millis", "micros", "nanos"] {
        assert_eq!(
            micros(&df, name),
            vec![Some(1_704_067_200_000_000)],
            "{name}"
        );
    }
}

#[test]
fn option_and_formatted_epochs_read_the_raw_value() {
    let df = epochs().to_dataframe().unwrap();
    assert_eq!(micros(&df, "maybe"), vec![Some(1_000_000)]);
    assert_eq!(micros(&df, "wrapped"), vec![Some(2_000_000)]);
    assert_eq!(micros(&df, "maybe_wrapped"), vec![None]);
}

#[test]
fn none_rows_become_null() {
    let mut row = epochs();
    row.maybe = None;
    row.maybe_wrapped = None;
    let df = Epochs::vec_to_dataframe(&[row, epochs()]).unwrap();
    assert_eq!(micros(&df, "maybe"), vec![None, Some(1_000_000)]);
    assert_eq!(micros(&df, "maybe_wrapped"), vec![None, None]);
}

#[test]
fn epoch_overflow_becomes_null() {
    let mut row = epochs();
    row.seconds = i64::MAX;
    row.millis = i64::MAX;
    let df = row.to_dataframe().unwrap();
    assert_eq!(micros(&df, "seconds"), vec![None]);
    assert_eq!(micros(&df, "millis"), vec![None]);
    assert_eq!(micros(&df, "micros"), vec![Some(1_704_067_200_000_000)]);
}

#[test]
fn zero_is_a_real_instant_without_nonzero() {
    let mut row = epochs();
    row.seconds = 0;
    let df = row.to_dataframe().unwrap();
    assert_eq!(micros(&df, "seconds"), vec![Some(0)]);
}

#[derive(ToDataFrame)]
struct EpochDates {
    #[dataframe(date = "s")]
    seconds: i64,
    #[dataframe(date = "ms")]
    millis: Option<i64>,
    #[dataframe(date = "s")]
    wrapped: Option<FormattedValue<i64>>,
    #[dataframe(date = "s", nonzero)]
    expire: i64,
    #[dataframe(datetime = "ms", nonzero)]
    time: Option<i64>,
}

#[test]
fn epoch_dates_take_the_utc_calendar_day() {
    let df = EpochDates {
        seconds: 1_704_153_599,
        millis: Some(-1),
        wrapped: Some(formatted(Some(1_704_153_600))),
        expire: 86_400,
        time: Some(5),
    }
    .to_dataframe()
    .unwrap();
    for name in ["seconds", "millis", "wrapped", "expire"] {
        assert_eq!(df.column(name).unwrap().dtype(), &DataType::Date, "{name}");
    }
    assert_eq!(days(&df, "seconds"), vec![Some(19_723)]);
    assert_eq!(days(&df, "millis"), vec![Some(-1)]);
    assert_eq!(days(&df, "wrapped"), vec![Some(19_724)]);
    assert_eq!(days(&df, "expire"), vec![Some(1)]);
    assert_eq!(micros(&df, "time"), vec![Some(5_000)]);
}

#[test]
fn nonzero_turns_zero_into_null() {
    let df = EpochDates {
        seconds: 0,
        millis: None,
        wrapped: None,
        expire: 0,
        time: Some(0),
    }
    .to_dataframe()
    .unwrap();
    assert_eq!(days(&df, "seconds"), vec![Some(0)]);
    assert_eq!(days(&df, "expire"), vec![None]);
    assert_eq!(micros(&df, "time"), vec![None]);
}

#[derive(ToDataFrame)]
struct Strings {
    #[dataframe(datetime)]
    at: String,
    #[dataframe(datetime)]
    maybe_at: Option<String>,
    #[dataframe(date)]
    day: String,
    #[dataframe(date)]
    maybe_day: Option<String>,
}

fn strings(at: &str, day: &str) -> Strings {
    Strings {
        at: at.into(),
        maybe_at: Some(at.into()),
        day: day.into(),
        maybe_day: Some(day.into()),
    }
}

#[test]
fn rfc3339_strings_become_utc_instants() {
    let df = Strings::vec_to_dataframe(&[
        strings("2024-01-01T00:00:00Z", "2024-01-02"),
        strings("2024-01-01T05:30:00+05:30", "2024-01-02"),
        strings("2024-01-01T00:00:00.5-0500", "2024-01-02"),
    ])
    .unwrap();
    assert_eq!(df.column("at").unwrap().dtype(), &utc());
    assert_eq!(
        micros(&df, "at"),
        vec![
            Some(1_704_067_200_000_000),
            Some(1_704_067_200_000_000),
            Some(1_704_085_200_500_000),
        ]
    );
    assert_eq!(micros(&df, "maybe_at"), micros(&df, "at"));
}

#[test]
fn date_strings_use_the_leading_day() {
    let df = Strings::vec_to_dataframe(&[
        strings("", "2024-01-02"),
        strings("", "2024-01-02 16:30:00"),
        strings("", "2024-01-02T00:00:00.000"),
        strings("", "1969-12-31"),
    ])
    .unwrap();
    assert_eq!(df.column("day").unwrap().dtype(), &DataType::Date);
    assert_eq!(
        days(&df, "day"),
        vec![Some(19_724), Some(19_724), Some(19_724), Some(-1)]
    );
}

#[test]
fn unparseable_strings_become_null() {
    let mut row = strings("N/A", "--");
    row.maybe_at = None;
    row.maybe_day = None;
    let df = Strings::vec_to_dataframe(&[
        row,
        strings("2024-01-01 00:00:00", "2024-13-40"),
        strings("", "é2024-01-02"),
        strings("", "2024-1-2"),
        strings("", "2024-02-30"),
        strings("", ""),
    ])
    .unwrap();
    assert_eq!(micros(&df, "at").iter().flatten().count(), 0);
    assert_eq!(days(&df, "day").iter().flatten().count(), 0);
    assert_eq!(micros(&df, "maybe_at")[0], None);
    assert_eq!(days(&df, "maybe_day")[0], None);
}

#[derive(ToDataFrame)]
struct Lists {
    #[dataframe(datetime = "s", list)]
    instants: Vec<i64>,
    #[dataframe(date = "s", list)]
    days: Option<Vec<FormattedValue<i64>>>,
    #[dataframe(datetime = "ms", list, nonzero)]
    sparse: Vec<i64>,
}

#[test]
fn lists_carry_the_time_dtype() {
    let df = Lists::vec_to_dataframe(&[
        Lists {
            instants: vec![1, 2],
            days: Some(vec![formatted(Some(86_400)), formatted(None)]),
            sparse: vec![0, 7],
        },
        Lists {
            instants: vec![],
            days: None,
            sparse: vec![],
        },
    ])
    .unwrap();
    let schema = df.schema();
    assert_eq!(
        schema.get("instants"),
        Some(&DataType::List(Box::new(utc())))
    );
    assert_eq!(
        schema.get("days"),
        Some(&DataType::List(Box::new(DataType::Date)))
    );
    assert_eq!(schema.get("sparse"), Some(&DataType::List(Box::new(utc()))));

    let instants = df.column("instants").unwrap().list().unwrap();
    let first = instants.get_as_series(0).unwrap().cast(&utc()).unwrap();
    assert_eq!(first.datetime().unwrap().phys.get(1), Some(2_000_000));
    assert_eq!(instants.get_as_series(1).unwrap().len(), 0);

    let days = df.column("days").unwrap().list().unwrap();
    let first = days
        .get_as_series(0)
        .unwrap()
        .cast(&DataType::Date)
        .unwrap();
    assert_eq!(first.date().unwrap().phys.get(0), Some(1));
    assert_eq!(first.date().unwrap().phys.get(1), None);
    assert!(matches!(
        df.column("days").unwrap().get(1).unwrap(),
        AnyValue::Null
    ));

    let sparse = df.column("sparse").unwrap().list().unwrap();
    let first = sparse.get_as_series(0).unwrap().cast(&utc()).unwrap();
    assert_eq!(first.datetime().unwrap().phys.get(0), None);
    assert_eq!(first.datetime().unwrap().phys.get(1), Some(7_000));
}

#[derive(ToDataFrame)]
struct Renamed {
    #[dataframe(datetime = "s", rename = "ts")]
    timestamp: i64,
}

#[derive(ToDataFrame)]
struct Parent {
    #[dataframe(flatten)]
    child: Renamed,
}

#[test]
fn time_columns_respect_rename_and_flatten_prefix() {
    let df = Parent {
        child: Renamed { timestamp: 3 },
    }
    .to_dataframe()
    .unwrap();
    assert_eq!(df.column("child_ts").unwrap().dtype(), &utc());
    assert_eq!(micros(&df, "child_ts"), vec![Some(3_000_000)]);
}

#[test]
fn empty_frames_keep_the_time_schema() {
    let df = Strings::vec_to_dataframe(&[]).unwrap();
    assert_eq!(df.height(), 0);
    assert_eq!(df.column("at").unwrap().dtype(), &utc());
    assert_eq!(df.column("day").unwrap().dtype(), &DataType::Date);
}

#[test]
fn utc_datetime_columns_display_without_panicking() {
    let rendered = format!("{}", epochs().to_dataframe().unwrap());
    assert!(rendered.contains("2024-01-01"));
}
