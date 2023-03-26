use crate::event::GameEventError;
use crate::SubjectError;
use std::num::ParseIntError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Malformed logfile")]
    Malformed,
    #[error("Incomplete logfile")]
    Incomplete,
    #[error("Incomplete logfile")]
    Skip,
    #[error("Malformed subject: {0}")]
    Subject(Box<SubjectError>),
    #[error("{0}")]
    MalformedEvent(Box<GameEventError>),
}

impl From<SubjectError> for Error {
    fn from(value: SubjectError) -> Self {
        Error::Subject(Box::new(value))
    }
}

impl From<GameEventError> for Error {
    fn from(value: GameEventError) -> Self {
        Error::MalformedEvent(Box::new(value))
    }
}

impl From<ParseIntError> for Error {
    fn from(_: ParseIntError) -> Self {
        Error::Malformed
    }
}

pub type Result<O, E = Error> = std::result::Result<O, E>;

#[doc(hidden)]
pub type IResult<'a, O, E = Error> = std::result::Result<(&'a str, O), E>;

pub trait ResultExt: Sized {
    fn skip_incomplete(self) -> Self;
}

impl<T> ResultExt for Result<T> {
    fn skip_incomplete(self) -> Self {
        self.map_err(|e| match e {
            Error::Incomplete => Error::Skip,
            e => e,
        })
    }
}
