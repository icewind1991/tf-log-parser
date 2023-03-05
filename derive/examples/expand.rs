use tf_log_parser::{
    event::{param_parse, parse_field, quoted, ParamIter},
    Event, IResult,
};

#[derive(Event)]
pub struct DamageEvent<'a> {
    #[event(name = "against")]
    pub target: RawSubject<'a>,
    pub damage: Option<NonZeroU32>,
    #[event(name = "realdamage")]
    pub real_damage: Option<NonZeroU32>,
    pub weapon: Option<&'a str>,
}

#[derive(Event)]
pub struct ShotFiredEvent {
    #[event(quoted)]
    pub weapon: u32,
    pub damage: Option<u32>,
}

pub fn main() {}
