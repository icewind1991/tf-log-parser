use crate::common::Team;
use crate::{Error, Result};
use crate::{SubjectError, SubjectId};
use chrono::{NaiveDate, NaiveDateTime};
use logos::{Lexer, Logos};
use nom::{IResult, Needed};
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
    let date = RawDate(&input[0..21]);

    let (input, subject) = subject_parser(&input[23..])?;

    let (input, ty) = event_type_parser(input.trim_start())?;

    Ok(RawEvent {
        date,
        subject,
        ty,
        params: input.trim(),
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

impl<'a> RawSubject<'a> {
    pub fn id(&self) -> Result<SubjectId, SubjectError> {
        self.try_into()
    }
}

pub fn split_player_subject(input: &str) -> Result<(&str, &str, &str, &str)> {
    let mut parts = input.splitn(4, '<');
    let (name, user_id, steam_id, team) =
        if let (Some(name), Some(user_id), Some(steam_id), Some(team)) =
            (parts.next(), parts.next(), parts.next(), parts.next())
        {
            (
                name,
                &user_id[0..user_id.len() - 1],
                &steam_id[0..steam_id.len() - 1],
                &team[0..team.len() - 1],
            )
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
    )
}

pub fn against_subject_parser(input: &str) -> IResult<&str, RawSubject> {
    subject_parser(input).map_err(|_| nom::Err::Incomplete(Needed::Unknown))
}

pub fn subject_parser(input: &str) -> Result<(&str, RawSubject)> {
    if let Some(input) = input.strip_prefix('"') {
        let (player, input) = input.split_once('"').ok_or(Error::Incomplete)?;
        if player.ends_with("e>") {
            Ok((input, RawSubject::Console))
        } else {
            Ok((input, RawSubject::Player(player)))
        }
    } else if input.starts_with("Te") {
        // Team "red" or Team "blue"
        if &input[6..7] == "r" {
            Ok((&input[10..], RawSubject::Team(Team::Red)))
        } else if &input[6..7] == "b" {
            Ok((&input[11..], RawSubject::Team(Team::Blue)))
        } else {
            let (_, input) = input[7..].split_once('"').ok_or(Error::Incomplete)?;
            Ok((input, RawSubject::Team(Team::Spectator)))
        }
    } else {
        let (system, input) = input.split_once(' ').ok_or(Error::Incomplete)?;
        Ok((input, RawSubject::System(system)))
    }
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
