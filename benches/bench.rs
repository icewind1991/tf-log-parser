use chrono::NaiveDateTime;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::fs::read_to_string;
use std::time::Duration;
use tf_log_parser::{parse, EventHandler, GameEvent, LineSplit, LogHandler, RawEvent, SubjectMap};

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

pub fn handle_event(c: &mut Criterion) {
    let input = read_to_string("test_data/log_2892242.log").unwrap();
    let events: Vec<_> = input
        .split("L ")
        .filter(|line| !line.is_empty())
        .flat_map(RawEvent::parse)
        .map(|raw| (GameEvent::parse(&raw).unwrap(), raw))
        .collect();
    c.bench_function("handle events 2892242", |b| {
        let mut handler = LogHandler::default();
        let mut subjects =
            SubjectMap::<<LogHandler as EventHandler>::PerSubjectData>::with_capacity(32);
        let mut start_time: Option<NaiveDateTime> = None;
        b.iter(|| {
            black_box(&events)
                .iter()
                .flat_map(|(event, raw_event)| {
                    black_box(handler.process(&raw_event, &event, &mut start_time, &mut subjects))
                })
                .count();
        })
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default().measurement_time(Duration::from_secs(10));
    targets = parse_benchmark, parse_raw, parse_event, handle_event);
criterion_main!(benches);
