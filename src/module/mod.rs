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

mod chat;
mod healspread;
mod lobbysettings;
mod medicstats;

pub trait EventHandler: Default {
    type Output;

    fn does_handle(&self, ty: RawEventType) -> bool;

    fn handle(&mut self, time: u32, subject: SubjectId, event: &GameEvent);

    fn finish(self, subjects: &SubjectMap) -> Self::Output;
}

#[derive(Default)]
pub struct HandlerStack<Head, Tail> {
    head: Head,
    tail: Tail,
}

impl<Head: EventHandler, Tail: EventHandler> EventHandler for HandlerStack<Head, Tail> {
    type Output = (Head::Output, Tail::Output);

    fn does_handle(&self, ty: RawEventType) -> bool {
        self.head.does_handle(ty) || self.tail.does_handle(ty)
    }

    fn handle(&mut self, time: u32, subject: SubjectId, event: &GameEvent) {
        self.head.handle(time, subject, event);
        self.tail.handle(time, subject, event);
    }

    fn finish(self, subjects: &SubjectMap) -> Self::Output {
        (self.head.finish(subjects), self.tail.finish(subjects))
    }
}
