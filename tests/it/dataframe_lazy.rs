use polars::prelude::*;

#[tokio::test(flavor = "multi_thread")]
async fn lazy_group_by_collects_inside_async_runtime() {
    let keys: Vec<i64> = (0..2000).map(|i| i % 4).collect();
    let values: Vec<i64> = (0..2000).collect();
    let df = df!("k" => keys, "v" => values).unwrap();

    let out = df
        .lazy()
        .group_by([col("k")])
        .agg([col("v").sum()])
        .sort(["k"], SortMultipleOptions::default())
        .collect()
        .unwrap();

    let sums: Vec<_> = out.column("v").unwrap().i64().unwrap().iter().collect();
    assert_eq!(
        sums,
        [Some(499_000), Some(499_500), Some(500_000), Some(500_500)]
    );
}
