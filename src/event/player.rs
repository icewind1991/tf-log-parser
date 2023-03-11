use crate::common::{Class, Team};
use crate::event::{param_parse_with, parse_field, quoted, ParamIter};
use crate::raw_event::RawSubject;
use crate::{Error, Event, Result};
use std::net::SocketAddr;
use std::num::NonZeroU32;

#[derive(Debug, Event)]
pub struct ShotFiredEvent<'a> {
    pub weapon: Option<&'a str>,
}

#[derive(Debug, Event)]
pub struct ShotHitEvent<'a> {
    pub weapon: Option<&'a str>,
}

#[derive(Debug, Event)]
pub struct DamageEvent<'a> {
    #[event(name = "against")]
    pub target: RawSubject<'a>,
    pub damage: Option<NonZeroU32>,
    #[event(name = "realdamage")]
    pub real_damage: Option<NonZeroU32>,
    pub weapon: Option<&'a str>,
}

#[derive(Debug, Event)]
pub struct KillEvent<'a> {
    #[event(unnamed)]
    #[event(skip_after = 1)]
    pub target: RawSubject<'a>,
    #[event(name = "with")]
    pub weapon: &'a str,
    pub attacker_position: Option<(i32, i32, i32)>,
    pub victim_position: Option<(i32, i32, i32)>,
}

#[derive(Debug, Event)]
pub struct KillAssistEvent<'a> {
    #[event(name = "against")]
    pub target: RawSubject<'a>,
    pub attacker_position: Option<(i32, i32, i32)>,
    pub victim_position: Option<(i32, i32, i32)>,
}

#[derive(Debug, Event)]
pub struct SpawnEvent {
    #[event(name = "as")]
    pub class: Option<Class>,
}

#[derive(Debug, Event)]
pub struct RoleChangeEvent {
    #[event(name = "to")]
    pub class: Option<Class>,
}

#[derive(Debug, Event)]
pub struct ConnectedEvent {
    #[event(name = "address")]
    pub address: SocketAddr,
}

#[derive(Debug, Event)]
pub struct JoinedTeamEvent {
    pub team: Team,
}

#[derive(Debug, Event)]
pub struct CommittedSuicideEvent<'a> {
    #[event(name = "with")]
    pub weapon: &'a str,
    pub attacker_position: Option<(i32, i32, i32)>,
}

#[derive(Debug, Event)]
pub struct PickedUpEvent<'a> {
    pub item: &'a str,
}

#[derive(Debug, Event)]
pub struct DominationEvent<'a> {
    pub against: RawSubject<'a>,
}

#[derive(Debug, Event)]
pub struct RevengeEvent<'a> {
    pub against: RawSubject<'a>,
}

#[derive(Debug, Event)]
pub struct DisconnectEvent<'a> {
    pub reason: Option<&'a str>,
}

#[derive(Debug, Event)]
pub struct BuiltObjectEvent<'a> {
    pub object: Option<&'a str>,
    pub position: Option<(i32, i32, i32)>,
}

#[derive(Debug, Event)]
pub struct BuiltCarryEvent<'a> {
    pub object: Option<&'a str>,
    pub position: Option<(i32, i32, i32)>,
}

#[derive(Debug, Event)]
pub struct DropObjectEvent<'a> {
    pub object: Option<&'a str>,
    pub position: Option<(i32, i32, i32)>,
}

#[derive(Debug, Event)]
pub struct KilledObjectEvent<'a> {
    pub object: Option<&'a str>,
    pub weapon: Option<&'a str>,
    #[event(name = "objectowner")]
    pub object_owner: Option<RawSubject<'a>>,
    pub attacker_position: Option<(i32, i32, i32)>,
}

#[derive(Debug, Event)]
pub struct ObjectDetonatedEvent<'a> {
    pub object: Option<&'a str>,
    #[event(name = "attacker_position")]
    pub position: Option<(i32, i32, i32)>,
}

#[derive(Debug, Event)]
pub struct ExtinguishedEvent<'a> {
    pub against: RawSubject<'a>,
    pub with: &'a str,
    pub attacker_position: Option<(i32, i32, i32)>,
    pub victim_position: Option<(i32, i32, i32)>,
}

#[derive(Debug, Event)]
pub struct SayEvent<'a> {
    #[event(unnamed)]
    pub text: &'a str,
}

#[derive(Debug, Event)]
pub struct SayTeamEvent<'a> {
    #[event(unnamed)]
    pub text: &'a str,
}
