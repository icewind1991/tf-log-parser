use flate2::read::GzDecoder;
use std::fs::File;
use std::io::Read;
use test_case::test_case;
use tf_log_parser::{LineSplit, RawEvent};

#[test_case("log_6s.log")]
#[test_case("log_2788889.log")]
#[test_case("log_2892242.log")]
#[test_case("log_bball.log")]
#[test_case("log_hl.log")]
fn smoke_test(name: &str) {
    let path = format!("tests/data/{}.gz", name);
    let mut content = String::new();
    GzDecoder::new(File::open(path).expect("failed to open"))
        .read_to_string(&mut content)
        .expect("failed to read");
    for line in LineSplit::new(&content) {
        RawEvent::parse(line).expect("failed to parse raw event");
    }
}
