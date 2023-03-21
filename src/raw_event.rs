use crate::common::Team;
use crate::parsing::{skip, skip_matches, split_once, split_subject_end};
use crate::{Error, Result};
use crate::{SubjectError, SubjectId};
use chrono::{NaiveDate, NaiveDateTime};
use logos::{Lexer, Logos};
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
    pub fn parse(line: &'a str) -> Result<Self> {
        event_parser(line)
    }
}

fn event_parser(input: &str) -> Result<RawEvent> {
    // println!("{}", input);
    if input.len() < 24 {
        return Err(Error::Skip);
    }
    let date = RawDate(&input[0..21]);

    let (input, subject) = subject_parser(&input[23..])?;

    let (input, ty) = event_type_parser(input)?;

    let params = skip_matches(input, b' ').0;

    Ok(RawEvent {
        date,
        subject,
        ty,
        params,
    })
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct RawDate<'a>(&'a str);

impl<'a> TryFrom<RawDate<'a>> for NaiveDateTime {
    type Error = ParseIntError;

    fn try_from(RawDate(raw): RawDate<'a>) -> Result<Self, Self::Error> {
        Ok(
            NaiveDate::from_ymd(raw[6..10].parse()?, raw[0..2].parse()?, raw[3..5].parse()?)
                .and_hms(
                    raw[13..15].parse()?,
                    raw[16..18].parse()?,
                    raw[19..21].parse()?,
                ),
        )
    }
}

#[test]
fn test_parse_date() {
    let raw = RawDate("08/06/2018 - 21:13:57");
    assert_eq!(
        NaiveDate::from_ymd(2018, 08, 06).and_hms(21, 13, 57),
        raw.try_into().unwrap()
    );
}

#[derive(Debug, PartialEq)]
pub enum RawSubject<'a> {
    Player(&'a str),
    Team(Team),
    System(&'a str),
    Console,
    World,
}

impl Default for RawSubject<'static> {
    fn default() -> Self {
        RawSubject::System("unknown")
    }
}

impl<'a> RawSubject<'a> {
    pub fn id(&self) -> Result<SubjectId, SubjectError> {
        self.try_into()
    }
}

pub fn split_player_subject(input: &str) -> Result<(&str, &str, &str, &str)> {
    let mut parts = input.rsplitn(4, '<');
    let (name, user_id, steam_id, team) =
        if let (Some(team), Some(steam_id), Some(user_id), Some(name)) =
            (parts.next(), parts.next(), parts.next(), parts.next())
        {
            if steam_id.is_empty() || user_id.is_empty() || team.is_empty() {
                (name, "0", "", "")
            } else {
                (
                    name,
                    user_id
                        .get(0..user_id.len() - 1)
                        .or_else(|| {
                            println!("{}", input);
                            None
                        })
                        .expect("asd"),
                    &steam_id[0..steam_id.len() - 1],
                    &team[0..team.len() - 1],
                )
            }
        } else {
            return Err(Error::Incomplete);
        };

    Ok((name, user_id, steam_id, team))
}

#[test]
fn test_split_player_subject() {
    assert_eq!(
        ("Fin", "4", "[U:1:129852188]", "Blue"),
        split_player_subject("Fin<4><[U:1:129852188]><Blue>").unwrap()
    );
    assert_eq!(
        ("Electra<3", "8", "[U:1:104485840]", "Red"),
        split_player_subject("Electra<3<8><[U:1:104485840]><Red>").unwrap()
    );
    assert_eq!(
        ("sorry, squidie", "15", "[U:1:83437541]", "Red"),
        split_player_subject("sorry, squidie<15><[U:1:83437541]><Red>").unwrap()
    );
}

pub fn against_subject_parser(input: &str) -> Result<RawSubject> {
    // "against" fields are always players, and unquoted
    if input.ends_with("le>") {
        Ok(RawSubject::Console)
    } else {
        Ok(RawSubject::Player(input))
    }
}

pub fn subject_parser(input: &str) -> Result<(&str, RawSubject)> {
    let full = input;
    if let Some(input) = input.strip_prefix('"') {
        let Ok((player, input)) = split_subject_end(input, 1) else {
            return Ok((full, RawSubject::Console))
        };
        let input = skip(input, 1)?;
        if player.ends_with("le>") {
            Ok((input, RawSubject::Console))
        } else {
            Ok((input, RawSubject::Player(player)))
        }
    } else if input.starts_with("Te") {
        // Team "red" or Team "blue"
        if &input[6..7] == "r" {
            Ok((&input[11..], RawSubject::Team(Team::Red)))
        } else if &input[6..7] == "b" {
            Ok((&input[12..], RawSubject::Team(Team::Blue)))
        } else {
            let (_, input) = split_once(&input[7..], b'"', 1)?;
            let input = skip(input, 1)?;
            Ok((input, RawSubject::Team(Team::Spectator)))
        }
    } else {
        let Ok((system, input)) = split_once(input, b' ', 1) else {
            return Ok(("", RawSubject::System(input)))
        };
        Ok((input, RawSubject::System(system)))
    }
}

#[test]
fn test_subject_parser() {
    assert_eq!(
        (
            "connected",
            RawSubject::Player(r#"Buddie :")<25><[U:1:123]><>"#)
        ),
        subject_parser(r#""Buddie :")<25><[U:1:123]><>" connected"#).unwrap()
    );
}

#[derive(Copy, Clone, Debug, PartialEq, Logos)]
pub enum RawEventType {
    #[token(r#"joined "#)]
    Joined,
    #[token(r#"changed role "#)]
    RoleChange,
    #[token(r#"triggered "shot_fired""#)]
    ShotFired,
    #[token(r#"triggered "shot_hit""#)]
    ShotHit,
    #[token(r#"triggered "damage""#)]
    Damage,
    #[token(r#"triggered "healed""#)]
    Healed,
    #[token(r#"triggered "first_heal_after_spawn""#)]
    FirstHeal,
    #[token(r#"killed "#)]
    Killed,
    #[token(r#"triggered "kill assist""#)]
    KillAssist,
    #[token(r#"committed suicide "#)]
    Suicide,
    #[token(r#"triggered "domination""#)]
    Domination,
    #[token(r#"triggered "revenge""#)]
    Revenge,
    #[token(r#"spawned "#)]
    Spawned,
    #[token(r#"say_team "#)]
    SayTeam,
    #[token(r#"say "#)]
    Say,
    #[token(r#"triggered "empty_uber""#)]
    EmptyUber,
    #[token(r#"triggered "player_builtobject""#)]
    BuiltObject,
    #[token(r#"triggered "player_dropobject""#)]
    CarryObject,
    #[token(r#"triggered "player_carryobject""#)]
    DropObject,
    #[token(r#"triggered "rocket_jump""#)]
    RocketJump,
    #[token(r#"triggered "killedobject""#)]
    KilledObject,
    #[token(r#"triggered "object_detonated""#)]
    ObjectDetonated,
    #[token(r#"triggered "player_extinguished""#)]
    Extinguished,
    #[token(r#"picked up "#)]
    PickedUp,
    #[token(r#"triggered "medic_death""#)]
    MedicDeath,
    #[token(r#"triggered "medic_death_ex""#)]
    MedicDeathEx,
    #[token(r#"triggered "chargeended""#)]
    ChargeEnded,
    #[token(r#"triggered "chargeready""#)]
    ChargeReady,
    #[token(r#"triggered "chargedeployed""#)]
    ChargeDeployed,
    #[token(r#"triggered "lost_uber_advantage""#)]
    AdvantageLost,
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
    RoundOverTime,
    #[token(r#"triggered "pointcaptured""#)]
    PointCaptured,
    #[token(r#"triggered "captureblocked""#)]
    CaptureBlocked,
    #[token(r#"triggered "Game_Over""#)]
    GameOver,
    #[token(r#"current "#)]
    CurrentScore,
    #[token(r#"final "#)]
    FinalScore,
    #[token(r#"triggered "Intermission_Win_Limit""#)]
    WinLimit,
    #[token(r#"triggered "Game_Paused""#)]
    Paused,
    #[token(r#"triggered "Game_Unpaused""#)]
    UnPaused,
    #[token(r#"Request:  "#)]
    Request,
    #[token(r#"Response:  "#)]
    Response,
    #[token(r#"connected, "#)]
    Connected,
    #[token(r#"disconnected "#)]
    Disconnect,
    #[token(r#"STEAM USERID validated "#)]
    SteamIdValidated,
    #[token(r#"entered the game "#)]
    Entered,
    #[token(r#"file started "#)]
    LogFileStarted,
    #[token(r#"file closed "#)]
    LogFileClosed,
    #[token(r#"The log might have not been uploaded. "#)]
    NotUploaded,
    #[token(r#"mode started "#)]
    TournamentModeStarted,
    #[token(r#"triggered "flagevent""#)]
    FlagEvent,
    #[token(r#"cvars "#)]
    CVars,
    #[error]
    Unknown,
}

fn event_type_parser(input: &str) -> Result<(&str, RawEventType)> {
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
            date: RawDate("08/06/2018 - 21:13:57"),
            subject: RawSubject::Player("makxbi<27><[U:1:40364391]><Red>"),
            ty: RawEventType::RoleChange,
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
