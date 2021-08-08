use crate::common::{SteamId3, SubjectId};
use crate::event::GameEvent;
use crate::module::EventHandler;
use crate::raw_event::RawEventType;
use crate::SubjectMap;
use std::collections::BTreeMap;
use std::convert::TryFrom;

#[derive(Default)]
pub struct HealSpreadHandler(BTreeMap<SteamId3, BTreeMap<SteamId3, u32>>);

impl EventHandler for HealSpreadHandler {
    type Output = BTreeMap<SteamId3, BTreeMap<SteamId3, u32>>;

    fn does_handle(&self, ty: RawEventType) -> bool {
        matches!(ty, RawEventType::Healed)
    }

    fn handle(&mut self, _time: u32, subject: SubjectId, event: &GameEvent) {
        let healer_steam_id = if let Some(steam_id) = subject.steam_id() {
            steam_id
        } else {
            return;
        };
        if let GameEvent::Healed(heal_event) = event {
            if let Ok(target_subject) = SubjectId::try_from(&heal_event.target) {
                if let Some(target_steam_id) = target_subject.steam_id() {
                    let healed = self
                        .0
                        .entry(SteamId3(healer_steam_id))
                        .or_default()
                        .entry(SteamId3(target_steam_id))
                        .or_default();
                    *healed += heal_event.amount
                }
            }
        }
    }

    fn finish(self, _subjects: &SubjectMap) -> Self::Output {
        self.0
    }
}
