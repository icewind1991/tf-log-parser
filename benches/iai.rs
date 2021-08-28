use iai::black_box;
use tf_log_parser::{parse, RawEvent};

static LOG: &str = include_str!("../test_data/log_2892242.log");

pub fn parse_benchmark() {
    black_box(parse(black_box(&LOG))).ok();
}

pub fn parse_raw() {
    black_box(
        black_box(&LOG)
            .split("L ")
            .filter(|line| !line.is_empty())
            .flat_map(RawEvent::parse)
            .count(),
    );
}

iai::main!(parse_benchmark, parse_raw);
