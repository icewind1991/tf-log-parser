use crate::common::{Class, ClassMap, SubjectId};
use crate::event::{DamageEvent, GameEvent, RoleChangeEvent, SpawnEvent};
use crate::module::EventHandler;
use crate::raw_event::{RawEventType, RawSubject};
use crate::{EventMeta, SubjectData, SubjectMap};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Serialize, Default, PartialEq)]
pub struct ClassStats {
    pub kills: ClassMap<u8>,
    pub deaths: ClassMap<u8>,
    pub assists: ClassMap<u8>,
    pub damage: ClassMap<u16>,
}

#[derive(Default)]
pub struct ClassStatsHandler {
    active: bool,
    data: BTreeMap<SubjectId, ClassStatData>,
}

#[derive(Default)]
pub struct ClassStatData {
    class: Class,
    deaths: ClassMap<u8>,
}

impl ClassStatsHandler {
    fn get_class(&self, subject: &RawSubject) -> Option<Class> {
        subject
            .id()
            .ok()
            .and_then(|id| self.data.get(&id))
            .map(|data| data.class)
    }

    fn data_mut(&mut self, id: SubjectId) -> &mut ClassStatData {
        self.data.entry(id).or_default()
    }
}

impl EventHandler for ClassStatsHandler {
    type GlobalOutput = ();
    type PerSubjectData = ClassStats;
    type PerSubjectOutput = ClassStats;

    fn does_handle(ty: RawEventType) -> bool {
        matches!(
            ty,
            RawEventType::Killed
                | RawEventType::KillAssist
                | RawEventType::Damage
                | RawEventType::Spawned
                | RawEventType::RoleChange
                | RawEventType::RoundWin
                | RawEventType::RoundStart
        )
    }

    fn handle(
        &mut self,
        _meta: &EventMeta,
        subject: SubjectId,
        subject_data: &mut Self::PerSubjectData,
        event: &GameEvent,
    ) {
        match event {
            GameEvent::Spawned(SpawnEvent { class: Some(class) })
            | GameEvent::RoleChange(RoleChangeEvent { class: Some(class) }) => {
                self.data_mut(subject).class = *class;
            }
            GameEvent::RoundStart => {
                self.active = true;
            }
            GameEvent::RoundWin(_) => {
                self.active = false;
            }
            GameEvent::Killed(kill) if self.active => {
                if let Ok(target) = kill.target.id() {
                    let subject_class = self.data.get(&subject).map(|data| data.class);
                    let target_data = self.data_mut(target);
                    if let Some(subject_class) = subject_class {
                        target_data.deaths[subject_class] += 1;
                    }
                    subject_data.kills[target_data.class] += 1;
                }
            }
            GameEvent::KillAssist(assist) if self.active => {
                if let Some(target_class) = self.get_class(&assist.target) {
                    subject_data.assists[target_class] += 1;
                }
            }
            GameEvent::Damage(DamageEvent {
                damage: Some(damage),
                target,
                ..
            }) if self.active && damage > &0 && damage < &1500 => {
                if let Some(target_class) = self.get_class(target) {
                    subject_data.damage[target_class] =
                        subject_data.damage[target_class].saturating_add(*damage as u16);
                }
            }
            _ => {}
        }
    }

    fn finish_global(self, _subjects: &SubjectMap) -> Self::GlobalOutput {}

    fn finish_per_subject(
        &mut self,
        subject: &SubjectData,
        mut data: Self::PerSubjectData,
    ) -> Self::PerSubjectOutput {
        data.deaths = self.data.remove(&subject.id()).unwrap_or_default().deaths;
        data
    }
}
