use crate::common::SubjectId;
use crate::module::EventHandler;
use crate::raw_event::{RawEvent, RawEventType};
use std::convert::Infallible;

pub struct ChatMessage {
    pub time: u32,
    pub subject: SubjectId,
    pub message: String,
    pub chat_type: ChatType,
}

pub enum ChatType {
    All,
    Team,
}

#[derive(Default)]
pub struct ChatHandler(Vec<ChatMessage>);

impl EventHandler for ChatHandler {
    type Output = Vec<ChatMessage>;
    type Error = Infallible;

    fn does_handle(&self, ty: RawEventType) -> bool {
        matches!(ty, RawEventType::SayTeam | RawEventType::Say)
    }

    fn handle(
        &mut self,
        time: u32,
        subject: SubjectId,
        event: &RawEvent,
    ) -> Result<(), Infallible> {
        if !matches!(subject, SubjectId::Player(_)) {
            return Ok(());
        }
        match event.ty {
            RawEventType::SayTeam => self.0.push(ChatMessage {
                time,
                subject,
                message: event.params.trim_matches('"').into(),
                chat_type: ChatType::Team,
            }),
            RawEventType::Say => self.0.push(ChatMessage {
                time,
                subject,
                message: event.params.trim_matches('"').into(),
                chat_type: ChatType::All,
            }),
            _ => {}
        }
        Ok(())
    }

    fn finish(self) -> Self::Output {
        self.0
    }
}
