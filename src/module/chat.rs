use crate::common::{SubjectData, SubjectId};
use crate::event::GameEvent;
use crate::module::GlobalData;
use crate::raw_event::RawEventType;
use crate::{EventMeta, SubjectMap};
use serde::Serialize;
use steamid_ng::SteamID;

struct BareChatMessage {
    pub time: u32,
    pub subject: SubjectId,
    pub message: String,
    pub chat_type: ChatType,
}

#[derive(Serialize, PartialEq)]
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
            (SubjectData::Player { name, steam_id, .. }, _) => (name.clone(), *steam_id),
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

#[derive(Serialize, PartialEq)]
pub enum ChatType {
    All,
    Team,
}

#[derive(Default)]
pub struct ChatMessages(Vec<BareChatMessage>);

impl GlobalData for ChatMessages {
    type Output = Vec<ChatMessage>;

    fn does_handle(ty: RawEventType) -> bool {
        matches!(ty, RawEventType::SayTeam | RawEventType::Say)
    }

    fn handle_event(&mut self, meta: &EventMeta, subject: SubjectId, event: &GameEvent) {
        let time = meta.time;
        if !matches!(subject, SubjectId::Player(_)) {
            return;
        }
        match event {
            GameEvent::SayTeam(message) => self.0.push(BareChatMessage {
                time,
                subject,
                message: message.to_string(),
                chat_type: ChatType::Team,
            }),
            GameEvent::Say(message) => self.0.push(BareChatMessage {
                time,
                subject,
                message: message.to_string(),
                chat_type: ChatType::All,
            }),
            _ => {}
        }
    }

    fn finish(self, subjects: &SubjectMap) -> Self::Output {
        self.0
            .into_iter()
            .map(|bare| ChatMessage::from_bare(bare, subjects))
            .collect()
    }
}
