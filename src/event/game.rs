use crate::event::{param_parse_with, parse_field, ParamIter};
use crate::raw_event::RawSubject;
use crate::{Error, Event, IResult};

use crate::common::{skip, take_until};

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
        let input = skip(input, "\nBlue Team: ".len())?;
        let (input, blue) = take_until(input, b'\n');
        let input = skip(input, "\nRed Team: ".len())?;
        let (input, red) = take_until(input, b'\n');
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
        let mut cp = Default::default();
        let mut cp_name = Default::default();
        let mut num_cappers = Default::default();
        let mut players = Vec::new();

        let mut params = ParamIter::new(input).peekable();
        while let Some((name, value)) = params.peek() {
            match *name {
                "cp" => {
                    cp = Some(value.parse().map_err(Error::from)?);
                    let _ = params.next();
                }
                "cpname" => {
                    cp_name = Some(*value);
                    let _ = params.next();
                }
                "numcappers" => {
                    num_cappers = Some(value.parse().map_err(Error::from)?);
                    let _ = params.next();
                }
                _ => {
                    break;
                }
            }
        }
        loop {
            match (params.next(), params.next()) {
                (Some((subject_key, subject)), Some((position_key, position_str)))
                    if subject_key.starts_with("player")
                        && position_key.starts_with("position") =>
                {
                    players.push((parse_field(subject)?.1, parse_field(position_str)?.1));
                }
                _ => break,
            }
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
