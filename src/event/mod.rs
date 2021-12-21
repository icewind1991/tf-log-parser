mod game;
mod medic;
mod player;

use crate::event::game::{RoundLengthEvent, RoundWinEvent};
use crate::{RawEvent, RawEventType, SubjectId};
pub use game::*;
pub use medic::*;
use nom::bytes::complete::{tag, take_while};
use nom::character::complete::{alpha1, digit1};
use nom::combinator::opt;
use nom::error::{ErrorKind, ParseError};
use nom::{Err, IResult};
pub use player::*;
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

impl<'a, T> GameEventErrTrait<T> for IResult<&str, T> {
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
                GameEvent::ShotFired(shot_fired_event_parser(raw.params).with_type(raw.ty)?)
            }
            RawEventType::ShotHit => {
                GameEvent::ShotHit(shot_hit_event_parser(raw.params).with_type(raw.ty)?)
            }
            RawEventType::Damage => {
                GameEvent::Damage(damage_event_parser(raw.params).with_type(raw.ty)?)
            }
            RawEventType::Killed => {
                GameEvent::Kill(kill_event_parser(raw.params).with_type(raw.ty)?)
            }
            RawEventType::KillAssist => {
                GameEvent::KillAssist(kill_assist_event_parser(raw.params).with_type(raw.ty)?)
            }
            RawEventType::SayTeam => GameEvent::SayTeam(raw.params.trim_matches('"')),
            RawEventType::Say => GameEvent::Say(raw.params.trim_matches('"')),
            RawEventType::Healed => {
                GameEvent::Healed(healed_event_parser(raw.params).with_type(raw.ty)?)
            }
            RawEventType::ChargeDeployed => GameEvent::ChargeDeployed(
                charge_deployed_event_parser(raw.params).with_type(raw.ty)?,
            ),
            RawEventType::ChargeEnd => {
                GameEvent::ChargeEnded(charge_ended_event_parser(raw.params).with_type(raw.ty)?)
            }
            RawEventType::UberAdvantageLost => {
                GameEvent::AdvantageLost(advantage_lost_event_parser(raw.params).with_type(raw.ty)?)
            }
            RawEventType::FirstHealAfterSpawn => {
                GameEvent::FirstHeal(first_heal_event_parser(raw.params).with_type(raw.ty)?)
            }
            RawEventType::ChargeReady => GameEvent::ChargeReady,
            RawEventType::MedicDeath => {
                GameEvent::MedicDeath(medic_death_event_parser(raw.params).with_type(raw.ty)?)
            }
            RawEventType::MedicDeathEx => {
                GameEvent::MedicDeathEx(medic_death_ex_event_parser(raw.params).with_type(raw.ty)?)
            }
            RawEventType::Spawned => {
                GameEvent::Spawned(spawn_event_parser(raw.params).with_type(raw.ty)?)
            }
            RawEventType::ChangedRole => {
                GameEvent::RoleChange(role_changed_event_parser(raw.params).with_type(raw.ty)?)
            }
            RawEventType::RoundStart => GameEvent::RoundStart,
            RawEventType::RoundLength => {
                GameEvent::RoundLength(round_length_event_parser(raw.params).with_type(raw.ty)?)
            }
            RawEventType::RoundWin => {
                GameEvent::RoundWin(round_win_event_parser(raw.params).with_type(raw.ty)?)
            }
            RawEventType::RoundOvertime => GameEvent::RoundOverTime,
            RawEventType::LogFileStarted => GameEvent::LogFileStarted(
                log_file_started_event_parser(raw.params).with_type(raw.ty)?,
            ),
            RawEventType::Connected => {
                GameEvent::Connected(connected_event_parser(raw.params).with_type(raw.ty)?)
            }
            RawEventType::Disconnected => {
                GameEvent::Disconnect(disconnected_event_parser(raw.params).with_type(raw.ty)?)
            }
            RawEventType::SteamIdValidated => GameEvent::SteamIdValidated,
            RawEventType::Entered => GameEvent::Entered,
            RawEventType::Joined => {
                GameEvent::Joined(joined_team_event_parser(raw.params).with_type(raw.ty)?)
            }
            RawEventType::Suicide => {
                GameEvent::Suicide(committed_suicide_event_parser(raw.params).with_type(raw.ty)?)
            }
            RawEventType::PickedUp => {
                GameEvent::PickedUp(picked_up_event_parser(raw.params).with_type(raw.ty)?)
            }
            RawEventType::Domination => {
                GameEvent::Domination(domination_event_parser(raw.params).with_type(raw.ty)?)
            }
            RawEventType::EmptyUber => GameEvent::EmptyUber,
            RawEventType::Revenge => {
                GameEvent::Revenge(revenge_event_parser(raw.params).with_type(raw.ty)?)
            }
            RawEventType::TournamentStart => GameEvent::TournamentModeStarted(
                tournament_mode_started_event_parser(raw.params).with_type(raw.ty)?,
            ),
            RawEventType::CaptureBlocked => GameEvent::CaptureBlocked(
                capture_blocked_event_parser(raw.params).with_type(raw.ty)?,
            ),
            RawEventType::PointCaptured => {
                GameEvent::PointCaptured(point_captures_event_parser(raw.params).with_type(raw.ty)?)
            }
            RawEventType::CurrentScore => {
                GameEvent::CurrentScore(current_score_event_parser(raw.params).with_type(raw.ty)?)
            }
            RawEventType::PlayerBuiltObject => {
                GameEvent::BuiltObject(built_object_event_parser(raw.params).with_type(raw.ty)?)
            }
            RawEventType::PlayerKilledObject => {
                GameEvent::KilledObject(killed_object_event_parser(raw.params).with_type(raw.ty)?)
            }
            RawEventType::PlayerExtinguished => {
                GameEvent::Extinguished(extinguished_event_parser(raw.params).with_type(raw.ty)?)
            }
            RawEventType::GameOver => {
                GameEvent::GameOver(game_over_event_parser(raw.params).with_type(raw.ty)?)
            }
            RawEventType::FinalScore => {
                GameEvent::FinalScore(final_score_event_parser(raw.params).with_type(raw.ty)?)
            }
            RawEventType::LogFileClosed => GameEvent::LogFileClosed,
            RawEventType::ObjectDetonated => GameEvent::ObjectDetonated(
                object_detonated_event_parser(raw.params).with_type(raw.ty)?,
            ),
            _ => {
                todo!("{:?} not parsed yet", raw.ty);
            }
        })
    }
}

struct ParamIter<'a> {
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

fn param_pair_parse(input: &str) -> IResult<&str, (&str, &str)> {
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

fn quoted<'a, T, P: Fn(&'a str) -> IResult<&'a str, T>>(
    parser: P,
) -> impl Fn(&'a str) -> IResult<&'a str, T> {
    move |input| {
        let (input, _) = tag(r#"""#)(input)?;
        let (input, res) = parser(input)?;
        let (input, _) = tag(r#"""#)(input)?;
        Ok((input, res))
    }
}

fn param_parse<'a>(key: &'a str) -> impl Fn(&'a str) -> IResult<&'a str, &'a str> {
    param_parse_with(key, quoted(take_while(|c| c != '"')))
}

fn param_parse_with<'a, T, P: Fn(&'a str) -> IResult<&'a str, T>>(
    key: &'a str,
    parser: P,
) -> impl Fn(&'a str) -> IResult<&'a str, T> {
    move |input: &str| {
        let (input, _) = opt(tag(" "))(input)?;
        let (input, open_tag) = opt(tag("("))(input)?;

        let (input, _) = tag(key)(input)?;
        let (input, _) = tag(r#" "#)(input)?;

        let (input, value) = parser(input)?;

        if open_tag.is_some() {
            let (_input, _) = tag(")")(input)?;
        }
        Ok((input.trim_start(), value))
    }
}

fn parse_from_str<T: FromStr>(input: &str) -> IResult<&str, T> {
    T::from_str(input)
        .map(|res| ("", res))
        .map_err(|_| nom::Err::Error(nom::error::Error::from_error_kind(input, ErrorKind::IsNot)))
}

fn int(input: &str) -> IResult<&str, i32> {
    let (input, sign) = opt(tag("-"))(input)?;
    let (input, raw) = digit1(input)?;
    let val: i32 = raw
        .parse()
        .map_err(|_| nom::Err::Error(nom::error::Error::new(raw, ErrorKind::Digit)))?;
    Ok((input, if sign.is_some() { -val } else { val }))
}

fn u_int(input: &str) -> IResult<&str, u32> {
    let (input, raw) = digit1(input)?;
    let val = raw
        .parse()
        .map_err(|_| nom::Err::Error(nom::error::Error::new(raw, ErrorKind::Digit)))?;
    Ok((input, val))
}

fn position(input: &str) -> IResult<&str, (i32, i32, i32)> {
    let (input, x) = int(input)?;
    let (input, _) = tag(" ")(input)?;
    let (input, y) = int(input)?;
    let (input, _) = tag(" ")(input)?;
    let (input, z) = int(input)?;
    Ok((input, (x, y, z)))
}
