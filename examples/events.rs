use main_error::MainError;
use std::env::args;
use std::fs;
use tf_log_parser::{raw_events, GameEvent};

fn main() -> Result<(), MainError> {
    let path = args().nth(1).expect("No path provided");
    let input = fs::read_to_string(path)?;

    let events: Vec<_> = raw_events(&input)
        .map(|res| res.expect("Failed to parse raw event"))
        .map(|raw| GameEvent::parse(&raw).expect("Failed to parse event"))
        .collect();

    println!("{} events parsed", events.len());

    Ok(())
}
