mod game;
mod medic;
mod player;

use crate::event::game::{RoundLengthEvent, RoundWinEvent};
use crate::raw_event::{against_subject_parser, RawSubject};
use crate::{IResult, RawEvent, RawEventType, SubjectId};
pub use game::*;
pub use medic::*;
use nom::bytes::complete::{tag, take_while};
use nom::character::complete::{alpha1, digit1};
use nom::combinator::opt;
use nom::error::{ErrorKind, ParseError};
use nom::number::complete::float;
use nom::Err;
pub use player::*;
use std::net::SocketAddr;
use std::num::NonZeroU32;
use std::str::FromStr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GameEventError {
    #[error("malformed game event({ty:?}): {err}")]
    Error {
        err: nom::error::Error<String>,
        ty: RawEventType,
    },
    #[error("incomplete event body({0:?})")]
    Incomplete(RawEventType),
}

trait GameEventErrTrait<T> {
    fn with_type(self, ty: RawEventType) -> Result<T, GameEventError>;
}

impl<'a, T> GameEventErrTrait<T> for IResult<'a, T> {
    fn with_type(self, ty: RawEventType) -> Result<T, GameEventError> {
        self.map_err(|err| match err {
            nom::Err::Error(e) | nom::Err::Failure(e) => GameEventError::Error {
                err: nom::error::Error {
                    input: e.input.to_string(),
                    code: e.code,
                },
                ty,
            },

            Err::Incomplete(_) => GameEventError::Incomplete(ty),
        })
        .map(|(_rest, t)| t)
    }
}

pub trait Event<'a>: Sized + 'a {
    fn parse(input: &'a str) -> IResult<Self>;
}

fn parse_event<'a, T: Event<'a>>(input: &'a str) -> IResult<T> {
    T::parse(input)
}

#[derive(Debug)]
pub struct EventMeta {
    pub time: u32,
    pub subject: SubjectId,
}

#[derive(Debug)]
pub enum GameEvent<'a> {
    ShotFired(ShotFiredEvent<'a>),
    ShotHit(ShotHitEvent<'a>),
    Damage(DamageEvent<'a>),
    Kill(KillEvent<'a>),
    KillAssist(KillAssistEvent<'a>),
    Say(&'a str),
    SayTeam(&'a str),
    Healed(HealedEvent<'a>),
    ChargeDeployed(ChargeDeployedEvent<'a>),
    ChargeEnded(ChargeEndedEvent),
    AdvantageLost(AdvantageLostEvent),
    FirstHeal(FirstHealEvent),
    ChargeReady,
    MedicDeath(MedicDeathEvent),
    MedicDeathEx(MedicDeathExEvent),
    Spawned(SpawnEvent),
    RoleChange(RoleChangeEvent),
    RoundStart,
    RoundWin(RoundWinEvent<'a>),
    RoundLength(RoundLengthEvent),
    RoundOverTime,
    LogFileStarted(LogFileStartedEvent<'a>),
    Connected(ConnectedEvent),
    Disconnect(DisconnectEvent<'a>),
    SteamIdValidated,
    Entered,
    Joined(JoinedTeamEvent),
    Suicide(CommittedSuicideEvent<'a>),
    PickedUp(PickedUpEvent<'a>),
    Domination(DominationEvent<'a>),
    EmptyUber,
    Revenge(RevengeEvent<'a>),
    TournamentModeStarted(TournamentModeStartedEvent<'a>),
    CaptureBlocked(CaptureBlockedEvent<'a>),
    PointCaptured(PointCapturedEvent<'a>),
    CurrentScore(CurrentScoreEvent),
    BuiltObject(BuiltObjectEvent<'a>),
    KilledObject(KilledObjectEvent<'a>),
    Extinguished(ExtinguishedEvent<'a>),
    GameOver(GameOverEvent<'a>),
    FinalScore(FinalScoreEvent),
    ObjectDetonated(ObjectDetonatedEvent<'a>),
    LogFileClosed,
}

impl<'a> GameEvent<'a> {
    pub fn parse(raw: &RawEvent<'a>) -> Result<GameEvent<'a>, GameEventError> {
        Ok(match raw.ty {
            RawEventType::ShotFired => {
                GameEvent::ShotFired(parse_event(raw.params).with_type(raw.ty)?)
            }
            RawEventType::ShotHit => GameEvent::ShotHit(parse_event(raw.params).with_type(raw.ty)?),
            RawEventType::Damage => GameEvent::Damage(parse_event(raw.params).with_type(raw.ty)?),
            RawEventType::Killed => GameEvent::Kill(parse_event(raw.params).with_type(raw.ty)?),
            RawEventType::KillAssist => {
                GameEvent::KillAssist(parse_event(raw.params).with_type(raw.ty)?)
            }
            RawEventType::SayTeam => GameEvent::SayTeam(raw.params.trim_matches('"')),
            RawEventType::Say => GameEvent::Say(raw.params.trim_matches('"')),
            RawEventType::Healed => GameEvent::Healed(parse_event(raw.params).with_type(raw.ty)?),
            RawEventType::ChargeDeployed => {
                GameEvent::ChargeDeployed(parse_event(raw.params).with_type(raw.ty)?)
            }
            RawEventType::ChargeEnd => {
                GameEvent::ChargeEnded(parse_event(raw.params).with_type(raw.ty)?)
            }
            RawEventType::UberAdvantageLost => {
                GameEvent::AdvantageLost(parse_event(raw.params).with_type(raw.ty)?)
            }
            RawEventType::FirstHealAfterSpawn => {
                GameEvent::FirstHeal(parse_event(raw.params).with_type(raw.ty)?)
            }
            RawEventType::ChargeReady => GameEvent::ChargeReady,
            RawEventType::MedicDeath => {
                GameEvent::MedicDeath(parse_event(raw.params).with_type(raw.ty)?)
            }
            RawEventType::MedicDeathEx => {
                GameEvent::MedicDeathEx(parse_event(raw.params).with_type(raw.ty)?)
            }
            RawEventType::Spawned => GameEvent::Spawned(parse_event(raw.params).with_type(raw.ty)?),
            RawEventType::ChangedRole => {
                GameEvent::RoleChange(parse_event(raw.params).with_type(raw.ty)?)
            }
            RawEventType::RoundStart => GameEvent::RoundStart,
            RawEventType::RoundLength => {
                GameEvent::RoundLength(parse_event(raw.params).with_type(raw.ty)?)
            }
            RawEventType::RoundWin => {
                GameEvent::RoundWin(parse_event(raw.params).with_type(raw.ty)?)
            }
            RawEventType::RoundOvertime => GameEvent::RoundOverTime,
            RawEventType::LogFileStarted => {
                GameEvent::LogFileStarted(parse_event(raw.params).with_type(raw.ty)?)
            }
            RawEventType::Connected => {
                GameEvent::Connected(parse_event(raw.params).with_type(raw.ty)?)
            }
            RawEventType::Disconnected => {
                GameEvent::Disconnect(parse_event(raw.params).with_type(raw.ty)?)
            }
            RawEventType::SteamIdValidated => GameEvent::SteamIdValidated,
            RawEventType::Entered => GameEvent::Entered,
            RawEventType::Joined => GameEvent::Joined(parse_event(raw.params).with_type(raw.ty)?),
            RawEventType::Suicide => GameEvent::Suicide(parse_event(raw.params).with_type(raw.ty)?),
            RawEventType::PickedUp => {
                GameEvent::PickedUp(parse_event(raw.params).with_type(raw.ty)?)
            }
            RawEventType::Domination => {
                GameEvent::Domination(parse_event(raw.params).with_type(raw.ty)?)
            }
            RawEventType::EmptyUber => GameEvent::EmptyUber,
            RawEventType::Revenge => GameEvent::Revenge(parse_event(raw.params).with_type(raw.ty)?),
            RawEventType::TournamentStart => {
                GameEvent::TournamentModeStarted(parse_event(raw.params).with_type(raw.ty)?)
            }
            RawEventType::CaptureBlocked => {
                GameEvent::CaptureBlocked(parse_event(raw.params).with_type(raw.ty)?)
            }
            RawEventType::PointCaptured => {
                GameEvent::PointCaptured(parse_event(raw.params).with_type(raw.ty)?)
            }
            RawEventType::CurrentScore => {
                GameEvent::CurrentScore(parse_event(raw.params).with_type(raw.ty)?)
            }
            RawEventType::PlayerBuiltObject => {
                GameEvent::BuiltObject(parse_event(raw.params).with_type(raw.ty)?)
            }
            RawEventType::PlayerKilledObject => {
                GameEvent::KilledObject(parse_event(raw.params).with_type(raw.ty)?)
            }
            RawEventType::PlayerExtinguished => {
                GameEvent::Extinguished(parse_event(raw.params).with_type(raw.ty)?)
            }
            RawEventType::GameOver => {
                GameEvent::GameOver(parse_event(raw.params).with_type(raw.ty)?)
            }
            RawEventType::FinalScore => {
                GameEvent::FinalScore(parse_event(raw.params).with_type(raw.ty)?)
            }
            RawEventType::LogFileClosed => GameEvent::LogFileClosed,
            RawEventType::ObjectDetonated => {
                GameEvent::ObjectDetonated(parse_event(raw.params).with_type(raw.ty)?)
            }
            _ => {
                todo!("{:?} not parsed yet", raw.ty);
            }
        })
    }
}

pub struct ParamIter<'a> {
    input: &'a str,
}

impl<'a> ParamIter<'a> {
    pub fn new(input: &'a str) -> Self {
        ParamIter { input }
    }
}

impl<'a> Iterator for ParamIter<'a> {
    type Item = (&'a str, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        let (input, res) = param_pair_parse(self.input).ok()?;
        self.input = input;
        Some(res)
    }
}

fn param_pair_parse(input: &str) -> IResult<'_, (&str, &str)> {
    let (input, open_tag) = opt(tag("("))(input)?;

    let (input, key) = alpha1(input)?;
    let (input, _) = tag(r#" ""#)(input)?;
    let (input, value) = take_while(|c| c != '"')(input)?;
    let (input, _) = tag(r#"""#)(input)?;

    if open_tag.is_some() {
        let (_input, _) = tag(")")(input)?;
    }
    Ok((input, (key, value)))
}

fn quoted<'a, T, P: Fn(&'a str) -> IResult<'a, T>>(
    parser: P,
) -> impl Fn(&'a str) -> IResult<'a, T> {
    move |input| {
        let (input, _) = tag(r#"""#)(input)?;
        let (input, res) = parser(input)?;
        let (input, _) = tag(r#"""#)(input)?;
        Ok((input, res))
    }
}

pub fn param_parse<'a, T: EventField<'a>>(key: &'a str) -> impl Fn(&'a str) -> IResult<'a, T> {
    param_parse_with(key, T::parse_field)
}

pub fn param_parse_with<'a, T, P: Fn(&'a str) -> IResult<'a, T>>(
    key: &'a str,
    parser: P,
) -> impl Fn(&'a str) -> IResult<'a, T> {
    move |input: &str| {
        debug_assert!(input.as_bytes()[0] != b' ');

        let has_open = input.as_bytes()[0] == b'(';
        let input = &input[has_open as usize..];

        let input = &input[key.len() + 1..]; // skip space + key

        let (input, value) = parser(input)?;

        let input = &input[has_open as usize..];

        debug_assert!(
            input.is_empty() || input.as_bytes()[0] == b' ',
            "\"{}\" starts with space",
            input
        );

        let input = &input[(!input.is_empty() as usize)..];
        Ok((input, value))
    }
}

fn parse_from_str<'a, T: FromStr + 'a>(input: &'a str) -> IResult<T> {
    T::from_str(input)
        .map(|res| ("", res))
        .map_err(|_| nom::Err::Error(nom::error::Error::from_error_kind(input, ErrorKind::IsNot)))
}

fn int(input: &str) -> IResult<i32> {
    let (input, sign) = opt(tag("-"))(input)?;
    let (input, raw) = digit1(input)?;
    let val: i32 = raw
        .parse()
        .map_err(|_| nom::Err::Error(nom::error::Error::new(raw, ErrorKind::Digit)))?;
    Ok((input, if sign.is_some() { -val } else { val }))
}

fn u_int(input: &str) -> IResult<u32> {
    let (input, quote) = opt(tag("\""))(input)?;

    let (input, raw) = digit1(input)?;
    let val = raw
        .parse()
        .map_err(|_| nom::Err::Error(nom::error::Error::new(raw, ErrorKind::Digit)))?;

    let input = if quote.is_some() {
        tag("\"")(input)?.0
    } else {
        input
    };
    Ok((input, val))
}

pub fn position(input: &str) -> IResult<(i32, i32, i32)> {
    let (input, x) = int(input)?;
    let (input, _) = tag(" ")(input)?;
    let (input, y) = int(input)?;
    let (input, _) = tag(" ")(input)?;
    let (input, z) = int(input)?;
    Ok((input, (x, y, z)))
}

pub trait EventField<'a>: Sized + 'a {
    fn parse_field(input: &'a str) -> crate::IResult<Self>;
}

impl<'a> EventField<'a> for &'a str {
    fn parse_field(input: &'a str) -> crate::IResult<Self> {
        if input.starts_with('"') {
            quoted(take_while(|c| c != '"'))(input)
        } else {
            Ok(("", input))
        }
    }
}

impl<'a> EventField<'a> for i32 {
    fn parse_field(input: &'a str) -> crate::IResult<Self> {
        int(input)
    }
}

impl<'a> EventField<'a> for u32 {
    fn parse_field(input: &'a str) -> crate::IResult<Self> {
        u_int(input)
    }
}

impl<'a> EventField<'a> for f32 {
    fn parse_field(input: &'a str) -> crate::IResult<Self> {
        float(input)
    }
}

impl<'a> EventField<'a> for u8 {
    fn parse_field(input: &'a str) -> crate::IResult<Self> {
        u_int(input).map(|(rest, num)| (rest, num as u8))
    }
}

impl<'a, T: EventField<'a>> EventField<'a> for Option<T> {
    fn parse_field(input: &'a str) -> crate::IResult<Self> {
        T::parse_field(input).map(|(rest, int)| (rest, Some(int)))
    }
}

pub trait EventFieldFromStr: FromStr {}

impl<'a, T: EventFieldFromStr + 'a> EventField<'a> for T {
    fn parse_field(input: &'a str) -> crate::IResult<Self> {
        parse_from_str(input)
    }
}

impl EventFieldFromStr for SocketAddr {}

impl<'a, T: EventField<'a>> EventField<'a> for (T, T, T) {
    fn parse_field(input: &'a str) -> crate::IResult<Self> {
        let (input, x) = T::parse_field(input)?;
        let (input, _) = tag(" ")(input)?;
        let (input, y) = T::parse_field(input)?;
        let (input, _) = tag(" ")(input)?;
        let (input, z) = T::parse_field(input)?;
        Ok((input, (x, y, z)))
    }
}

impl<'a> EventField<'a> for Option<NonZeroU32> {
    fn parse_field(input: &'a str) -> crate::IResult<Self> {
        u32::parse_field(input).map(|(rest, int)| (rest, NonZeroU32::new(int)))
    }
}

impl<'a> EventField<'a> for RawSubject<'a> {
    fn parse_field(input: &'a str) -> crate::IResult<Self> {
        against_subject_parser(input)
    }
}

pub fn parse_field<'a, T: EventField<'a>>(input: &'a str) -> crate::IResult<T> {
    T::parse_field(input)
}
