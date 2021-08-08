use crate::{SubjectError, SubjectId};
use chrono::{DateTime, TimeZone, Utc};
use enum_iterator::IntoEnumIterator;
use nom::branch::alt;
use nom::bytes::complete::{tag, take_while};
use nom::character::complete::{digit1, one_of};
use nom::error::{make_error, ErrorKind};
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
    let (input, _) = tag("L ")(input)?;
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
    Player {
        name: &'a str,
        user_id: &'a str,
        steam_id: &'a str,
        team: &'a str,
    },
    Team(&'a str),
    System(&'a str),
    Console,
    World,
}

impl<'a> RawSubject<'a> {
    pub fn name(&self) -> &'a str {
        match self {
            RawSubject::Player { name, .. } => name,
            RawSubject::Team(team) => team,
            RawSubject::System(system) => system,
            RawSubject::Console => "Console",
            RawSubject::World => "World",
        }
    }

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

    let (input, team) = alt((tag("Red"), tag("Blue")))(input)?;

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

fn subject_parser_player(input: &str) -> IResult<&str, RawSubject> {
    let (input, _) = one_of("\"")(input)?;

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

    let (input, _) = one_of("\"")(input)?;

    Ok((
        input,
        RawSubject::Player {
            name,
            user_id,
            steam_id,
            team,
        },
    ))
}

pub fn subject_parser(input: &str) -> IResult<&str, RawSubject> {
    alt((
        subject_parser_console,
        subject_parser_world,
        subject_parser_team,
        subject_parser_player,
        subject_parser_system,
    ))(input)
}

#[derive(IntoEnumIterator, Copy, Clone, Debug, PartialEq)]
pub enum RawEventType {
    JoinedTeam,
    ChangedRole,
    ShotFired,
    ShotHit,
    Damage,
    Healed,
    FirstHealAfterSpawn,
    Killed,
    KillAssist,
    Suicide,
    Domination,
    Revenge,
    Spawned,
    SayTeam,
    Say,
    EmptyUber,
    PlayerBuiltObject,
    PlayerCarryObject,
    PlayerDropObject,
    PlayerKilledObject,
    PlayerExtinguished,
    ObjectDetonated,
    PickedUpItem,
    MedicDeath,
    MedicDeathEx,
    ChargeEnd,
    ChargeReady,
    ChargeDeployed,
    UberAdvantageLost,
    RoundStart,
    RoundSetupBegin,
    RoundSetupEnd,
    MiniRoundSelected,
    MiniRoundStart,
    RoundWin,
    MiniRoundWin,
    RoundLength,
    MiniRoundLength,
    RoundOvertime,
    PointCaptured,
    CaptureBlocked,
    GameOver,
    CurrentScore,
    FinalScore,
    WinLimit,
    Paused,
    UnPaused,
    Request,
    Response,
    Connected,
    Disconnected,
    SteamIdValidated,
    Entered,
    LogFileStarted,
    LogFileClosed,
    NotUploaded,
    TournamentStart,
    FlagEvent,
}

impl RawEventType {
    pub fn tag(self) -> &'static str {
        match self {
            RawEventType::JoinedTeam => r#"joined team"#,
            RawEventType::ChangedRole => r#"changed role"#,
            RawEventType::ShotFired => r#"triggered "shot_fired""#,
            RawEventType::ShotHit => r#"triggered "shot_hit""#,
            RawEventType::Damage => r#"triggered "damage""#,
            RawEventType::Healed => r#"triggered "healed""#,
            RawEventType::FirstHealAfterSpawn => r#"triggered "first_heal_after_spawn""#,
            RawEventType::Killed => r#"killed"#,
            RawEventType::KillAssist => r#"triggered "kill assist""#,
            RawEventType::Suicide => r#"committed suicide"#,
            RawEventType::Domination => r#"triggered "domination""#,
            RawEventType::Revenge => r#"triggered "revenge""#,
            RawEventType::Spawned => r#"spawned"#,
            RawEventType::SayTeam => r#"say_team"#,
            RawEventType::Say => r#"say"#,
            RawEventType::EmptyUber => r#"triggered "empty_uber""#,
            RawEventType::PlayerBuiltObject => r#"triggered "player_builtobject""#,
            RawEventType::PlayerDropObject => r#"triggered "player_dropobject""#,
            RawEventType::PlayerCarryObject => r#"triggered "player_carryobject""#,
            RawEventType::PlayerKilledObject => r#"triggered "killedobject""#,
            RawEventType::ObjectDetonated => r#"triggered "object_detonated""#,
            RawEventType::PlayerExtinguished => r#"triggered "player_extinguished""#,
            RawEventType::PickedUpItem => r#"picked up item"#,
            RawEventType::MedicDeath => r#"triggered "medic_death""#,
            RawEventType::MedicDeathEx => r#"triggered "medic_death_ex""#,
            RawEventType::ChargeEnd => r#"triggered "chargeended""#,
            RawEventType::ChargeReady => r#"triggered "chargeready""#,
            RawEventType::ChargeDeployed => r#"triggered "chargedeployed""#,
            RawEventType::UberAdvantageLost => r#"triggered "lost_uber_advantage""#,
            RawEventType::RoundStart => r#"triggered "Round_Start""#,
            RawEventType::RoundSetupBegin => r#"triggered "Round_Setup_Begin""#,
            RawEventType::RoundSetupEnd => r#"triggered "Round_Setup_End""#,
            RawEventType::MiniRoundSelected => r#"triggered "Mini_Round_Selected""#,
            RawEventType::MiniRoundStart => r#"triggered "Mini_Round_Start""#,
            RawEventType::RoundWin => r#"triggered "Round_Win""#,
            RawEventType::MiniRoundWin => r#"triggered "Mini_Round_Win""#,
            RawEventType::RoundLength => r#"triggered "Round_Length""#,
            RawEventType::MiniRoundLength => r#"triggered "Mini_Round_Length""#,
            RawEventType::RoundOvertime => r#"triggered "Round_Overtime""#,
            RawEventType::PointCaptured => r#"triggered "pointcaptured""#,
            RawEventType::CaptureBlocked => r#"triggered "captureblocked""#,
            RawEventType::GameOver => r#"triggered "Game_Over""#,
            RawEventType::CurrentScore => r#"current score"#,
            RawEventType::FinalScore => r#"final score"#,
            RawEventType::WinLimit => r#"triggered "Intermission_Win_Limit""#,
            RawEventType::Paused => r#"triggered "Game_Paused""#,
            RawEventType::UnPaused => r#"triggered "Game_Unpaused""#,
            RawEventType::Request => r#"Request: "#,
            RawEventType::Response => r#"Response: "#,
            RawEventType::Connected => r#"connected,"#,
            RawEventType::Disconnected => r#"disconnected"#,
            RawEventType::SteamIdValidated => r#"STEAM USERID validated"#,
            RawEventType::Entered => r#"entered the game"#,
            RawEventType::LogFileStarted => r#"file started"#,
            RawEventType::LogFileClosed => r#"file closed"#,
            RawEventType::NotUploaded => r#"The log might have not been uploaded."#,
            RawEventType::TournamentStart => r#"mode started"#,
            RawEventType::FlagEvent => r#"triggered "flagevent""#,
        }
    }
}

fn event_type_parser(input: &str) -> IResult<&str, RawEventType> {
    for event_type in RawEventType::into_enum_iter() {
        if let Ok((input, _ty)) = tag::<_, _, nom::error::Error<&str>>(event_type.tag())(input) {
            return Ok((input, event_type));
        }
    }
    dbg!(input);
    Err(nom::Err::Error(make_error(input, ErrorKind::NoneOf)))
}

#[test]
fn test_parse_raw() {
    let input =
        r#"L 08/06/2018 - 21:13:57: "makxbi<27><[U:1:40364391]><Red>" changed role to "sniper""#;
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
            subject: RawSubject::Player {
                name: "makxbi",
                user_id: "27",
                steam_id: "[U:1:40364391]",
                team: "Red",
            },
            ty: RawEventType::ChangedRole,
            params: r#""sniper""#,
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
        for line in buff.trim().lines() {
            if line.starts_with("L ") {
                RawEvent::parse(line).unwrap();
            }
        }
    }
}
