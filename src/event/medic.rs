use crate::event::{param_parse, param_parse_with, quoted, u_int};
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
