use crate::common::SubjectId;
use crate::event::GameEvent;
use crate::module::PlayerSpecificData;
use crate::raw_event::RawEventType;
use crate::EventMeta;
use serde::Serialize;

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

#[derive(Debug, Serialize, Default, PartialEq)]
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
        if builder.start_healing_count == 0 {
            return Self::default();
        }
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

impl PlayerSpecificData for MedicStatsBuilder {
    type Output = MedicStats;

    fn does_handle(ty: RawEventType) -> bool {
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

    fn handle_event(&mut self, meta: &EventMeta, _subject: SubjectId, event: &GameEvent) {
        match event {
            GameEvent::ChargeEnded(end) => {
                self.total_uber_length += end.duration.unwrap_or_default();
                self.last_uber_end = meta.time;
            }
            GameEvent::ChargeDeployed(_deployed) => {
                self.charge_count += 1;
            }
            GameEvent::AdvantageLost(lost) => {
                self.advantages_lost += 1;
                let time = lost.time.unwrap_or_default();
                if time > self.biggest_advantage_lost {
                    self.biggest_advantage_lost = time;
                }
            }
            GameEvent::FirstHeal(first) => {
                self.total_time_before_healing += first.time.unwrap_or_default();
                self.start_healing_count += 1;
                self.last_build_start = meta.time;
            }
            GameEvent::ChargeReady => {
                if self.last_build_start > 0 {
                    let build_time = meta.time - self.last_build_start;
                    self.last_build_start = 0;
                    self.total_time_to_build += build_time;
                    self.uber_build_count += 1;
                }
            }
            GameEvent::MedicDeath(death) => {
                let charge = death.charge.unwrap_or_default();
                if charge >= 95 && charge < 100 {
                    self.near_full_charge_death += 1;
                } else if charge >= 100 {
                    self.drops += 1;
                }
                if meta.time - self.last_uber_end <= 10 {
                    self.deaths_after_uber += 1;
                }
            }
            _ => {}
        }
    }

    fn finish(self) -> Self::Output {
        self.into()
    }
}
