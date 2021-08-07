use crate::common::{SteamId3, SubjectId};
use crate::event::GameEvent;
use crate::module::EventHandler;
use crate::raw_event::RawEventType;
use crate::SubjectMap;
use serde::Serialize;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Default)]
struct MedicStatsBuilder {
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
pub struct MedicStatsHandler(HashMap<SteamId3, MedicStatsBuilder>);

impl EventHandler for MedicStatsHandler {
    type Output = HashMap<SteamId3, MedicStats>;

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

    fn handle(&mut self, time: u32, subject: SubjectId, event: &GameEvent) {
        let healer_steam_id = if let Some(steam_id) = subject.steam_id() {
            steam_id
        } else {
            return;
        };
        match event {
            GameEvent::ChargeEnded(end) => {
                let builder = self.0.entry(SteamId3(healer_steam_id)).or_default();
                builder.total_uber_length += end.duration.unwrap_or_default();
                builder.last_uber_end = time;
            }
            GameEvent::ChargeDeployed(_deployed) => {
                let builder = self.0.entry(SteamId3(healer_steam_id)).or_default();
                builder.charge_count += 1;
            }
            GameEvent::AdvantageLost(lost) => {
                let builder = self.0.entry(SteamId3(healer_steam_id)).or_default();
                builder.advantages_lost += 1;
                let time = lost.time.unwrap_or_default();
                if time > builder.biggest_advantage_lost {
                    builder.biggest_advantage_lost = time;
                }
            }
            GameEvent::FirstHeal(first) => {
                let builder = self.0.entry(SteamId3(healer_steam_id)).or_default();
                builder.total_time_before_healing += first.time.unwrap_or_default();
                builder.start_healing_count += 1;
                builder.last_build_start = time;
            }
            GameEvent::ChargeReady => {
                let builder = self.0.entry(SteamId3(healer_steam_id)).or_default();
                if builder.last_build_start > 0 {
                    let build_time = time - builder.last_build_start;
                    builder.last_build_start = 0;
                    builder.total_time_to_build += build_time;
                    builder.uber_build_count += 1;
                }
            }
            GameEvent::MedicDeath(death) => {
                let builder = self.0.entry(SteamId3(healer_steam_id)).or_default();
                let charge = death.charge.unwrap_or_default();
                if charge > 95 && charge < 100 {
                    builder.near_full_charge_death += 1;
                } else if charge >= 100 {
                    builder.drops += 1;
                }
                if time - builder.last_uber_end <= 20 {
                    builder.deaths_after_uber += 1;
                }
            }
            _ => {}
        }
    }

    fn finish(self, _subjects: &SubjectMap) -> Self::Output {
        self.0
            .into_iter()
            .filter(|(_, builder)| builder.start_healing_count > 0)
            .map(|(steam_id, builder)| (steam_id, builder.into()))
            .collect()
    }
}
