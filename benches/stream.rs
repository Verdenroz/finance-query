use criterion::{Criterion, black_box, criterion_group, criterion_main};
use finance_query::streaming::{MarketHoursType, OptionType, PriceUpdate, QuoteType};

fn make_price_update() -> PriceUpdate {
    PriceUpdate {
        id: "AAPL".to_string(),
        price: 175.5_f32,
        time: 1774040720,
        currency: "USD".to_string(),
        exchange: "NMS".to_string(),
        quote_type: QuoteType::Equity,
        market_hours: MarketHoursType::RegularMarket,
        change_percent: 1.23_f32,
        day_volume: 52_345_678,
        day_high: 176.2_f32,
        day_low: 174.1_f32,
        change: 2.15_f32,
        short_name: "Apple Inc.".to_string(),
        expire_date: 0,
        open_price: 173.8_f32,
        previous_close: 173.35_f32,
        strike_price: 0.0_f32,
        underlying_symbol: String::new(),
        open_interest: 0,
        options_type: OptionType::Call,
        mini_option: 0,
        last_size: 100,
        bid: 175.48_f32,
        bid_size: 200,
        ask: 175.52_f32,
        ask_size: 300,
        price_hint: 2,
        vol_24hr: 0,
        vol_all_currencies: 0,
        from_currency: String::new(),
        last_market: String::new(),
        circulating_supply: 0.0,
        market_cap: 2_700_000_000_000.0,
    }
}

fn bench_price_update_serialize(c: &mut Criterion) {
    let update = make_price_update();
    c.bench_function("price_update_serialize", |b| {
        b.iter(|| {
            let json = serde_json::to_string(black_box(&update)).unwrap();
            black_box(json);
        })
    });
}

fn bench_price_update_deserialize(c: &mut Criterion) {
    let json = serde_json::to_string(&make_price_update()).unwrap();
    c.bench_function("price_update_deserialize", |b| {
        b.iter(|| {
            let result: PriceUpdate = serde_json::from_str(black_box(&json)).unwrap();
            black_box(result);
        })
    });
}

fn bench_price_update_roundtrip(c: &mut Criterion) {
    let update = make_price_update();
    c.bench_function("price_update_roundtrip_serialize_deserialize", |b| {
        b.iter(|| {
            let json = serde_json::to_string(black_box(&update)).unwrap();
            let result: PriceUpdate = serde_json::from_str(&json).unwrap();
            black_box(result);
        })
    });
}

fn bench_batch_price_updates_serialize(c: &mut Criterion) {
    let updates: Vec<PriceUpdate> = (0..100).map(|_| make_price_update()).collect();
    c.bench_function("batch_100_price_updates_serialize", |b| {
        b.iter(|| {
            let json = serde_json::to_string(black_box(&updates)).unwrap();
            black_box(json);
        })
    });
}

criterion_group!(
    stream_benches,
    bench_price_update_serialize,
    bench_price_update_deserialize,
    bench_price_update_roundtrip,
    bench_batch_price_updates_serialize,
);
criterion_main!(stream_benches);
