use crate::common::SubjectId;
use crate::event::GameEvent;
use crate::module::EventHandler;
use crate::raw_event::RawEventType;
use crate::SubjectMap;
use chrono::{DateTime, FixedOffset, NaiveDateTime, TimeZone, Utc};
use std::str::{FromStr, ParseBoolError};
use steamid_ng::SteamID;
use thiserror::Error;

#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Debug, Default)]
pub struct LobbyLeader {
    name: String,
    steam_id: SteamID,
}

impl FromStr for LobbyLeader {
    type Err = LobbySettingsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((name, steam_id)) = s.rsplit_once(" (") {
            if let Ok(steam_id) = steam_id.trim_end_matches(")").parse::<u64>() {
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

#[derive(Debug)]
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
        Settings {
            id: 0,
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

#[derive(Debug, Error)]
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
    InvalidDate(#[from] chrono::ParseError),
}

#[derive(Default)]
pub struct LobbySettingsHandler(Settings);

impl EventHandler for LobbySettingsHandler {
    type Output = Option<Settings>;
    type Error = LobbySettingsError;

    fn does_handle(&self, ty: RawEventType) -> bool {
        matches!(ty, RawEventType::Say)
    }

    fn handle(
        &mut self,
        _time: u32,
        subject: SubjectId,
        event: &GameEvent,
    ) -> Result<(), Self::Error> {
        if !matches!(subject, SubjectId::Console) {
            return Ok(());
        }
        if let GameEvent::Say(msg) = event {
            if let Some((id, _)) = msg
                .strip_prefix("TF2Center Lobby #")
                .and_then(|s| str::split_once(s, " |"))
            {
                self.0.id = id
                    .parse()
                    .map_err(|_| LobbySettingsError::InvalidLobbyId(id.into()))?;
            }
            if let Some((key, value)) = msg.split_once(": ") {
                match key {
                    "Leader" => self.0.leader = value.parse()?,
                    "Map" => self.0.map = value.into(),
                    "GameType" => self.0.game_type = value.parse()?,
                    "Location" => self.0.location = value.parse()?,
                    "Advanced Lobby" => self.0.advanced = value.parse()?,
                    "Region lock" => self.0.region_lock = value.parse()?,
                    "Allow offclassing" => self.0.allow_offclassing = value.parse()?,
                    "Balancing" => self.0.balancing = value.parse()?,
                    "Restriction" => self.0.restriction = value.into(),
                    "Mumble required" => self.0.mumble_required = value.parse()?,
                    "Launch date" => {
                        self.0.date = get_timezone(value)?
                            .from_local_datetime(&NaiveDateTime::parse_from_str(
                                value,
                                "%a %b %d %H:%M:%S %Z %Y",
                            )?)
                            .earliest()
                            .unwrap()
                            .into()
                    }
                    "Server" => self.0.server = value.into(),
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn finish(self, _subjects: &SubjectMap) -> Self::Output {
        if self.0.id > 0 {
            Some(self.0)
        } else {
            None
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
