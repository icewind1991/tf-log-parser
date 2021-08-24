use crate::raw_event::{split_player_subject, RawSubject};
use enum_iterator::IntoEnumIterator;
use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};
use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Index, IndexMut};
use std::str::FromStr;
use steamid_ng::{AccountType, Instance, SteamID, Universe};
use thiserror::Error;

#[derive(Serialize, Copy, Clone, Eq, PartialEq, Debug, Ord, PartialOrd, Hash)]
pub enum Team {
    Red,
    Blue,
    Spectator,
}

impl Team {
    pub fn as_str(self) -> &'static str {
        match self {
            Team::Red => "Red",
            Team::Blue => "Blue",
            Team::Spectator => "Spectator",
        }
    }
}

impl Display for Team {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <str as Debug>::fmt(self.as_str(), f)
    }
}

impl FromStr for Team {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Blue" => Ok(Team::Blue),
            "Red" => Ok(Team::Red),
            "Spectator" => Ok(Team::Spectator),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, IntoEnumIterator, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Class {
    Scout,
    Soldier,
    Pyro,
    DemoMan,
    HeavyWeapons,
    Engineer,
    Medic,
    Sniper,
    Spy,
}

impl Class {
    pub fn as_str(self) -> &'static str {
        match self {
            Class::Scout => "scout",
            Class::Soldier => "soldier",
            Class::Pyro => "pyro",
            Class::DemoMan => "demoman",
            Class::HeavyWeapons => "heavyweapons",
            Class::Engineer => "engineer",
            Class::Medic => "medic",
            Class::Sniper => "sniper",
            Class::Spy => "spy",
        }
    }
}

impl FromStr for Class {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "scout" => Ok(Class::Scout),
            "soldier" => Ok(Class::Soldier),
            "pyro" => Ok(Class::Pyro),
            "demoman" => Ok(Class::DemoMan),
            "heavyweapons" => Ok(Class::HeavyWeapons),
            "engineer" => Ok(Class::Engineer),
            "medic" => Ok(Class::Medic),
            "sniper" => Ok(Class::Sniper),
            "spy" => Ok(Class::Spy),
            _ => Err(()),
        }
    }
}

pub struct ClassMap<T>([T; 9]);

impl<T> Index<Class> for ClassMap<T> {
    type Output = T;

    fn index(&self, index: Class) -> &Self::Output {
        match index {
            Class::Scout => &self.0[0],
            Class::Soldier => &self.0[1],
            Class::Pyro => &self.0[2],
            Class::DemoMan => &self.0[3],
            Class::HeavyWeapons => &self.0[4],
            Class::Engineer => &self.0[5],
            Class::Medic => &self.0[6],
            Class::Sniper => &self.0[7],
            Class::Spy => &self.0[8],
        }
    }
}

impl<T> IndexMut<Class> for ClassMap<T> {
    fn index_mut(&mut self, index: Class) -> &mut Self::Output {
        match index {
            Class::Scout => &mut self.0[0],
            Class::Soldier => &mut self.0[1],
            Class::Pyro => &mut self.0[2],
            Class::DemoMan => &mut self.0[3],
            Class::HeavyWeapons => &mut self.0[4],
            Class::Engineer => &mut self.0[5],
            Class::Medic => &mut self.0[6],
            Class::Sniper => &mut self.0[7],
            Class::Spy => &mut self.0[8],
        }
    }
}

impl<T> Serialize for ClassMap<T>
where
    T: Serialize + Default + PartialEq,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        for class in Class::into_enum_iter() {
            let stats = &self[class];
            if stats != &T::default() {
                map.serialize_entry(&class, stats)?;
            }
        }
        map.end()
    }
}

impl<T: Default> Default for ClassMap<T> {
    fn default() -> Self {
        ClassMap(<[T; 9]>::default())
    }
}

impl<T: Debug> Debug for ClassMap<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <[T; 9] as Debug>::fmt(&self.0, f)
    }
}

impl<T: PartialEq> PartialEq for ClassMap<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

/// Optimized subject id
#[derive(Copy, Clone, Eq, PartialEq, Debug, Ord, PartialOrd, Hash)]
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
                Instance::Desktop,
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
            RawSubject::Player(raw) => {
                if let Some(raw_account_id) = raw
                    .rsplit_once(":")
                    .map(|(_, s)| s)
                    .and_then(|s| s.split_once(']'))
                    .map(|(s, _)| s)
                {
                    SubjectId::Player(
                        raw_account_id
                            .parse()
                            .map_err(|_| SubjectError::InvalidSteamId)?,
                    )
                } else {
                    return Err(SubjectError::InvalidSteamId);
                }
            }
            RawSubject::Team(team) => {
                SubjectId::Team(team.parse().map_err(|_| SubjectError::InvalidTeam)?)
            }
            RawSubject::System(_) => SubjectId::System,
            RawSubject::Console => SubjectId::Console,
            RawSubject::World => SubjectId::World,
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
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

impl SubjectData {
    pub fn id(&self) -> SubjectId {
        match self {
            SubjectData::Player { steam_id, .. } => SubjectId::Player(steam_id.account_id()),
            SubjectData::Team(team) => SubjectId::Team(*team),
            SubjectData::System(_) => SubjectId::System,
            SubjectData::Console => SubjectId::Console,
            SubjectData::World => SubjectId::World,
        }
    }
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
            RawSubject::Player(raw) => {
                let (_, (name, user_id, steam_id, team)) =
                    split_player_subject(raw).map_err(|_| SubjectError::InvalidUserId)?;
                SubjectData::Player {
                    name: name.to_string(),
                    user_id: user_id.parse().map_err(|_| SubjectError::InvalidUserId)?,
                    steam_id: SteamID::from_steam3(steam_id)
                        .map_err(|_| SubjectError::InvalidSteamId)?,
                    team: team.parse().map_err(|_| SubjectError::InvalidTeam)?,
                }
            }
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

impl PartialOrd<Self> for SteamId3 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        (u64::from(self.0)).partial_cmp(&u64::from(other.0))
    }
}

impl Ord for SteamId3 {
    fn cmp(&self, other: &Self) -> Ordering {
        (u64::from(self.0)).cmp(&u64::from(other.0))
    }
}

impl From<SteamID> for SteamId3 {
    fn from(id: SteamID) -> Self {
        SteamId3(id)
    }
}

impl Serialize for SteamId3 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.steam3().serialize(serializer)
    }
}
