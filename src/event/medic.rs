use crate::event::{param_parse_with, parse_field, quoted, ParamIter};
use crate::raw_event::RawSubject;
use crate::{Event, IResult};

#[derive(Debug, Event)]
pub struct HealedEvent<'a> {
    #[event(name = "against")]
    pub target: RawSubject<'a>,
    #[event(name = "healing")]
    #[event(quoted)]
    pub amount: u32,
}

#[derive(Debug, Event)]
pub struct ChargeDeployedEvent<'a> {
    pub medigun: Option<&'a str>,
}

#[derive(Debug, Event)]
pub struct ChargeEndedEvent {
    pub duration: Option<f32>,
}

#[derive(Debug, Event)]
pub struct AdvantageLostEvent {
    pub time: Option<f32>,
}

#[derive(Debug, Event)]
pub struct FirstHealEvent {
    pub time: Option<f32>,
}

#[derive(Debug, Event)]
pub struct MedicDeathEvent {
    #[event(name = "ubercharge")]
    pub charge: Option<u32>,
}

#[derive(Debug, Event)]
pub struct MedicDeathExEvent {
    pub charge_percentage: Option<u8>,
}
