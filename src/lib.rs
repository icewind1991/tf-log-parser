pub use crate::common::{SteamId3, SubjectData, SubjectError, SubjectId};
use crate::event::{GameEvent, GameEventError};
use crate::module::{
    ChatHandler, ChatMessage, EventHandler, HealSpreadHandler, MedicStats, MedicStatsHandler,
};
use crate::raw_event::RawSubject;
use chrono::{DateTime, Utc};
pub use raw_event::{RawEvent, RawEventType};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};
use std::convert::TryInto;
use std::fmt::Debug;
use std::ops::Index;
use thiserror::Error;

mod common;
mod event;
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

pub fn parse(log: &str) -> Result<LogOutput, Error> {
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

#[derive(Default)]
pub struct LogHandler {
    chat: ChatHandler,
    heal_spread: HealSpreadHandler,
    medic_stats: MedicStatsHandler,
}

#[derive(Default, Serialize)]
pub struct LogOutput {
    chat: Vec<ChatMessage>,
    heal_spread: HashMap<SteamId3, HashMap<SteamId3, u32>>,
    medic_stats: HashMap<SteamId3, MedicStats>,
}

#[derive(Error, Debug)]
pub enum LogError {
    #[error("{0}")]
    MalformedEvent(#[from] GameEventError),
}

impl EventHandler for LogHandler {
    type Output = LogOutput;

    fn does_handle(&self, ty: RawEventType) -> bool {
        self.chat.does_handle(ty)
            || self.heal_spread.does_handle(ty)
            || self.medic_stats.does_handle(ty)
    }

    fn handle(&mut self, time: u32, subject: SubjectId, event: &GameEvent) {
        self.chat.handle(time, subject, event);
        self.heal_spread.handle(time, subject, event);
        self.medic_stats.handle(time, subject, event);
    }

    fn finish(self, subjects: &SubjectMap) -> Self::Output {
        LogOutput {
            chat: self.chat.finish(subjects),
            heal_spread: self.heal_spread.finish(subjects),
            medic_stats: self.medic_stats.finish(subjects),
        }
    }
}
