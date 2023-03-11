pub use crate::common::{SteamId3, SubjectData, SubjectError, SubjectId};
use crate::event::GameEventError;
pub use crate::module::EventHandler;
use crate::module::{
    ChatMessages, ClassStatsHandler, HealSpread, MedicStatsBuilder, PlayerHandler,
};
pub use crate::subjectmap::SubjectMap;
use chrono::NaiveDateTime;
pub use event::{Event, EventMeta, GameEvent};
use memchr::memmem::{find_iter, FindIter};
pub use raw_event::{RawEvent, RawEventType};
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::num::ParseIntError;
pub use tf_log_parser_derive::Event;
use thiserror::Error;

mod common;
pub mod event;
#[macro_use]
pub mod module;
pub(crate) mod parsing;
pub mod raw_event;
mod subjectmap;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Malformed logfile")]
    Malformed,
    #[error("Incomplete logfile")]
    Incomplete,
    #[error("Malformed subject: {0}")]
    Subject(#[from] SubjectError),
    #[error("{0}")]
    MalformedEvent(#[from] GameEventError),
}

impl From<ParseIntError> for Error {
    fn from(_: ParseIntError) -> Self {
        Error::Malformed
    }
}

pub type Result<O, E = Error> = std::result::Result<O, E>;

#[doc(hidden)]
pub type IResult<'a, O, E = Error> = std::result::Result<(&'a str, O), E>;

pub fn parse(
    log: &str,
) -> Result<
    (
        <LogHandler as EventHandler>::GlobalOutput,
        BTreeMap<SteamId3, <LogHandler as EventHandler>::PerSubjectOutput>,
    ),
    Error,
> {
    parse_with_handler::<LogHandler>(log)
}

pub fn parse_with_handler<Handler: EventHandler>(
    log: &str,
) -> Result<
    (
        Handler::GlobalOutput,
        BTreeMap<SteamId3, Handler::PerSubjectOutput>,
    ),
    Error,
> {
    let events = LineSplit::new(log).map(RawEvent::parse);

    let mut handler = Handler::default();

    let mut start_time: Option<NaiveDateTime> = None;
    let mut subjects = SubjectMap::<Handler::PerSubjectData>::with_capacity(32);

    for event_res in events {
        let raw_event = event_res?;
        let should_handle = Handler::does_handle(raw_event.ty);
        if should_handle || start_time.is_none() {
            if should_handle {
                let event = GameEvent::parse(&raw_event)?;
                handler.process(&raw_event, &event, &mut start_time, &mut subjects)?;
            }
        }
    }

    let just_subjects = subjects.to_just_subjects();
    let per_player = subjects
        .into_iter()
        .filter_map(|(id, subject, data)| Some((id.steam_id()?, subject, data)))
        .map(|(steam_id, subject, data)| {
            (
                SteamId3(steam_id),
                handler.finish_per_subject(&subject, data),
            )
        })
        .collect();
    let global = handler.finish_global(&just_subjects);

    Ok((global, per_player))
}

handler!(LogHandler {
    chat: ChatMessages,
    heal_spread: PlayerHandler::<HealSpread>,
    medic_stats: PlayerHandler::<MedicStatsBuilder>,
    class_stats: ClassStatsHandler,
});

pub struct LineSplit<'a> {
    input: &'a str,
    start: usize,
    iter: FindIter<'a, 'static>,
}

impl<'a> LineSplit<'a> {
    pub fn new(input: &'a str) -> Self {
        let input = &input[2..]; //skip first
        LineSplit {
            input,
            start: 0,
            iter: find_iter(input.as_bytes(), b"L "),
        }
    }
}

impl<'a> Iterator for LineSplit<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(next) => {
                let line = &self.input[self.start..next - 1]; // -1 for the newline we strip
                self.start = next + 2;
                Some(line)
            }
            None if self.start < self.input.len() => {
                let line = &self.input[self.start..];
                self.start = self.input.len();
                Some(line.trim_end_matches("\n"))
            }
            _ => None,
        }
    }
}

#[test]
fn test_split() {
    let input = std::fs::read_to_string("test_data/log_2892242.log").unwrap();
    let split: Vec<_> = LineSplit::new(&input).collect();
    let expected: Vec<_> = input
        .split("L ")
        .filter(|line| !line.is_empty())
        .map(|line| line.trim_end_matches("\n"))
        .collect();
    assert_eq!(expected.len(), split.len());
    assert_eq!(expected, split);
}
