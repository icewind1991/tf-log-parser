use crate::event::{param_parse, param_parse_with, parse_from_str, position, u_int, ParamIter};
use crate::raw_event::{subject_parser, RawSubject};
use nom::bytes::complete::{tag, take_while};
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

#[derive(Debug)]
pub struct LogFileStartedEvent<'a> {
    pub file: Option<&'a str>,
    pub game: Option<&'a str>,
    pub version: Option<&'a str>,
}

pub fn log_file_started_event_parser(input: &str) -> IResult<&str, LogFileStartedEvent> {
    let (input, file) = opt(param_parse("file"))(input)?;
    let (input, game) = opt(param_parse("game"))(input)?;
    let (input, version) = opt(param_parse("version"))(input)?;
    Ok((
        input,
        LogFileStartedEvent {
            file,
            game,
            version,
        },
    ))
}

#[derive(Debug)]
pub struct TournamentModeStartedEvent<'a> {
    pub blue: &'a str,
    pub red: &'a str,
}

pub fn tournament_mode_started_event_parser(
    input: &str,
) -> IResult<&str, TournamentModeStartedEvent> {
    let (input, _) = tag("\nBlue Team: ")(input)?;
    let (input, blue) = take_while(|c| c != '\n')(input)?;
    let (input, _) = tag("\nRed Team: ")(input)?;
    let (input, red) = take_while(|c| c != '\n')(input)?;
    Ok((input, TournamentModeStartedEvent { blue, red }))
}

#[derive(Debug)]
pub struct CaptureBlockedEvent<'a> {
    pub cp: Option<u8>,
    pub cp_name: Option<&'a str>,
    pub position: Option<(i32, i32, i32)>,
}

pub fn capture_blocked_event_parser(input: &str) -> IResult<&str, CaptureBlockedEvent> {
    let (input, cp) = opt(param_parse_with("cp", u_int))(input)?;
    let (input, cp_name) = opt(param_parse("map"))(input)?;
    let (input, position) = opt(param_parse_with("red", position))(input)?;
    Ok((
        input,
        CaptureBlockedEvent {
            cp: cp.map(|cp| cp as u8),
            cp_name,
            position,
        },
    ))
}

#[derive(Debug)]
pub struct PointCapturedEvent<'a> {
    pub cp: Option<u8>,
    pub cp_name: Option<&'a str>,
    pub num_cappers: Option<u8>,
    pub players: Vec<(RawSubject<'a>, (i32, i32, i32))>,
}

pub fn point_captures_event_parser(input: &str) -> IResult<&str, PointCapturedEvent> {
    let (input, cp) = opt(param_parse_with("cp", u_int))(input)?;
    let (input, cp_name) = opt(param_parse("map"))(input)?;
    let (input, num_cappers) = opt(param_parse_with("numcappers", u_int))(input)?;

    let mut players = Vec::new();

    let mut params = ParamIter::new(input);
    match (params.next(), params.next()) {
        (Some((subject_key, subject)), Some((position_key, position_str)))
            if subject_key.starts_with("player") && position_key.starts_with("position") =>
        {
            let (_, subject) = subject_parser(subject)?;
            let (_, position) = position(position_str)?;
            players.push((subject, position));
        }
        _ => {}
    }

    Ok((
        input,
        PointCapturedEvent {
            cp: cp.map(|cp| cp as u8),
            num_cappers: num_cappers.map(|num| num as u8),
            cp_name,
            players,
        },
    ))
}

#[derive(Debug)]
pub struct CurrentScoreEvent {
    pub score: u8,
    pub player: u8,
}

pub fn current_score_event_parser(input: &str) -> IResult<&str, CurrentScoreEvent> {
    let (input, score) = param_parse_with("cp", u_int)(input)?;
    let (input, player) = param_parse_with("with", u_int)(input)?;
    Ok((
        input,
        CurrentScoreEvent {
            score: score as u8,
            player: player as u8,
        },
    ))
}

#[derive(Debug)]
pub struct GameOverEvent<'a> {
    pub reason: &'a str,
}

pub fn game_over_event_parser(input: &str) -> IResult<&str, GameOverEvent> {
    let (input, reason) = param_parse("reason")(input)?;
    Ok((input, GameOverEvent { reason }))
}

#[derive(Debug)]
pub struct FinalScoreEvent {
    pub score: u8,
    pub player: u8,
}

pub fn final_score_event_parser(input: &str) -> IResult<&str, FinalScoreEvent> {
    let (input, score) = param_parse_with("cp", u_int)(input)?;
    let (input, player) = param_parse_with("with", u_int)(input)?;
    Ok((
        input,
        FinalScoreEvent {
            score: score as u8,
            player: player as u8,
        },
    ))
}
