use main_error::MainError;
use serde::Serialize;
use std::collections::HashMap;
use std::env::args;
use std::fs;
use std::io::stdout;
use tf_log_parser::module::{
    ChatHandler, ChatMessage, EventHandler, HealSpreadHandler, InvalidHealEvent,
};
use tf_log_parser::{
    parse, parse_with_handler, LogHandler, RawEvent, RawEventType, SteamId3, SubjectId, SubjectMap,
};

fn main() -> Result<(), MainError> {
    let path = args().skip(1).next().expect("No path provided");
    let content = fs::read_to_string(path)?;

    let log = parse(&content)?;

    serde_json::to_writer_pretty(stdout().lock(), &log).unwrap();

    Ok(())
}
