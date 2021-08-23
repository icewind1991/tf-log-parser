use crate::common::{Class, ClassMap, SubjectId};
use crate::event::{DamageEvent, GameEvent, RoleChangeEvent, SpawnEvent};
use crate::module::EventHandler;
use crate::raw_event::{RawEventType, RawSubject};
use crate::{EventMeta, SubjectData, SubjectMap};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Serialize, Default, PartialEq)]
pub struct ClassStats {
    kills: ClassMap<u8>,
    deaths: ClassMap<u8>,
    assists: ClassMap<u8>,
    damage: ClassMap<u16>,
}

#[derive(Default)]
pub struct ClassStatsHandler {
    active: bool,
    classes: BTreeMap<SubjectId, Class>,
    deaths: BTreeMap<SubjectId, ClassMap<u8>>,
}

impl ClassStatsHandler {
    fn get_class(&self, subject: &RawSubject) -> Option<Class> {
        subject
            .id()
            .ok()
            .and_then(|id| self.classes.get(&id))
            .copied()
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
                | RawEventType::ChangedRole
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
                self.classes.insert(subject, *class);
            }
            GameEvent::RoundStart => {
                self.active = true;
            }
            GameEvent::RoundWin(_) => {
                self.active = false;
            }
            GameEvent::Kill(kill) if self.active => {
                if let Some(target_class) = self.get_class(&kill.target) {
                    subject_data.kills[target_class] += 1;
                }
                if let Ok(target) = kill.target.id() {
                    if let Some(subject_class) = self.classes.get(&subject) {
                        self.deaths.entry(target).or_default()[*subject_class] += 1;
                    }
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
            }) if self.active => {
                if let Some(target_class) = self.get_class(target) {
                    subject_data.damage[target_class] += damage.get() as u16;
                }
            }
            _ => {}
        }
    }

    fn finish_global(self, _subjects: &SubjectMap) -> Self::GlobalOutput {
        ()
    }

    fn finish_per_subject(
        &mut self,
        subject: &SubjectData,
        mut data: Self::PerSubjectData,
    ) -> Self::PerSubjectOutput {
        data.deaths = self.deaths.remove(&subject.id()).unwrap_or_default();
        data
    }
}
