use crate::raw_event::RawSubject;
use serde::{Serialize, Serializer};
use std::convert::TryFrom;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use steamid_ng::{AccountType, Instance, SteamID, Universe};
use thiserror::Error;

#[derive(Serialize, Copy, Clone, Eq, PartialEq, Debug, Ord, PartialOrd)]
pub enum Team {
    Red,
    Blue,
}

impl Team {
    pub fn as_str(self) -> &'static str {
        match self {
            Team::Red => "Red",
            Team::Blue => "Blue",
        }
    }
}

impl Display for Team {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}

impl FromStr for Team {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Blue" => Ok(Team::Blue),
            "Red" => Ok(Team::Red),
            _ => Err(()),
        }
    }
}

/// Optimized subject id
#[derive(Copy, Clone, Eq, PartialEq, Debug, Ord, PartialOrd)]
pub enum SubjectId {
    Player(u32),
    Team(Team),
    System,
    World,
    Console,
}

impl SubjectId {
    pub fn steam_id(&self) -> Option<SteamID> {
        match self {
            SubjectId::Player(account_id) => Some(SteamID::new(
                *account_id,
                Instance::All,
                AccountType::Individual,
                Universe::Public,
            )),
            _ => None,
        }
    }
}

impl TryFrom<&RawSubject<'_>> for SubjectId {
    type Error = SubjectError;

    fn try_from(raw: &RawSubject) -> Result<Self, Self::Error> {
        Ok(match raw {
            RawSubject::Player { steam_id, .. } => SubjectId::Player(
                SteamID::from_steam3(steam_id)
                    .map_err(|_| SubjectError::InvalidSteamId)?
                    .account_id(),
            ),
            RawSubject::Team(team) => {
                SubjectId::Team(team.parse().map_err(|_| SubjectError::InvalidTeam)?)
            }
            RawSubject::System(_) => SubjectId::System,
            RawSubject::Console => SubjectId::Console,
            RawSubject::World => SubjectId::World,
        })
    }
}

#[derive(Debug, PartialEq)]
pub enum SubjectData {
    Player {
        name: String,
        user_id: u8,
        steam_id: SteamID,
        team: Team,
    },
    Team(Team),
    System(String),
    Console,
    World,
}

#[derive(Debug, Error)]
pub enum SubjectError {
    #[error("Invalid user id")]
    InvalidUserId,
    #[error("Invalid steam id")]
    InvalidSteamId,
    #[error("Invalid team name")]
    InvalidTeam,
}

impl TryFrom<&RawSubject<'_>> for SubjectData {
    type Error = SubjectError;

    fn try_from(raw: &RawSubject<'_>) -> Result<Self, Self::Error> {
        Ok(match raw {
            RawSubject::Player {
                name,
                user_id,
                steam_id,
                team,
            } => SubjectData::Player {
                name: name.to_string(),
                user_id: user_id.parse().map_err(|_| SubjectError::InvalidUserId)?,
                steam_id: SteamID::from_steam3(steam_id)
                    .map_err(|_| SubjectError::InvalidSteamId)?,
                team: team.parse().map_err(|_| SubjectError::InvalidTeam)?,
            },
            RawSubject::Team(team) => SubjectData::Team(team.parse().unwrap()),
            RawSubject::System(name) => SubjectData::System(name.to_string()),
            RawSubject::Console => SubjectData::Console,
            RawSubject::World => SubjectData::World,
        })
    }
}

/// Steam id formatted as steamid3 when serialized
#[derive(Debug, Hash, Eq, PartialEq)]
pub struct SteamId3(pub SteamID);

impl Serialize for SteamId3 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.steam3().serialize(serializer)
    }
}
