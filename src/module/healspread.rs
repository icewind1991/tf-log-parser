use crate::common::{SteamId3, SubjectId};
use crate::event::GameEvent;
use crate::module::EventHandler;
use crate::raw_event::RawEventType;
use crate::{SubjectData, SubjectMap};
use std::collections::BTreeMap;
use std::convert::TryFrom;

#[derive(Default)]
pub struct HealSpreadHandler;

impl EventHandler for HealSpreadHandler {
    type GlobalOutput = ();
    type PerSubjectData = BTreeMap<SteamId3, u32>;
    type PerSubjectOutput = BTreeMap<SteamId3, u32>;

    fn does_handle(&self, ty: RawEventType) -> bool {
        matches!(ty, RawEventType::Healed)
    }

    fn handle(
        &mut self,
        _time: u32,
        _subject: SubjectId,
        subject_data: &mut Self::PerSubjectData,
        event: &GameEvent,
    ) {
        if let GameEvent::Healed(heal_event) = event {
            if let Ok(target_subject) = SubjectId::try_from(&heal_event.target) {
                if let Some(target_steam_id) = target_subject.steam_id() {
                    let healed = subject_data.entry(SteamId3(target_steam_id)).or_default();
                    *healed += heal_event.amount
                }
            }
        }
    }

    fn finish_global(self, _subjects: &SubjectMap) -> Self::GlobalOutput {
        ()
    }

    fn finish_per_subject(
        &self,
        _subject: &SubjectData,
        data: Self::PerSubjectData,
    ) -> Self::PerSubjectOutput {
        data
    }
}
