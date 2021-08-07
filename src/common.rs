use crate::raw_event::RawSubject;
use std::convert::TryFrom;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use steamid_ng::SteamID;
use thiserror::Error;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Ord, PartialOrd)]
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
    Player(u8),
    Team(Team),
    System,
    World,
    Console,
}

impl From<&RawSubject<'_>> for SubjectId {
    fn from(raw: &RawSubject) -> Self {
        match raw {
            RawSubject::Player { user_id, .. } => SubjectId::Player(user_id.parse().unwrap()),
            RawSubject::Team(team) => SubjectId::Team(team.parse().unwrap()),
            RawSubject::System(_) => SubjectId::System,
            RawSubject::Console => SubjectId::Console,
            RawSubject::World => SubjectId::World,
        }
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
