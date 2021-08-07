use crate::raw_event::{subject_parser, RawSubject};
use nom::bytes::complete::tag;
use nom::character::complete::digit1;
use nom::error::ErrorKind;
use nom::IResult;

#[derive(Debug)]
pub struct HealedEvent<'a> {
    pub subject: RawSubject<'a>,
    pub amount: u32,
}

pub fn healed_event_parser(input: &str) -> IResult<&str, HealedEvent> {
    let (input, _) = tag("against ")(input)?;
    let (input, subject) = subject_parser(input)?;
    let (input, _) = tag(r#" (healing ""#)(input)?;
    let (input, raw_amount) = digit1(input)?;
    let amount = raw_amount
        .parse()
        .map_err(|_| nom::Err::Error(nom::error::Error::new(raw_amount, ErrorKind::Digit)))?;
    Ok((input, HealedEvent { subject, amount }))
}
