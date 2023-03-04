use crate::common::{Class, Team};
use crate::event::{param_parse, param_parse_with, parse_from_str, position, u_int, ParamIter};
use crate::raw_event::{against_subject_parser, RawSubject};
use nom::combinator::opt;
use nom::IResult;
use std::net::SocketAddr;
use std::num::NonZeroU32;

#[derive(Debug)]
pub struct ShotFiredEvent<'a> {
    pub weapon: Option<&'a str>,
}

pub fn shot_fired_event_parser(input: &str) -> IResult<&str, ShotFiredEvent> {
    let (input, weapon) = opt(param_parse("weapon"))(input)?;
    Ok((input, ShotFiredEvent { weapon }))
}

#[derive(Debug)]
pub struct ShotHitEvent<'a> {
    pub weapon: Option<&'a str>,
}

pub fn shot_hit_event_parser(input: &str) -> IResult<&str, ShotHitEvent> {
    let (input, weapon) = opt(param_parse("weapon"))(input)?;
    Ok((input, ShotHitEvent { weapon }))
}

#[derive(Debug)]
pub struct DamageEvent<'a> {
    pub target: RawSubject<'a>,
    pub damage: Option<NonZeroU32>,
    pub real_damage: Option<NonZeroU32>,
    pub weapon: Option<&'a str>,
}

pub fn damage_event_parser(input: &str) -> IResult<&str, DamageEvent> {
    let (input, target) = param_parse_with("against", against_subject_parser)(input)?;
    let mut event = DamageEvent {
        target,
        damage: None,
        real_damage: None,
        weapon: None,
    };
    for (key, value) in ParamIter::new(input) {
        match key {
            "damage" => event.damage = NonZeroU32::new(u_int(value)?.1),
            "realdamage" => event.real_damage = NonZeroU32::new(u_int(value)?.1),
            "weapon" => event.weapon = Some(value.trim_matches('"')),
            _ => {}
        }
    }
    Ok(("", event))
}

#[derive(Debug)]
pub struct KillEvent<'a> {
    pub target: RawSubject<'a>,
    pub weapon: &'a str,
    pub attacker_position: Option<(i32, i32, i32)>,
    pub victim_position: Option<(i32, i32, i32)>,
}

pub fn kill_event_parser(input: &str) -> IResult<&str, KillEvent> {
    let (input, target) = against_subject_parser(input)?;
    let (input, weapon) = param_parse("with")(&input[1..])?;
    let mut event = KillEvent {
        target,
        weapon,
        attacker_position: None,
        victim_position: None,
    };
    for (key, value) in ParamIter::new(input) {
        match key {
            "attacker_position" => event.attacker_position = Some(position(value)?.1),
            "victim_position" => event.victim_position = Some(position(value)?.1),
            _ => {}
        }
    }
    Ok(("", event))
}

#[derive(Debug)]
pub struct KillAssistEvent<'a> {
    pub target: RawSubject<'a>,
    pub attacker_position: Option<(i32, i32, i32)>,
    pub victim_position: Option<(i32, i32, i32)>,
}

pub fn kill_assist_event_parser(input: &str) -> IResult<&str, KillAssistEvent> {
    let (input, target) = param_parse_with("against", against_subject_parser)(input)?;
    let mut event = KillAssistEvent {
        target,
        attacker_position: None,
        victim_position: None,
    };
    for (key, value) in ParamIter::new(input) {
        match key {
            "attacker_position" => event.attacker_position = Some(position(value)?.1),
            "victim_position" => event.victim_position = Some(position(value)?.1),
            _ => {}
        }
    }
    Ok(("", event))
}

#[derive(Debug)]
pub struct SpawnEvent {
    pub class: Option<Class>,
}

pub fn spawn_event_parser(input: &str) -> IResult<&str, SpawnEvent> {
    let (input, class_str) = param_parse("as")(input)?;
    Ok((
        input,
        SpawnEvent {
            class: class_str.parse().ok(),
        },
    ))
}

#[derive(Debug)]
pub struct RoleChangeEvent {
    pub class: Option<Class>,
}

pub fn role_changed_event_parser(input: &str) -> IResult<&str, RoleChangeEvent> {
    let (input, class_str) = param_parse("to")(input)?;
    Ok((
        input,
        RoleChangeEvent {
            class: class_str.parse().ok(),
        },
    ))
}

#[derive(Debug)]
pub struct ConnectedEvent {
    pub address: SocketAddr,
}

pub fn connected_event_parser(input: &str) -> IResult<&str, ConnectedEvent> {
    let (input, address) = param_parse_with("to", parse_from_str)(input)?;
    Ok((input, ConnectedEvent { address }))
}

#[derive(Debug)]
pub struct JoinedTeamEvent {
    pub team: Team,
}

pub fn joined_team_event_parser(input: &str) -> IResult<&str, JoinedTeamEvent> {
    let (input, team) = param_parse_with("team", parse_from_str)(input)?;
    Ok((input, JoinedTeamEvent { team }))
}

#[derive(Debug)]
pub struct CommittedSuicideEvent<'a> {
    pub weapon: &'a str,
    pub attacker_position: Option<(i32, i32, i32)>,
}

pub fn committed_suicide_event_parser(input: &str) -> IResult<&str, CommittedSuicideEvent> {
    let (input, weapon) = param_parse("with")(input)?;
    let (input, attacker_position) = opt(param_parse_with("attacker_position", position))(input)?;
    Ok((
        input,
        CommittedSuicideEvent {
            weapon,
            attacker_position,
        },
    ))
}

#[derive(Debug)]
pub struct PickedUpEvent<'a> {
    pub item: &'a str,
}

pub fn picked_up_event_parser(input: &str) -> IResult<&str, PickedUpEvent> {
    let (input, item) = param_parse("item")(input)?;
    Ok((input, PickedUpEvent { item }))
}

#[derive(Debug)]
pub struct DominationEvent<'a> {
    pub against: RawSubject<'a>,
}

pub fn domination_event_parser(input: &str) -> IResult<&str, DominationEvent> {
    let (input, against) = param_parse_with("against", against_subject_parser)(input)?;
    Ok((input, DominationEvent { against }))
}

#[derive(Debug)]
pub struct RevengeEvent<'a> {
    pub against: RawSubject<'a>,
}

pub fn revenge_event_parser(input: &str) -> IResult<&str, RevengeEvent> {
    let (input, against) = param_parse_with("against", against_subject_parser)(input)?;
    Ok((input, RevengeEvent { against }))
}

#[derive(Debug)]
pub struct DisconnectEvent<'a> {
    pub reason: Option<&'a str>,
}

pub fn disconnected_event_parser(input: &str) -> IResult<&str, DisconnectEvent> {
    let (input, reason) = opt(param_parse("reason"))(input)?;
    Ok((input, DisconnectEvent { reason }))
}

#[derive(Debug)]
pub struct BuiltObjectEvent<'a> {
    pub object: Option<&'a str>,
    pub position: Option<(i32, i32, i32)>,
}

pub fn built_object_event_parser(input: &str) -> IResult<&str, BuiltObjectEvent> {
    let (input, object) = opt(param_parse("object"))(input)?;
    let (input, position) = opt(param_parse_with("position", position))(input)?;
    Ok((input, BuiltObjectEvent { object, position }))
}

#[derive(Debug)]
pub struct KilledObjectEvent<'a> {
    pub object: Option<&'a str>,
    pub weapon: Option<&'a str>,
    pub object_owner: Option<RawSubject<'a>>,
    pub attacker_position: Option<(i32, i32, i32)>,
}

pub fn killed_object_event_parser(input: &str) -> IResult<&str, KilledObjectEvent> {
    let (input, object) = opt(param_parse("object"))(input)?;
    let (input, weapon) = opt(param_parse("weapon"))(input)?;
    let (input, object_owner) =
        opt(param_parse_with("objectowner", against_subject_parser))(input)?;
    let (input, attacker_position) = opt(param_parse_with("attacker_position", position))(input)?;
    Ok((
        input,
        KilledObjectEvent {
            object,
            weapon,
            object_owner,
            attacker_position,
        },
    ))
}

#[derive(Debug)]
pub struct ObjectDetonatedEvent<'a> {
    pub object: Option<&'a str>,
    pub position: Option<(i32, i32, i32)>,
}

pub fn object_detonated_event_parser(input: &str) -> IResult<&str, ObjectDetonatedEvent> {
    let (input, object) = opt(param_parse("object"))(input)?;
    let (input, position) = opt(param_parse_with("attacker_position", position))(input)?;
    Ok((input, ObjectDetonatedEvent { object, position }))
}

#[derive(Debug)]
pub struct ExtinguishedEvent<'a> {
    pub against: RawSubject<'a>,
    pub with: &'a str,
    pub attacker_position: Option<(i32, i32, i32)>,
    pub victim_position: Option<(i32, i32, i32)>,
}

pub fn extinguished_event_parser(input: &str) -> IResult<&str, ExtinguishedEvent> {
    let (input, against) = param_parse_with("against", against_subject_parser)(input)?;
    let (input, with) = param_parse("with")(input)?;
    let (input, attacker_position) = opt(param_parse_with("attacker_position", position))(input)?;
    let (input, victim_position) = opt(param_parse_with("victim_position", position))(input)?;
    Ok((
        input,
        ExtinguishedEvent {
            against,
            with,
            attacker_position,
            victim_position,
        },
    ))
}
