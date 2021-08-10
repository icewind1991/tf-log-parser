use crate::common::{Class, ClassMap, SteamId3, SubjectId};
use crate::event::{DamageEvent, GameEvent, RoleChangeEvent, SpawnEvent};
use crate::module::EventHandler;
use crate::raw_event::{RawEventType, RawSubject};
use crate::{SubjectData, SubjectMap};
use serde::Serialize;
use std::collections::BTreeMap;
use std::ops::{Add, AddAssign};

#[derive(Debug, Serialize, Default, PartialEq)]
pub struct ClassStat {
    kills: u8,
    deaths: u8,
    assists: u8,
    damage: u16,
}

impl Add for ClassStat {
    type Output = ClassStat;

    fn add(self, rhs: Self) -> Self::Output {
        ClassStat {
            kills: self.kills + rhs.kills,
            deaths: self.deaths + rhs.deaths,
            assists: self.assists + rhs.assists,
            damage: self.damage + rhs.damage,
        }
    }
}

impl AddAssign for ClassStat {
    fn add_assign(&mut self, rhs: Self) {
        self.kills += rhs.kills;
        self.deaths += rhs.deaths;
        self.assists += rhs.assists;
        self.damage += rhs.damage;
    }
}

#[derive(Default)]
pub struct ClassStatsHandler {
    active: bool,
    classes: BTreeMap<SubjectId, Class>,
    stats: BTreeMap<SteamId3, ClassMap<ClassStat>>,
}

impl ClassStatsHandler {
    fn handle_stats(&mut self, subject: SubjectId, target: &RawSubject, stats: ClassStat) {
        if let Ok(target) = target.id() {
            self.handle_stats_id(subject, target, stats)
        }
    }

    fn handle_stats_id(&mut self, subject: SubjectId, target: SubjectId, stats: ClassStat) {
        let subject = if let Some(steam_id) = subject.steam_id() {
            steam_id
        } else {
            return;
        };

        if let Some(target_class) = self.classes.get(&target) {
            self.stats.entry(SteamId3(subject)).or_default()[*target_class] += stats;
        }
    }
}

impl EventHandler for ClassStatsHandler {
    type GlobalOutput = ();
    type PerSubjectData = ClassMap<ClassStat>;
    type PerSubjectOutput = ClassMap<ClassStat>;

    fn does_handle(&self, ty: RawEventType) -> bool {
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
        _time: u32,
        subject: SubjectId,
        _subject_data: &mut Self::PerSubjectData,
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
                self.handle_stats(
                    subject,
                    &kill.target,
                    ClassStat {
                        kills: 1,
                        ..Default::default()
                    },
                );
                if let Ok(target) = kill.target.id() {
                    self.handle_stats_id(
                        target,
                        subject,
                        ClassStat {
                            deaths: 1,
                            ..Default::default()
                        },
                    );
                }
            }
            GameEvent::KillAssist(kill) if self.active => {
                self.handle_stats(
                    subject,
                    &kill.target,
                    ClassStat {
                        assists: 1,
                        ..Default::default()
                    },
                );
            }
            GameEvent::Damage(DamageEvent {
                damage: Some(damage),
                target,
                ..
            }) if self.active => {
                self.handle_stats(
                    subject,
                    target,
                    ClassStat {
                        damage: damage.get() as u16,
                        ..Default::default()
                    },
                );
            }
            _ => {}
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
