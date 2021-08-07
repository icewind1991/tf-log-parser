pub use crate::common::{SteamId3, SubjectData, SubjectError, SubjectId};
use crate::event::{GameEvent, GameEventError};
use crate::module::{ChatHandler, ChatMessage, EventHandler, HealSpreadHandler};
use crate::raw_event::RawSubject;
use chrono::{DateTime, Utc};
pub use raw_event::{RawEvent, RawEventType};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};
use std::convert::TryInto;
use std::fmt::{Debug, Formatter};
use std::ops::Index;
use thiserror::Error;

mod common;
mod event;
pub mod module;
mod raw_event;

#[derive(Error)]
pub enum Error<Handler: EventHandler> {
    #[error("Malformed logfile: {0}")]
    Malformed(String),
    #[error("{0}")]
    MalformedEvent(#[from] GameEventError),
    #[error("{0}")]
    HandlerError(Handler::Error),
}

impl<Handler: EventHandler> Debug for Error<Handler> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Malformed(e) => e.fmt(f),
            Error::MalformedEvent(e) => e.fmt(f),
            Error::HandlerError(e) => e.fmt(f),
        }
    }
}

impl<Handler: EventHandler> From<SubjectError> for Error<Handler> {
    fn from(e: SubjectError) -> Self {
        Error::Malformed(e.to_string())
    }
}

impl<Handler: EventHandler> From<nom::error::Error<&'_ str>> for Error<Handler> {
    fn from(e: nom::error::Error<&str>) -> Self {
        Error::Malformed(e.to_string())
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

pub fn parse(log: &str) -> Result<LogOutput, Error<LogHandler>> {
    parse_with_handler::<LogHandler>(log)
}

pub fn parse_with_handler<Handler: EventHandler>(
    log: &str,
) -> Result<Handler::Output, Error<Handler>> {
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
                handler
                    .handle(match_time, subjects.insert(&raw_event.subject)?, &event)
                    .map_err(Error::HandlerError)?;
            }
        }
    }

    Ok(handler.finish(&subjects))
}

#[derive(Default)]
pub struct LogHandler {
    chat: ChatHandler,
    heal_spread: HealSpreadHandler,
}

#[derive(Default, Serialize)]
pub struct LogOutput {
    chat: Vec<ChatMessage>,
    heal_spread: HashMap<SteamId3, HashMap<SteamId3, u32>>,
}

#[derive(Error, Debug)]
pub enum LogError {
    #[error("{0}")]
    MalformedEvent(#[from] GameEventError),
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
        event: &GameEvent,
    ) -> Result<(), Self::Error> {
        self.chat.handle(time, subject, event).unwrap();
        self.heal_spread.handle(time, subject, event).unwrap();
        Ok(())
    }

    fn finish(self, subjects: &SubjectMap) -> Self::Output {
        LogOutput {
            chat: self.chat.finish(subjects),
            heal_spread: self.heal_spread.finish(subjects),
        }
    }
}
