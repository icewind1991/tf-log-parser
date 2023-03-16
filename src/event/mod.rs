mod game;
mod medic;
mod player;

use crate::event::game::{RoundLengthEvent, RoundWinEvent};
use crate::parsing::{skip, skip_matches, split_once, split_subject_end};
use crate::raw_event::{against_subject_parser, RawSubject};
use crate::{Error, Events, IResult, RawEvent, RawEventType, Result, SubjectId};
pub use game::*;
pub use medic::*;
pub use player::*;
use std::net::SocketAddr;
use std::num::NonZeroU32;
use std::str::FromStr;

#[derive(thiserror::Error, Debug)]
pub enum GameEventError {
    #[error("malformed game event({ty:?}): {err} in \"{params}\"")]
    Error {
        err: Box<Error>,
        ty: RawEventType,
        params: String,
    },
    #[error("incomplete event body({0:?})")]
    Incomplete(RawEventType),
}

trait GameEventErrTrait<T> {
    fn with_raw(self, raw: &RawEvent) -> Result<T, GameEventError>;
}

impl<'a, T> GameEventErrTrait<T> for Result<T> {
    fn with_raw(self, raw: &RawEvent) -> Result<T, GameEventError> {
        self.map_err(|err| GameEventError::Error {
            err: Box::new(err),
            ty: raw.ty,
            params: raw.params.to_string(),
        })
    }
}

pub trait Event<'a>: Sized + 'a {
    fn parse(input: &'a str) -> Result<Self>;
}

fn parse_event<'a, T: Event<'a>>(input: &'a str) -> Result<T> {
    T::parse(input)
}

#[derive(Debug)]
pub struct EventMeta {
    pub time: u32,
    pub subject: SubjectId,
}

#[derive(Debug, Events)]
pub enum GameEvent<'a> {
    ShotFired(ShotFiredEvent<'a>),
    ShotHit(ShotHitEvent<'a>),
    Damage(DamageEvent<'a>),
    Killed(KillEvent<'a>),
    KillAssist(KillAssistEvent<'a>),
    Say(SayEvent<'a>),
    SayTeam(SayTeamEvent<'a>),
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
    DropObject(DropObjectEvent<'a>),
    CarryObject(BuiltCarryEvent<'a>),
    KilledObject(KilledObjectEvent<'a>),
    Extinguished(ExtinguishedEvent<'a>),
    GameOver(GameOverEvent<'a>),
    FinalScore(FinalScoreEvent),
    ObjectDetonated(ObjectDetonatedEvent<'a>),
    Request(UnparsedEvent<'a>),
    Response(UnparsedEvent<'a>),
    LogFileClosed,
}

#[derive(Debug)]
pub struct UnparsedEvent<'a> {
    pub params: &'a str,
}

impl<'a> Event<'a> for UnparsedEvent<'a> {
    fn parse(input: &'a str) -> Result<Self> {
        Ok(UnparsedEvent { params: input })
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
    let (input, open_tag) = skip_matches(input, b'(');

    let (key, input) = split_once(input, b' ', 1)?;
    let input = skip(input, 1)?;
    let (value, input) = split_once(input, b'"', 1)?;

    let input = if open_tag { skip(input, 1)? } else { input };
    Ok((input, (key, value)))
}

fn quoted<'a, T, P: Fn(&'a str) -> Result<T>>(parser: P) -> impl Fn(&'a str) -> IResult<'a, T> {
    move |input| {
        let input = skip(input, 1)?;
        let (inner, input) = split_once(input, b'"', 1)?;
        let res = parser(inner)?;
        Ok((input, res))
    }
}

pub fn param_parse<'a, T: EventField<'a>>(key: &'a str) -> impl Fn(&'a str) -> IResult<'a, T> {
    param_parse_with(key, T::parse_field)
}

pub fn param_parse_with<'a, T, P: Fn(&'a str) -> Result<T>>(
    key: &'a str,
    parser: P,
) -> impl Fn(&'a str) -> IResult<'a, T> {
    move |input: &str| {
        let (input, has_open) = skip_matches(input, b'(');

        let input = skip(input, key.len() + 2)?; // skip space + key + quote

        // hack to handle quotes in names
        let (value, input) = if key == "against" || key == "objectowner" {
            split_subject_end(input, 1)?
        } else {
            split_once(input, b'"', 1)?
        };

        let value = parser(value)?;

        let input = skip(input, has_open as usize)?;

        let input = skip(input, 1).unwrap_or(input);
        Ok((input, value))
    }
}

fn parse_from_str<'a, T: FromStr + 'a>(input: &'a str) -> Result<T> {
    T::from_str(input).map_err(|_| Error::Malformed)
}

pub trait EventField<'a>: Sized + 'a {
    fn parse_field(input: &'a str) -> Result<Self>;
}

impl<'a> EventField<'a> for &'a str {
    fn parse_field(input: &'a str) -> Result<Self> {
        Ok(input)
    }
}

impl<'a, T: EventField<'a>> EventField<'a> for Option<T> {
    fn parse_field(input: &'a str) -> Result<Self> {
        T::parse_field(input).map(Some)
    }
}

pub trait EventFieldFromStr: FromStr {}

impl<'a, T: EventFieldFromStr + 'a> EventField<'a> for T {
    fn parse_field(input: &'a str) -> Result<Self> {
        parse_from_str(input)
    }
}

impl EventFieldFromStr for SocketAddr {}
impl EventFieldFromStr for u8 {}
impl EventFieldFromStr for u32 {}
impl EventFieldFromStr for i32 {}
impl EventFieldFromStr for f32 {}

impl<'a, T: EventField<'a>> EventField<'a> for (T, T, T) {
    fn parse_field(input: &'a str) -> Result<Self> {
        let (x, input) = split_once(input, b' ', 1)?;
        let x = parse_field(x)?;
        let (y, input) = split_once(input, b' ', 1)?;
        let y = parse_field(y)?;
        let z = parse_field(input)?;
        Ok((x, y, z))
    }
}

impl<'a> EventField<'a> for Option<NonZeroU32> {
    fn parse_field(input: &'a str) -> Result<Self> {
        u32::parse_field(input).map(|int| NonZeroU32::new(int))
    }
}

impl<'a> EventField<'a> for RawSubject<'a> {
    fn parse_field(input: &'a str) -> Result<Self> {
        against_subject_parser(input)
    }
}

pub fn parse_field<'a, T: EventField<'a>>(input: &'a str) -> Result<T> {
    T::parse_field(input)
}
