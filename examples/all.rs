use main_error::MainError;
use std::env::args;
use std::fs;
use tf_log_parser::{GameEvent, RawEvent};

fn main() -> Result<(), MainError> {
    let path = args().nth(1).expect("No path provided");
    let input = fs::read_to_string(path)?;

    let events: Vec<_> = input
        .split("L ")
        .filter(|line| !line.is_empty())
        .flat_map(RawEvent::parse)
        .flat_map(|raw| GameEvent::parse(&raw))
        .collect();

    println!("{} events parsed", events.len());

    Ok(())
}