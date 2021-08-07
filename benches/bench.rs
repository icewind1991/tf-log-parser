use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::fs::read_to_string;
use tf_log_parser::parse;

pub fn parse_benchmark(c: &mut Criterion) {
    let input = read_to_string("test_data/log_2892242.log").unwrap();
    c.bench_function("parse 2892242", |b| b.iter(|| parse(black_box(&input))));
}

criterion_group!(benches, parse_benchmark);
criterion_main!(benches);
