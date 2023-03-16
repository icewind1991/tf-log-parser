use serde::Serialize;
use std::collections::BTreeMap;
use std::fs::read_to_string;
use test_case::test_case;
use tf_log_parser::module::{ClassStats, MedicStats};
use tf_log_parser::{parse, EventHandler, LogHandler, LogHandlerPerSubjectOutput};

#[derive(Serialize)]
struct LogResult {
    global: <LogHandler as EventHandler>::GlobalOutput,
    per_player: BTreeMap<String, LogPlayerData>,
}

#[derive(Serialize)]
struct LogPlayerData {
    stats: ClassStatsRaw,
    heals: BTreeMap<String, u32>,
    medic: MedicStats,
}

impl From<LogHandlerPerSubjectOutput> for LogPlayerData {
    fn from(value: LogHandlerPerSubjectOutput) -> Self {
        LogPlayerData {
            stats: value.class_stats.into(),
            medic: value.medic_stats,
            heals: value
                .heal_spread
                .into_iter()
                .map(|(user, heals)| (user.0.steam3(), heals))
                .collect(),
        }
    }
}

#[derive(Serialize)]
struct ClassStatsRaw {
    kills: [u8; 10],
    deaths: [u8; 10],
    assists: [u8; 10],
    damage: [u16; 10],
}

impl From<ClassStats> for ClassStatsRaw {
    fn from(value: ClassStats) -> Self {
        ClassStatsRaw {
            kills: value.kills.into(),
            deaths: value.deaths.into(),
            assists: value.assists.into(),
            damage: value.damage.into(),
        }
    }
}

#[test_case("log_2892242.log")]
fn test_parse(input: &str) {
    let content = read_to_string(&format!("test_data/{}", input)).unwrap();
    let (global, per_player) = parse(&content).unwrap();
    let log = LogResult {
        global,
        per_player: per_player
            .into_iter()
            .map(|(key, value)| (key.0.steam3(), value.into()))
            .collect(),
    };
    insta::assert_json_snapshot!(log);
}
