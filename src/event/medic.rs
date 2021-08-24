use crate::event::{param_parse, param_parse_with, quoted, u_int, ParamIter};
use crate::raw_event::{subject_parser, RawSubject};
use nom::combinator::opt;
use nom::number::complete::float;
use nom::IResult;

#[derive(Debug)]
pub struct HealedEvent<'a> {
    pub target: RawSubject<'a>,
    pub amount: u32,
}

pub fn healed_event_parser(input: &str) -> IResult<&str, HealedEvent> {
    let (input, subject) = param_parse_with("against", subject_parser)(input)?;
    let (input, amount) = param_parse_with("healing", quoted(u_int))(input)?;
    Ok((
        input,
        HealedEvent {
            target: subject,
            amount,
        },
    ))
}

#[derive(Debug)]
pub struct ChargeDeployedEvent<'a> {
    pub medigun: Option<&'a str>,
}

pub fn charge_deployed_event_parser(input: &str) -> IResult<&str, ChargeDeployedEvent> {
    let (input, medigun) = opt(param_parse("healing"))(input)?;
    Ok((input, ChargeDeployedEvent { medigun }))
}

#[derive(Debug)]
pub struct ChargeEndedEvent {
    pub duration: Option<f32>,
}

pub fn charge_ended_event_parser(input: &str) -> IResult<&str, ChargeEndedEvent> {
    let (input, duration) = opt(param_parse_with("duration", quoted(float)))(input)?;
    Ok((input, ChargeEndedEvent { duration }))
}

#[derive(Debug)]
pub struct AdvantageLostEvent {
    pub time: Option<f32>,
}

pub fn advantage_lost_event_parser(input: &str) -> IResult<&str, AdvantageLostEvent> {
    let (input, time) = opt(param_parse_with("time", quoted(float)))(input)?;
    Ok((input, AdvantageLostEvent { time }))
}

#[derive(Debug)]
pub struct FirstHealEvent {
    pub time: Option<f32>,
}

pub fn first_heal_event_parser(input: &str) -> IResult<&str, FirstHealEvent> {
    let (input, time) = opt(param_parse_with("time", quoted(float)))(input)?;
    Ok((input, FirstHealEvent { time }))
}

#[derive(Debug)]
pub struct MedicDeathEvent {
    pub charge: Option<u32>,
}

pub fn medic_death_event_parser(input: &str) -> IResult<&str, MedicDeathEvent> {
    let mut charge = None;
    for (key, value) in ParamIter::new(input) {
        if key == "ubercharge" {
            charge = Some(quoted(u_int)(value)?.1);
        }
    }
    Ok((input, MedicDeathEvent { charge }))
}

#[derive(Debug)]
pub struct MedicDeathExEvent {
    pub charge_percentage: Option<u8>,
}

pub fn medic_death_ex_event_parser(input: &str) -> IResult<&str, MedicDeathExEvent> {
    let (input, charge_percentage) = opt(param_parse_with("time", quoted(u_int)))(input)?;
    Ok((
        input,
        MedicDeathExEvent {
            charge_percentage: charge_percentage.map(|charge: u32| charge as u8),
        },
    ))
}
