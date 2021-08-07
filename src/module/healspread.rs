use crate::common::{SteamId3, SubjectId};
use crate::event::healed_event_parser;
use crate::module::EventHandler;
use crate::raw_event::{RawEvent, RawEventType};
use crate::SubjectMap;
use serde::{Serialize, Serializer};
use std::collections::HashMap;
use std::convert::TryFrom;
use thiserror::Error;

impl Serialize for SteamId3 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.steam3().serialize(serializer)
    }
}

#[derive(Default)]
pub struct HealSpreadHandler(HashMap<SteamId3, HashMap<SteamId3, u32>>);

#[derive(Error, Debug)]
#[error("Invalid healed event: {0}")]
pub struct InvalidHealEvent(String);

impl EventHandler for HealSpreadHandler {
    type Output = HashMap<SteamId3, HashMap<SteamId3, u32>>;
    type Error = InvalidHealEvent;

    fn does_handle(&self, ty: RawEventType) -> bool {
        matches!(ty, RawEventType::Healed)
    }

    fn handle(
        &mut self,
        _time: u32,
        subject: SubjectId,
        event: &RawEvent,
    ) -> Result<(), Self::Error> {
        let healer_steam_id = if let Some(steam_id) = subject.steam_id() {
            steam_id
        } else {
            return Ok(());
        };
        if matches!(event.ty, RawEventType::Healed) {
            let (_, heal_event) = healed_event_parser(event.params)
                .map_err(|_| InvalidHealEvent(event.params.into()))?;
            if let Ok(target_subject) = SubjectId::try_from(&heal_event.subject) {
                if let Some(target_steam_id) = target_subject.steam_id() {
                    let healed = self
                        .0
                        .entry(SteamId3(healer_steam_id))
                        .or_default()
                        .entry(SteamId3(target_steam_id))
                        .or_default();
                    *healed += heal_event.amount
                }
            }
        }
        Ok(())
    }

    fn finish(self, _subjects: &SubjectMap) -> Self::Output {
        self.0
    }
}
