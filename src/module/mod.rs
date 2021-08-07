use crate::common::SubjectId;
use crate::event::GameEvent;
use crate::raw_event::RawEventType;
use crate::SubjectMap;
pub use chat::{ChatHandler, ChatMessage, ChatType};
pub use healspread::HealSpreadHandler;
pub use lobbysettings::{
    LobbySettingsError, LobbySettingsHandler, Location, Settings as LobbySettings,
};
pub use medicstats::{MedicStats, MedicStatsHandler};
use std::convert::Infallible;
use std::error::Error;
use std::fmt::Debug;
use thiserror::Error;

mod chat;
mod healspread;
mod lobbysettings;
mod medicstats;

pub trait EventHandler: Default {
    type Output;
    type Error: Error + Debug;

    fn does_handle(&self, ty: RawEventType) -> bool;

    fn handle(
        &mut self,
        time: u32,
        subject: SubjectId,
        event: &GameEvent,
    ) -> Result<(), Self::Error>;

    fn finish(self, subjects: &SubjectMap) -> Self::Output;
}

#[derive(Default)]
pub struct HandlerStack<Head, Tail> {
    head: Head,
    tail: Tail,
}

impl<Head: EventHandler, Tail: EventHandler> EventHandler for HandlerStack<Head, Tail> {
    type Output = (Head::Output, Tail::Output);
    type Error = EitherError<Head::Error, Tail::Error>;

    fn does_handle(&self, ty: RawEventType) -> bool {
        self.head.does_handle(ty) || self.tail.does_handle(ty)
    }

    fn handle(
        &mut self,
        time: u32,
        subject: SubjectId,
        event: &GameEvent,
    ) -> Result<(), Self::Error> {
        self.head
            .handle(time, subject, event)
            .map_err(|e| EitherError::E1(e))?;
        self.tail
            .handle(time, subject, event)
            .map_err(|e| EitherError::E2(e))?;
        Ok(())
    }

    fn finish(self, subjects: &SubjectMap) -> Self::Output {
        (self.head.finish(subjects), self.tail.finish(subjects))
    }
}

#[derive(Debug, Error)]
pub enum EitherError<E1: Error, E2: Error> {
    #[error("{0}")]
    E1(E1),
    #[error("{0}")]
    E2(E2),
}

/// A handler that doesn't stop the parsing on failure
pub enum OptionalHandler<Handler: EventHandler> {
    Active(Handler),
    Failed(Handler::Error),
}

impl<Handler: EventHandler> Default for OptionalHandler<Handler> {
    fn default() -> Self {
        OptionalHandler::Active(Handler::default())
    }
}

impl<Handler: EventHandler> EventHandler for OptionalHandler<Handler> {
    type Output = Result<Handler::Output, Handler::Error>;
    type Error = Infallible;

    fn does_handle(&self, ty: RawEventType) -> bool {
        match self {
            OptionalHandler::Active(handler) => handler.does_handle(ty),
            OptionalHandler::Failed(_) => false,
        }
    }

    fn handle(
        &mut self,
        time: u32,
        subject: SubjectId,
        event: &GameEvent,
    ) -> Result<(), Self::Error> {
        let res = if let OptionalHandler::Active(handler) = self {
            handler.handle(time, subject, event)
        } else {
            Ok(())
        };

        if let Err(e) = res {
            dbg!(&e);
            *self = OptionalHandler::Failed(e)
        }
        Ok(())
    }

    fn finish(self, subjects: &SubjectMap) -> Self::Output {
        match self {
            OptionalHandler::Active(handler) => Ok(handler.finish(subjects)),
            OptionalHandler::Failed(e) => Err(e),
        }
    }
}
