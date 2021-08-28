use crate::{SubjectError, SubjectId};
use chrono::{DateTime, TimeZone, Utc};
use logos::{Lexer, Logos};
use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case, take_while};
use nom::character::complete::{digit1, one_of};
use nom::{Finish, IResult};
use std::convert::{TryFrom, TryInto};
use std::num::ParseIntError;

/// Event that has only been minimally parsed.
/// that way we can decide if we're interested in handling the event before parsing further
#[derive(Debug, PartialEq)]
pub struct RawEvent<'a> {
    pub date: RawDate<'a>,
    pub subject: RawSubject<'a>,
    pub ty: RawEventType,
    pub params: &'a str,
}

impl<'a> RawEvent<'a> {
    pub fn parse(line: &'a str) -> Result<Self, nom::error::Error<&'a str>> {
        let (_, event) = event_parser(line).finish()?;
        Ok(event)
    }
}

fn event_parser(input: &str) -> IResult<&str, RawEvent> {
    let (input, date) = date_parser(input)?;

    let (input, _) = tag(": ")(input)?;
    let (input, subject) = subject_parser(input)?;

    let (input, _) = tag(" ")(input)?;
    let (input, ty) = event_type_parser(input)?;

    Ok((
        input,
        RawEvent {
            date,
            subject,
            ty,
            params: input.trim(),
        },
    ))
}

#[derive(Debug, PartialEq)]
pub struct RawDate<'a> {
    pub month: &'a str,
    pub day: &'a str,
    pub year: &'a str,
    pub hour: &'a str,
    pub minutes: &'a str,
    pub seconds: &'a str,
}

impl<'a> TryFrom<&RawDate<'a>> for DateTime<Utc> {
    type Error = ParseIntError;

    fn try_from(value: &RawDate<'a>) -> Result<Self, Self::Error> {
        Ok(Utc
            .ymd(
                value.year.parse()?,
                value.month.parse()?,
                value.day.parse()?,
            )
            .and_hms(
                value.hour.parse()?,
                value.minutes.parse()?,
                value.seconds.parse()?,
            ))
    }
}

fn date_parser(input: &str) -> IResult<&str, RawDate> {
    let (input, month) = digit1(input)?;
    let (input, _) = tag("/")(input)?;
    let (input, day) = digit1(input)?;
    let (input, _) = tag("/")(input)?;
    let (input, year) = digit1(input)?;

    let (input, _) = tag(" - ")(input)?;

    let (input, hour) = digit1(input)?;
    let (input, _) = tag(":")(input)?;
    let (input, minutes) = digit1(input)?;
    let (input, _) = tag(":")(input)?;
    let (input, seconds) = digit1(input)?;
    Ok((
        input,
        RawDate {
            month,
            day,
            year,
            hour,
            minutes,
            seconds,
        },
    ))
}

#[derive(Debug, PartialEq)]
pub enum RawSubject<'a> {
    Player(&'a str),
    Team(&'a str),
    System(&'a str),
    Console,
    World,
}

impl<'a> RawSubject<'a> {
    pub fn id(&self) -> Result<SubjectId, SubjectError> {
        self.try_into()
    }
}

fn subject_parser_world(input: &str) -> IResult<&str, RawSubject> {
    let (input, _) = tag("World")(input)?;
    Ok((input, RawSubject::World))
}

fn subject_parser_console(input: &str) -> IResult<&str, RawSubject> {
    let (input, _) = tag(r#""Console<0><Console><Console>""#)(input)?;
    Ok((input, RawSubject::Console))
}

fn subject_parser_team(input: &str) -> IResult<&str, RawSubject> {
    let (input, _) = tag(r#"Team ""#)(input)?;

    let (input, team) = alt((tag_no_case("red"), tag_no_case("blue")))(input)?;

    let (input, _) = one_of("\"")(input)?;
    Ok((input, RawSubject::Team(team)))
}

fn subject_parser_system_bracket(input: &str) -> IResult<&str, &str> {
    let (input, _) = tag("[")(input)?;

    let (input, name) = take_while(|c| c != ']')(input)?;

    let (input, _) = tag("]")(input)?;
    IResult::Ok((input, name))
}

fn subject_parser_system(input: &str) -> IResult<&str, RawSubject> {
    let (input, name) = alt((subject_parser_system_bracket, tag("Log"), tag("Tournament")))(input)?;
    Ok((input, RawSubject::System(name)))
}

pub fn split_player_subject(input: &str) -> IResult<&str, (&str, &str, &str, &str)> {
    let (input, name) = take_while(|c| c != '<')(input)?;

    let (input, _) = one_of("<")(input)?;
    let (input, user_id) = digit1(input)?;
    let (input, _) = one_of(">")(input)?;

    let (input, _) = one_of("<")(input)?;
    let (input, steam_id) = take_while(|c| c != '>')(input)?;
    let (input, _) = one_of(">")(input)?;

    let (input, _) = one_of("<")(input)?;
    let (input, team) = take_while(|c| c != '>')(input)?;
    let (input, _) = one_of(">")(input)?;

    Ok((input, (name, user_id, steam_id, team)))
}

fn subject_parser_player(input: &str) -> IResult<&str, RawSubject> {
    let (input, _) = one_of("\"")(input)?;

    let (input, subject) = take_while(|c| c != '"')(input)?;

    let (input, _) = one_of("\"")(input)?;

    Ok((input, RawSubject::Player(subject)))
}

pub fn subject_parser(input: &str) -> IResult<&str, RawSubject> {
    alt((
        subject_parser_console,
        subject_parser_player,
        subject_parser_world,
        subject_parser_team,
        subject_parser_system,
    ))(input)
}

#[derive(Copy, Clone, Debug, PartialEq, Logos)]
pub enum RawEventType {
    #[token(r#"joined"#)]
    Joined,
    #[token(r#"changed role"#)]
    ChangedRole,
    #[token(r#"triggered "shot_fired""#)]
    ShotFired,
    #[token(r#"triggered "shot_hit""#)]
    ShotHit,
    #[token(r#"triggered "damage""#)]
    Damage,
    #[token(r#"triggered "healed""#)]
    Healed,
    #[token(r#"triggered "first_heal_after_spawn""#)]
    FirstHealAfterSpawn,
    #[token(r#"killed"#)]
    Killed,
    #[token(r#"triggered "kill assist""#)]
    KillAssist,
    #[token(r#"committed suicide"#)]
    Suicide,
    #[token(r#"triggered "domination""#)]
    Domination,
    #[token(r#"triggered "revenge""#)]
    Revenge,
    #[token(r#"spawned"#)]
    Spawned,
    #[token(r#"say_team"#)]
    SayTeam,
    #[token(r#"say"#)]
    Say,
    #[token(r#"triggered "empty_uber""#)]
    EmptyUber,
    #[token(r#"triggered "player_builtobject""#)]
    PlayerBuiltObject,
    #[token(r#"triggered "player_dropobject""#)]
    PlayerCarryObject,
    #[token(r#"triggered "player_carryobject""#)]
    PlayerDropObject,
    #[token(r#"triggered "killedobject""#)]
    PlayerKilledObject,
    #[token(r#"triggered "object_detonated""#)]
    PlayerExtinguished,
    #[token(r#"triggered "player_extinguished""#)]
    ObjectDetonated,
    #[token(r#"picked up"#)]
    PickedUp,
    #[token(r#"triggered "medic_death""#)]
    MedicDeath,
    #[token(r#"triggered "medic_death_ex""#)]
    MedicDeathEx,
    #[token(r#"triggered "chargeended""#)]
    ChargeEnd,
    #[token(r#"triggered "chargeready""#)]
    ChargeReady,
    #[token(r#"triggered "chargedeployed""#)]
    ChargeDeployed,
    #[token(r#"triggered "lost_uber_advantage""#)]
    UberAdvantageLost,
    #[token(r#"triggered "Round_Start""#)]
    RoundStart,
    #[token(r#"triggered "Round_Setup_Begin""#)]
    RoundSetupBegin,
    #[token(r#"triggered "Round_Setup_End""#)]
    RoundSetupEnd,
    #[token(r#"triggered "Mini_Round_Selected""#)]
    MiniRoundSelected,
    #[token(r#"triggered "Mini_Round_Start""#)]
    MiniRoundStart,
    #[token(r#"triggered "Round_Win""#)]
    RoundWin,
    #[token(r#"triggered "Mini_Round_Win""#)]
    MiniRoundWin,
    #[token(r#"triggered "Round_Length""#)]
    RoundLength,
    #[token(r#"triggered "Mini_Round_Length""#)]
    MiniRoundLength,
    #[token(r#"triggered "Round_Overtime""#)]
    RoundOvertime,
    #[token(r#"triggered "pointcaptured""#)]
    PointCaptured,
    #[token(r#"triggered "captureblocked""#)]
    CaptureBlocked,
    #[token(r#"triggered "Game_Over""#)]
    GameOver,
    #[token(r#"current"#)]
    CurrentScore,
    #[token(r#"final"#)]
    FinalScore,
    #[token(r#"triggered "Intermission_Win_Limit""#)]
    WinLimit,
    #[token(r#"triggered "Game_Paused""#)]
    Paused,
    #[token(r#"triggered "Game_Unpaused""#)]
    UnPaused,
    #[token(r#"Request: "#)]
    Request,
    #[token(r#"Response: "#)]
    Response,
    #[token(r#"connected,"#)]
    Connected,
    #[token(r#"disconnected"#)]
    Disconnected,
    #[token(r#"STEAM USERID validated"#)]
    SteamIdValidated,
    #[token(r#"entered the game"#)]
    Entered,
    #[token(r#"file started"#)]
    LogFileStarted,
    #[token(r#"file closed"#)]
    LogFileClosed,
    #[token(r#"The log might have not been uploaded."#)]
    NotUploaded,
    #[token(r#"mode started"#)]
    TournamentStart,
    #[token(r#"triggered "flagevent""#)]
    FlagEvent,
    #[error]
    Unknown,
}

fn event_type_parser(input: &str) -> IResult<&str, RawEventType> {
    let mut lexer = Lexer::new(input);
    let ty = lexer.next().unwrap_or(RawEventType::Unknown);
    Ok((lexer.remainder(), ty))
}

#[test]
fn test_parse_raw() {
    let input =
        r#"08/06/2018 - 21:13:57: "makxbi<27><[U:1:40364391]><Red>" changed role to "sniper""#;
    let raw = RawEvent::parse(input).unwrap();
    assert_eq!(
        RawEvent {
            date: RawDate {
                month: "08",
                day: "06",
                year: "2018",
                hour: "21",
                minutes: "13",
                seconds: "57",
            },
            subject: RawSubject::Player("makxbi<27><[U:1:40364391]><Red>"),
            ty: RawEventType::ChangedRole,
            params: r#"to "sniper""#,
        },
        raw
    );
}

#[test]
fn test_parse_all_valid() {
    use std::io::Read;

    let files = [
        "test_data/log_6s.log",
        "test_data/log_hl.log",
        "test_data/log_bball.log",
        "test_data/log_2788889.log",
        "test_data/log_2892242.log",
    ];
    let mut buff = String::new();

    for file in files {
        buff.clear();
        std::fs::File::open(file)
            .unwrap()
            .read_to_string(&mut buff)
            .unwrap();
        for line in buff.trim().split("L ").filter(|line| !line.is_empty()) {
            if line.starts_with("L ") {
                RawEvent::parse(line).unwrap();
            }
        }
    }
}
