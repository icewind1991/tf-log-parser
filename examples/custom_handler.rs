//! Example of creating a custom handler to extract data from the log

use main_error::MainError;
use std::env::args;
use std::fs;
use tf_log_parser::event::DamageEvent;
use tf_log_parser::{
    parse_with_handler, EventHandler, GameEvent, RawEventType, SubjectData, SubjectId, SubjectMap,
};

struct HighestDamage {
    user: String,
    damage: u32,
}

#[derive(Default)]
struct HighestDamageHandler {
    current: Option<(SubjectId, u32)>,
}

impl EventHandler for HighestDamageHandler {
    type Output = Option<HighestDamage>;

    fn does_handle(&self, ty: RawEventType) -> bool {
        matches!(ty, RawEventType::Damage)
    }

    fn handle(&mut self, _time: u32, subject: SubjectId, event: &GameEvent) {
        if let GameEvent::Damage(DamageEvent {
            damage: Some(damage),
            ..
        }) = event
        {
            let damage = damage.get();
            match &mut self.current {
                Some((_, current_damage)) if damage > *current_damage => {
                    self.current = Some((subject, damage))
                }
                None => self.current = Some((subject, damage)),
                _ => {}
            }
        }
    }

    fn finish(self, subjects: &SubjectMap) -> Self::Output {
        self.current.map(|(subject, damage)| {
            let user = match &subjects[subject] {
                SubjectData::Player { name, .. } => name.clone(),
                _ => {
                    panic!("A non player did the most damage?")
                }
            };
            HighestDamage { user, damage }
        })
    }
}

fn main() -> Result<(), MainError> {
    let path = args().skip(1).next().expect("No path provided");
    let content = fs::read_to_string(path)?;

    let HighestDamage { user, damage } =
        parse_with_handler::<HighestDamageHandler>(&content)?.expect("nobody did any damage?");

    println!("highest damage was {} done by {}", user, damage);
    Ok(())
}
