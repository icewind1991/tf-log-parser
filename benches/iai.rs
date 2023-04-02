use iai::black_box;
use std::convert::TryFrom;
use tf_log_parser::raw_event::RawSubject;
use tf_log_parser::{parse, LineSplit, RawEvent, SubjectId};

static LOG: &str = include_str!("../tests/data/log_2892242.log");

pub fn parse_benchmark() {
    black_box(parse(black_box(LOG))).ok();
}

pub fn parse_raw() {
    black_box(
        LineSplit::new(black_box(&LOG))
            .filter(|line| !line.is_empty())
            .flat_map(RawEvent::parse)
            .count(),
    );
}
pub fn subject_id() {
    let raw = black_box(RawSubject::Player("Kumis<10><[U:1:169048576]><Blue>"));
    black_box(SubjectId::try_from(&raw).unwrap());
}

iai::main!(parse_benchmark, parse_raw, subject_id);
