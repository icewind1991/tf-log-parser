mod medic;
mod player;

pub use medic::*;
use nom::bytes::complete::{tag, take_while};
use nom::character::complete::{alpha1, digit1};
use nom::combinator::opt;
use nom::error::ErrorKind;
use nom::IResult;
pub use player::*;

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
        let (_input, _) = tag("(")(input)?;
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
