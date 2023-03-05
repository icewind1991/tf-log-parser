use crate::event::{param_parse, param_parse_with, parse_field, ParamIter};
use crate::raw_event::RawSubject;
use crate::{Event, IResult};

use nom::bytes::complete::{tag, take_while};
use nom::combinator::opt;

#[derive(Debug, Event)]
pub struct RoundWinEvent<'a> {
    #[event(name = "winner")]
    pub team: Option<&'a str>,
}

#[derive(Debug, Event)]
pub struct RoundLengthEvent {
    #[event(name = "seconds")]
    pub length: Option<f32>,
}

#[derive(Debug, Event)]
pub struct LogFileStartedEvent<'a> {
    pub file: Option<&'a str>,
    pub game: Option<&'a str>,
    pub version: Option<&'a str>,
}

#[derive(Debug)]
pub struct TournamentModeStartedEvent<'a> {
    pub blue: &'a str,
    pub red: &'a str,
}

impl<'a> Event<'a> for TournamentModeStartedEvent<'a> {
    fn parse(input: &'a str) -> IResult<Self> {
        let (input, _) = tag("\nBlue Team: ")(input)?;
        let (input, blue) = take_while(|c| c != '\n')(input)?;
        let (input, _) = tag("\nRed Team: ")(input)?;
        let (input, red) = take_while(|c| c != '\n')(input)?;
        Ok((input, TournamentModeStartedEvent { blue, red }))
    }
}

#[derive(Debug, Event)]
pub struct CaptureBlockedEvent<'a> {
    pub cp: Option<u8>,
    #[event(name = "cpname")]
    pub cp_name: Option<&'a str>,
    pub position: Option<(i32, i32, i32)>,
}

#[derive(Debug)]
pub struct PointCapturedEvent<'a> {
    pub cp: Option<u8>,
    pub cp_name: Option<&'a str>,
    pub num_cappers: Option<u8>,
    pub players: Vec<(RawSubject<'a>, (i32, i32, i32))>,
}

impl<'a> Event<'a> for PointCapturedEvent<'a> {
    fn parse(input: &'a str) -> IResult<Self> {
        let (input, cp) = opt(param_parse("cp"))(input)?;
        let (input, cp_name) = opt(param_parse("cpname"))(input)?;
        let (input, num_cappers) = opt(param_parse("numcappers"))(input)?;

        let mut players = Vec::new();

        let mut params = ParamIter::new(input);
        match (params.next(), params.next()) {
            (Some((subject_key, subject)), Some((position_key, position_str)))
                if subject_key.starts_with("player") && position_key.starts_with("position") =>
            {
                players.push((parse_field(subject)?.1, parse_field(position_str)?.1));
            }
            _ => {}
        }

        Ok((
            input,
            PointCapturedEvent {
                cp,
                num_cappers,
                cp_name,
                players,
            },
        ))
    }
}

#[derive(Debug, Event)]
pub struct CurrentScoreEvent {
    #[event(unnamed)]
    pub score: u8,
    #[event(name = "with")]
    pub players: u8,
}

#[derive(Debug, Event)]
pub struct GameOverEvent<'a> {
    pub reason: &'a str,
}

#[derive(Debug, Event)]
pub struct FinalScoreEvent {
    #[event(unnamed)]
    pub score: u8,
    #[event(name = "with")]
    pub players: u8,
}
