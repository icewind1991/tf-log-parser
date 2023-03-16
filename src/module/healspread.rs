use crate::common::{SteamId3, SubjectId};
use crate::event::GameEvent;
use crate::module::PlayerSpecificData;
use crate::raw_event::RawEventType;
use crate::EventMeta;
use serde::Serialize;
use std::collections::BTreeMap;
use std::convert::TryFrom;

#[derive(Default, Serialize, PartialEq)]
pub struct HealSpread(BTreeMap<SteamId3, u32>);

impl PlayerSpecificData for HealSpread {
    type Output = HealSpread;

    fn does_handle(ty: RawEventType) -> bool {
        matches!(ty, RawEventType::Healed)
    }

    fn handle_event(&mut self, _meta: &EventMeta, _subject: SubjectId, event: &GameEvent) {
        if let GameEvent::Healed(heal_event) = event {
            if let Ok(target_subject) = SubjectId::try_from(&heal_event.target) {
                if let Some(target_steam_id) = target_subject.steam_id() {
                    let healed = self.0.entry(SteamId3(target_steam_id)).or_default();
                    *healed += heal_event.amount
                }
            }
        }
    }

    fn finish(self) -> Self::Output {
        self
    }
}

impl IntoIterator for HealSpread {
    type Item = (SteamId3, u32);
    type IntoIter = <BTreeMap<SteamId3, u32> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
