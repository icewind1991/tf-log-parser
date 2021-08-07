use crate::module::EventHandler;
use crate::raw_event::RawEvent;
use chrono::{DateTime, Utc};
use std::convert::TryInto;
use std::fmt::{Debug, Formatter};
use thiserror::Error;

mod common;
pub mod module;
mod raw_event;

#[derive(Error)]
pub enum Error<Handler: EventHandler> {
    #[error("Malformed logfile: {0}")]
    Malformed(String),
    #[error("{0}")]
    HandlerError(Handler::Error),
}

impl<Handler: EventHandler> Debug for Error<Handler> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Malformed(e) => e.fmt(f),
            Error::HandlerError(e) => e.fmt(f),
        }
    }
}

impl<Handler: EventHandler> From<nom::error::Error<&'_ str>> for Error<Handler> {
    fn from(e: nom::error::Error<&str>) -> Self {
        Error::Malformed(e.to_string())
    }
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

    for event_res in events {
        let event = event_res?;
        if handler.does_handle(event.ty) || start_time.is_none() {
            let event_time: DateTime<Utc> = (&event.date).try_into().unwrap();
            let match_time = match start_time {
                Some(start_time) => (event_time - start_time).num_seconds() as u32,
                None => {
                    start_time = Some(event_time);
                    0
                }
            };
            handler
                .handle(match_time, (&event.subject).into(), &event)
                .map_err(Error::HandlerError)?;
        }
    }

    Ok(handler.finish())
}
