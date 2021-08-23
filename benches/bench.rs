use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::fs::read_to_string;
use std::time::Duration;
use tf_log_parser::{parse, RawEvent};

pub fn parse_benchmark(c: &mut Criterion) {
    let input = read_to_string("test_data/log_2892242.log").unwrap();
    c.bench_function("parse log 2892242", |b| b.iter(|| parse(black_box(&input))));
}

pub fn parse_raw(c: &mut Criterion) {
    let input = read_to_string("test_data/log_2892242.log").unwrap();
    c.bench_function("parse raw 2892242", |b| {
        b.iter(|| {
            black_box(&input)
                .lines()
                .filter(|line| line.starts_with("L "))
                .flat_map(RawEvent::parse)
                .count();
        })
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default().measurement_time(Duration::from_secs(10));
    targets = parse_benchmark, parse_raw);
criterion_main!(benches);
