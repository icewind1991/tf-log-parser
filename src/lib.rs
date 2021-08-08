pub use crate::common::{SteamId3, SubjectData, SubjectError, SubjectId};
use crate::event::GameEventError;
pub use crate::module::EventHandler;
use crate::module::{ChatHandler, ClassStatsHandler, HealSpreadHandler, MedicStatsHandler};
use crate::raw_event::RawSubject;
use chrono::{DateTime, Utc};
pub use event::GameEvent;
pub use raw_event::{RawEvent, RawEventType};
use std::collections::BTreeMap;
use std::convert::TryInto;
use std::fmt::Debug;
use std::ops::Index;
use thiserror::Error;

mod common;
pub mod event;
#[macro_use]
pub mod module;
mod raw_event;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Malformed logfile: {0}")]
    Malformed(nom::error::Error<String>),
    #[error("Incomplete logfile")]
    Incomplete,
    #[error("Malformed subject: {0}")]
    Subject(#[from] SubjectError),
    #[error("{0}")]
    MalformedEvent(#[from] GameEventError),
}

impl From<nom::error::Error<&'_ str>> for Error {
    fn from(e: nom::error::Error<&str>) -> Self {
        Error::Malformed(nom::error::Error {
            input: e.input.to_string(),
            code: e.code,
        })
    }
}

#[derive(Default)]
pub struct SubjectMap(BTreeMap<SubjectId, SubjectData>);

impl Index<SubjectId> for SubjectMap {
    type Output = SubjectData;

    fn index(&self, index: SubjectId) -> &Self::Output {
        self.0
            .get(&index)
            .expect("subject id created without matching subject data")
    }
}

impl SubjectMap {
    pub fn insert(&mut self, raw: &RawSubject) -> Result<SubjectId, SubjectError> {
        let id = raw.try_into()?;
        if !self.0.contains_key(&id) {
            self.0.insert(id, raw.try_into()?);
        }
        Ok(id)
    }
}

pub fn parse(log: &str) -> Result<<LogHandler as EventHandler>::Output, Error> {
    parse_with_handler::<LogHandler>(log)
}

pub fn parse_with_handler<Handler: EventHandler>(log: &str) -> Result<Handler::Output, Error> {
    let events = log
        .lines()
        .filter(|line| line.starts_with("L "))
        .map(RawEvent::parse);

    let mut handler = Handler::default();

    let mut start_time: Option<DateTime<Utc>> = None;
    let mut subjects = SubjectMap::default();

    for event_res in events {
        let raw_event = event_res?;
        let should_handle = handler.does_handle(raw_event.ty);
        if should_handle || start_time.is_none() {
            let event_time: DateTime<Utc> = (&raw_event.date).try_into().unwrap();
            let match_time = match start_time {
                Some(start_time) => (event_time - start_time).num_seconds() as u32,
                None => {
                    start_time = Some(event_time);
                    0
                }
            };
            if should_handle {
                let event = GameEvent::parse(&raw_event)?;
                handler.handle(match_time, subjects.insert(&raw_event.subject)?, &event);
            }
        }
    }

    Ok(handler.finish(&subjects))
}

handler!(LogHandler {
    chat: ChatHandler,
    heal_spread: HealSpreadHandler,
    medic_stats: MedicStatsHandler,
    class_stats: ClassStatsHandler,
});
