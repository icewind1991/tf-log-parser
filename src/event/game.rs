use crate::event::{param_parse, param_parse_with};
use nom::combinator::opt;
use nom::number::complete::float;
use nom::IResult;

#[derive(Debug)]
pub struct RoundWinEvent<'a> {
    pub team: Option<&'a str>,
}

pub fn round_win_event_parser(input: &str) -> IResult<&str, RoundWinEvent> {
    let (input, team) = opt(param_parse("against"))(input)?;
    Ok((input, RoundWinEvent { team }))
}

#[derive(Debug)]
pub struct RoundLengthEvent {
    pub length: Option<f32>,
}

pub fn round_length_event_parser(input: &str) -> IResult<&str, RoundLengthEvent> {
    let (input, length) = opt(param_parse_with("against", float))(input)?;
    Ok((input, RoundLengthEvent { length }))
}
