pub use crate::common::{SteamId3, SubjectData, SubjectError, SubjectId};
pub use crate::module::EventHandler;
use crate::module::{
    ChatMessages, ClassStatsHandler, HealSpread, MedicStatsBuilder, PlayerHandler,
};
pub use crate::subjectmap::SubjectMap;
use chrono::{Duration, NaiveDate, NaiveDateTime};
pub(crate) use error::ResultExt;
pub use error::{Error, IResult, Result};
pub use event::{Event, EventMeta, GameEvent};
use memchr::memmem::{find_iter, FindIter};
pub use raw_event::{RawEvent, RawEventType};
use std::collections::BTreeMap;
use std::convert::TryInto;
pub(crate) use tf_log_parser_derive::{Event, Events};

mod common;
pub mod event;
#[macro_use]
pub mod module;
mod error;
pub(crate) mod parsing;
pub mod raw_event;
mod subjectmap;

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

pub fn raw_events<'a>(log: &'a str) -> impl Iterator<Item = Result<RawEvent<'a>>> + 'a {
    LineSplit::new(log).map(RawEvent::parse)
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
    let mut events = raw_events(log);

    let mut handler = Handler::default();

    let mut start_time: Option<NaiveDateTime> = None;
    let mut subjects = SubjectMap::<Handler::PerSubjectData>::with_capacity(32);

    while let Some(event_res) = events.next() {
        let raw_event = match event_res {
            Ok(raw_event) => raw_event,
            Err(Error::Incomplete) if events.next().is_none() => break,
            Err(Error::Skip) => continue,
            Err(e) => return Err(e),
        };
        let should_handle = Handler::does_handle(raw_event.ty);
        if should_handle || start_time.is_none() {
            if should_handle {
                let event = match GameEvent::parse(&raw_event) {
                    Ok(event) => event,
                    Err(e) => {
                        let old_date: NaiveDateTime = raw_event
                            .date
                            .try_into()
                            .unwrap_or_else(|_| NaiveDateTime::from_timestamp(0, 0));

                        // truncated newline during log combining, ignore error
                        if contains_line_start(raw_event.params, &old_date.date()) {
                            continue;
                        }

                        let Some(next) = events.next() else {
                            // log is truncated
                            break;
                        };

                        if let Ok(next) = next {
                            let new_date: NaiveDateTime = next
                                .date
                                .try_into()
                                .unwrap_or_else(|_| NaiveDateTime::from_timestamp(0, 0));

                            // truncated lines during log combining, ignore error
                            if new_date.signed_duration_since(old_date) > Duration::seconds(60) {
                                continue;
                            }
                        }

                        return Err(e.into());
                    }
                };
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

fn contains_line_start(line: &str, date: &NaiveDate) -> bool {
    let expected_start = format!("L {}", date.format("%m/%d/%Y"));
    line.contains(&expected_start)
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
        // skip first delimiter, and any byte order mark
        let (_, input) = input.split_once("L ").unwrap_or_default();
        LineSplit {
            input,
            start: 0,
            iter: find_iter(input.as_bytes(), b"\nL "),
        }
    }
}

impl<'a> Iterator for LineSplit<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(next) => {
                let line = &self.input[self.start..next];
                self.start = next + 3;
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
