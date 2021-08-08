use crate::common::Class;
use crate::event::{param_parse, param_parse_with, position, u_int, ParamIter};
use crate::raw_event::{subject_parser, RawSubject};
use nom::combinator::opt;
use nom::IResult;
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
    let (input, target) = param_parse_with("against", subject_parser)(input)?;
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
    let (input, target) = subject_parser(input)?;
    let (input, weapon) = param_parse("with")(input)?;
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
    let (input, target) = param_parse_with("against", subject_parser)(input)?;
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
