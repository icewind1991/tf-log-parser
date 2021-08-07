use main_error::MainError;
use serde::Serialize;
use std::collections::HashMap;
use std::env::args;
use std::fs;
use std::io::stdout;
use tf_log_parser::module::{
    ChatHandler, ChatMessage, EventHandler, HealSpreadHandler, InvalidHealEvent,
};
use tf_log_parser::{parse_with_handler, RawEvent, RawEventType, SteamId3, SubjectId, SubjectMap};
use thiserror::Error;

#[derive(Default)]
struct LogHandler {
    chat: ChatHandler,
    heal_spread: HealSpreadHandler,
}

#[derive(Default, Serialize)]
struct LogOutput {
    chat: Vec<ChatMessage>,
    heal_spread: HashMap<SteamId3, HashMap<SteamId3, u32>>,
}

#[derive(Error, Debug)]
enum LogError {
    #[error("{0}")]
    HealSpread(#[from] InvalidHealEvent),
}

impl EventHandler for LogHandler {
    type Output = LogOutput;
    type Error = LogError;

    fn does_handle(&self, ty: RawEventType) -> bool {
        self.chat.does_handle(ty) || self.heal_spread.does_handle(ty)
    }

    fn handle(
        &mut self,
        time: u32,
        subject: SubjectId,
        event: &RawEvent,
    ) -> Result<(), Self::Error> {
        self.chat.handle(time, subject, event).unwrap();
        self.heal_spread.handle(time, subject, event)?;
        Ok(())
    }

    fn finish(self, subjects: &SubjectMap) -> Self::Output {
        LogOutput {
            chat: self.chat.finish(subjects),
            heal_spread: self.heal_spread.finish(subjects),
        }
    }
}

fn main() -> Result<(), MainError> {
    let path = args().skip(1).next().expect("No path provided");
    let content = fs::read_to_string(path)?;

    let log = parse_with_handler::<LogHandler>(&content)?;

    serde_json::to_writer_pretty(stdout().lock(), &log).unwrap();

    Ok(())
}
