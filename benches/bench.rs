use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::fs::read_to_string;
use std::time::Duration;
use tf_log_parser::{parse, GameEvent, LineSplit, RawEvent};

pub fn parse_benchmark(c: &mut Criterion) {
    let input = read_to_string("test_data/log_2892242.log").unwrap();
    c.bench_function("parse log 2892242", |b| b.iter(|| parse(black_box(&input))));
}

pub fn parse_event(c: &mut Criterion) {
    let input = read_to_string("test_data/log_2892242.log").unwrap();
    let raw: Vec<_> = input
        .split("L ")
        .filter(|line| !line.is_empty())
        .flat_map(RawEvent::parse)
        .collect();
    c.bench_function("parse event 2892242", |b| {
        b.iter(|| {
            black_box(&raw).iter().flat_map(GameEvent::parse).count();
        })
    });
}

pub fn parse_raw(c: &mut Criterion) {
    let input = read_to_string("test_data/log_2892242.log").unwrap();
    c.bench_function("parse raw 2892242", |b| {
        b.iter(|| {
            LineSplit::new(black_box(&input))
                .filter(|line| !line.is_empty())
                .flat_map(RawEvent::parse)
                .count();
        })
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default().measurement_time(Duration::from_secs(10));
    targets = parse_benchmark, parse_raw, parse_event);
criterion_main!(benches);
