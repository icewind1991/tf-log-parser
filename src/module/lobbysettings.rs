use crate::common::SubjectId;
use crate::event::GameEvent;
use crate::module::GlobalData;
use crate::raw_event::RawEventType;
use crate::{EventMeta, SubjectMap};
use chrono::{DateTime, FixedOffset, NaiveDateTime, TimeZone, Utc};
use serde::{Serialize, Serializer};
use std::num::ParseIntError;
use std::str::{FromStr, ParseBoolError};
use steamid_ng::SteamID;
use thiserror::Error;

#[derive(Debug, Serialize, PartialEq)]
pub enum GameType {
    Sixes,
    Highlander,
}

impl FromStr for GameType {
    type Err = LobbySettingsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "6v6" => Ok(Self::Sixes),
            "highlander" => Ok(Self::Highlander),
            unknown => Err(LobbySettingsError::UnknownGameType(unknown.into())),
        }
    }
}

#[derive(Debug, Serialize, PartialEq)]
pub enum Location {
    Europe,
    NorthAmerica,
}

impl FromStr for Location {
    type Err = LobbySettingsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Europe" => Ok(Self::Europe),
            "North America" => Ok(Self::NorthAmerica),
            unknown => Err(LobbySettingsError::UnknownLocation(unknown.into())),
        }
    }
}

#[derive(Debug, Default, Serialize, PartialEq)]
pub struct LobbyLeader {
    name: String,
    steam_id: SteamID,
}

impl FromStr for LobbyLeader {
    type Err = LobbySettingsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((name, steam_id)) = s.rsplit_once(" (") {
            if let Ok(steam_id) = steam_id.trim_end_matches(')').parse::<u64>() {
                Ok(LobbyLeader {
                    name: name.into(),
                    steam_id: steam_id.into(),
                })
            } else {
                Err(LobbySettingsError::MalformedLeader(s.into()))
            }
        } else {
            Err(LobbySettingsError::MalformedLeader(s.into()))
        }
    }
}

#[derive(Debug, Serialize, PartialEq)]
pub struct Settings {
    id: u32,
    leader: LobbyLeader,
    map: String,
    game_type: GameType,
    location: Location,
    advanced: bool,
    region_lock: bool,
    allow_offclassing: bool,
    balancing: bool,
    restriction: String,
    mumble_required: bool,
    date: DateTime<Utc>,
    server: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self::with_id(0)
    }
}

impl Settings {
    pub fn with_id(id: u32) -> Self {
        Settings {
            id,
            leader: LobbyLeader::default(),
            map: "".to_string(),
            game_type: GameType::Sixes,
            location: Location::Europe,
            advanced: false,
            region_lock: false,
            allow_offclassing: false,
            balancing: false,
            restriction: "".to_string(),
            mumble_required: false,
            date: Utc.ymd(1, 1, 1).and_hms(0, 0, 0),
            server: "".to_string(),
        }
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum LobbySettingsError {
    #[error("Malformed lobby id: {0}")]
    InvalidLobbyId(String),
    #[error("Unknown game type: {0}")]
    UnknownGameType(String),
    #[error("Unknown location: {0}")]
    UnknownLocation(String),
    #[error("Unknown timezone in date: {0}")]
    UnknownTimezone(String),
    #[error("Malformed leader: {0}")]
    MalformedLeader(String),
    #[error("{0}")]
    InvalidBool(#[from] ParseBoolError),
    #[error("{0}")]
    InvalidInt(#[from] ParseIntError),
    #[error("{0}")]
    InvalidDate(#[from] chrono::ParseError),
}

impl Serialize for LobbySettingsError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        format!("{}", self).serialize(serializer)
    }
}

pub enum LobbySettingsHandler {
    NotAvailable,
    Active(Settings),
    Err(LobbySettingsError),
}

impl Default for LobbySettingsHandler {
    fn default() -> Self {
        LobbySettingsHandler::NotAvailable
    }
}

impl LobbySettingsHandler {
    fn try_handle(&mut self, msg: &str) -> Result<(), LobbySettingsError> {
        match self {
            LobbySettingsHandler::NotAvailable => {
                if let Some((id, _)) = msg
                    .strip_prefix("TF2Center Lobby #")
                    .and_then(|s| str::split_once(s, " |"))
                {
                    let settings = Settings::with_id(id.parse()?);
                    *self = LobbySettingsHandler::Active(settings);
                }
            }
            LobbySettingsHandler::Active(settings) => {
                if let Some((key, value)) = msg.split_once(": ") {
                    match key {
                        "Leader" => settings.leader = value.parse()?,
                        "Map" => settings.map = value.into(),
                        "GameType" => settings.game_type = value.parse()?,
                        "Location" => settings.location = value.parse()?,
                        "Advanced Lobby" => settings.advanced = value.parse()?,
                        "Region lock" => settings.region_lock = value.parse()?,
                        "Allow offclassing" => settings.allow_offclassing = value.parse()?,
                        "Balancing" => settings.balancing = value.parse()?,
                        "Restriction" => settings.restriction = value.into(),
                        "Mumble required" => settings.mumble_required = value.parse()?,
                        "Launch date" => {
                            settings.date = get_timezone(value)?
                                .from_local_datetime(&NaiveDateTime::parse_from_str(
                                    value,
                                    "%a %b %d %H:%M:%S %Z %Y",
                                )?)
                                .earliest()
                                .unwrap()
                                .into()
                        }
                        "Server" => settings.server = value.into(),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}

impl GlobalData for LobbySettingsHandler {
    type Output = Option<Result<Settings, LobbySettingsError>>;

    fn does_handle(ty: RawEventType) -> bool {
        matches!(ty, RawEventType::Say)
    }

    fn handle_event(&mut self, _meta: &EventMeta, subject: SubjectId, event: &GameEvent) {
        if !matches!(subject, SubjectId::Console) {
            return;
        }
        if let GameEvent::Say(msg) = event {
            if let Err(e) = self.try_handle(msg.text) {
                *self = LobbySettingsHandler::Err(e)
            }
        }
    }

    fn finish(self, _subjects: &SubjectMap) -> Self::Output {
        match self {
            LobbySettingsHandler::NotAvailable => None,
            LobbySettingsHandler::Active(settings) => Some(Ok(settings)),
            LobbySettingsHandler::Err(e) => Some(Err(e)),
        }
    }
}

fn get_timezone(date: &str) -> Result<FixedOffset, LobbySettingsError> {
    if date.contains("CEST") {
        Ok(FixedOffset::east(2 * 60 * 60))
    } else if date.contains("CET") {
        Ok(FixedOffset::east(60 * 60))
    } else {
        Err(LobbySettingsError::UnknownTimezone(date.into()))
    }
}
