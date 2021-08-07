use crate::event::{param_parse_with, quoted, u_int};
use crate::raw_event::{subject_parser, RawSubject};
use nom::IResult;

#[derive(Debug)]
pub struct HealedEvent<'a> {
    pub subject: RawSubject<'a>,
    pub amount: u32,
}

pub fn healed_event_parser(input: &str) -> IResult<&str, HealedEvent> {
    let (input, subject) = param_parse_with("against", subject_parser)(input)?;
    let (input, amount) = param_parse_with("healing", quoted(u_int))(input)?;
    Ok((input, HealedEvent { subject, amount }))
}
