use crate::common::SubjectId;
use crate::event::GameEvent;
use crate::module::EventHandler;
use crate::raw_event::RawEventType;
use crate::{SubjectData, SubjectMap};
use serde::Serialize;
use thiserror::Error;

#[derive(Default)]
pub struct MedicStatsBuilder {
    advantages_lost: u32,
    biggest_advantage_lost: f32,
    near_full_charge_death: u32,
    deaths_after_uber: u32,
    total_time_before_healing: f32,
    start_healing_count: u32,
    total_time_to_build: u32,
    uber_build_count: u32,
    total_time_to_use: f32,
    total_uber_length: f32,
    charge_count: u32,
    last_build_start: u32,
    last_uber_end: u32,
    drops: u32,
}

#[derive(Debug, Serialize)]
pub struct MedicStats {
    advantages_lost: u32,
    biggest_advantage_lost: f32,
    near_full_charge_death: u32,
    deaths_after_uber: u32,
    avg_time_before_healing: f32,
    avg_time_to_build: f32,
    avg_time_to_use: f32,
    avg_uber_length: f32,
    charge_count: u32,
    drops: u32,
}

impl From<MedicStatsBuilder> for MedicStats {
    fn from(builder: MedicStatsBuilder) -> Self {
        MedicStats {
            advantages_lost: builder.advantages_lost,
            biggest_advantage_lost: builder.biggest_advantage_lost,
            near_full_charge_death: builder.near_full_charge_death,
            deaths_after_uber: builder.deaths_after_uber,
            avg_time_before_healing: builder.total_time_before_healing
                / builder.start_healing_count as f32,
            avg_time_to_build: builder.total_time_to_build as f32 / builder.uber_build_count as f32,
            avg_time_to_use: builder.total_time_to_use / builder.charge_count as f32,
            avg_uber_length: builder.total_uber_length / builder.charge_count as f32,
            charge_count: builder.charge_count,
            drops: builder.drops,
        }
    }
}

#[derive(Error, Debug)]
#[error("Invalid charge event: {0}")]
pub struct InvalidMedicEvent(String);

#[derive(Default)]
pub struct MedicStatsHandler;

impl EventHandler for MedicStatsHandler {
    type GlobalOutput = ();
    type PerSubjectData = MedicStatsBuilder;
    type PerSubjectOutput = MedicStats;

    fn does_handle(&self, ty: RawEventType) -> bool {
        matches!(
            ty,
            RawEventType::ChargeDeployed
                | RawEventType::ChargeEnd
                | RawEventType::ChargeReady
                | RawEventType::UberAdvantageLost
                | RawEventType::MedicDeath
                | RawEventType::FirstHealAfterSpawn
        )
    }

    fn handle(
        &mut self,
        time: u32,
        _subject: SubjectId,
        subject_data: &mut Self::PerSubjectData,
        event: &GameEvent,
    ) {
        match event {
            GameEvent::ChargeEnded(end) => {
                subject_data.total_uber_length += end.duration.unwrap_or_default();
                subject_data.last_uber_end = time;
            }
            GameEvent::ChargeDeployed(_deployed) => {
                subject_data.charge_count += 1;
            }
            GameEvent::AdvantageLost(lost) => {
                subject_data.advantages_lost += 1;
                let time = lost.time.unwrap_or_default();
                if time > subject_data.biggest_advantage_lost {
                    subject_data.biggest_advantage_lost = time;
                }
            }
            GameEvent::FirstHeal(first) => {
                subject_data.total_time_before_healing += first.time.unwrap_or_default();
                subject_data.start_healing_count += 1;
                subject_data.last_build_start = time;
            }
            GameEvent::ChargeReady => {
                if subject_data.last_build_start > 0 {
                    let build_time = time - subject_data.last_build_start;
                    subject_data.last_build_start = 0;
                    subject_data.total_time_to_build += build_time;
                    subject_data.uber_build_count += 1;
                }
            }
            GameEvent::MedicDeath(death) => {
                let charge = death.charge.unwrap_or_default();
                if charge >= 95 && charge < 100 {
                    subject_data.near_full_charge_death += 1;
                } else if charge >= 100 {
                    subject_data.drops += 1;
                }
                if time - subject_data.last_uber_end <= 10 {
                    subject_data.deaths_after_uber += 1;
                }
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
        data.into()
    }
}
