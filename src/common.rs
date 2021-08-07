use crate::raw_event::RawSubject;
use std::str::FromStr;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Team {
    Red,
    Blue,
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
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
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
