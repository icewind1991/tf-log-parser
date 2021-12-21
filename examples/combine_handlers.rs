//! Example for creating a handler that combines the output of multiple handlers

use main_error::MainError;
use std::env::args;
use std::fs;
use tf_log_parser::module::{ChatMessages, LobbySettingsHandler};
use tf_log_parser::{handler, parse_with_handler};

handler!(Handler {
    chat: ChatMessages,
    lobby_settings: LobbySettingsHandler
});

fn main() -> Result<(), MainError> {
    let path = args().nth(1).expect("No path provided");
    let content = fs::read_to_string(path)?;

    let (
        HandlerGlobalOutput {
            chat,
            lobby_settings,
        },
        _,
    ) = parse_with_handler::<Handler>(&content)?;

    if let Some(Ok(settings)) = lobby_settings {
        println!("Lobby settings: {:#?}", settings);
        println!();
    }

    for message in chat {
        println!("{}: {}", message.name, message.message);
    }
    Ok(())
}
