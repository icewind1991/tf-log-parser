use crate::common::{SubjectData, SubjectId};
use crate::module::EventHandler;
use crate::raw_event::{RawEvent, RawEventType};
use crate::SubjectMap;
use std::convert::Infallible;
use steamid_ng::SteamID;

struct BareChatMessage {
    pub time: u32,
    pub subject: SubjectId,
    pub message: String,
    pub chat_type: ChatType,
}

pub struct ChatMessage {
    pub time: u32,
    pub name: String,
    pub steam_id: SteamID,
    pub message: String,
    pub chat_type: ChatType,
}

impl ChatMessage {
    fn from_bare(bare: BareChatMessage, subjects: &SubjectMap) -> Self {
        let (name, steam_id) = match &subjects[bare.subject] {
            SubjectData::Player { name, steam_id, .. } => (name.clone(), steam_id.clone()),
            _ => {
                unreachable!("only player messages are added");
            }
        };
        ChatMessage {
            time: bare.time,
            name,
            steam_id,
            message: bare.message,
            chat_type: bare.chat_type,
        }
    }
}

pub enum ChatType {
    All,
    Team,
}

#[derive(Default)]
pub struct ChatHandler(Vec<BareChatMessage>);

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
            RawEventType::SayTeam => self.0.push(BareChatMessage {
                time,
                subject,
                message: event.params.trim_matches('"').into(),
                chat_type: ChatType::Team,
            }),
            RawEventType::Say => self.0.push(BareChatMessage {
                time,
                subject,
                message: event.params.trim_matches('"').into(),
                chat_type: ChatType::All,
            }),
            _ => {}
        }
        Ok(())
    }

    fn finish(self, subjects: &SubjectMap) -> Self::Output {
        self.0
            .into_iter()
            .map(|bare| ChatMessage::from_bare(bare, subjects))
            .collect()
    }
}
