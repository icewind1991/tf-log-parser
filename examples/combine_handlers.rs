//! Example for creating a handler that combines the output of multiple handlers

use main_error::MainError;
use std::env::args;
use std::fs;
use tf_log_parser::module::{ChatHandler, LobbySettingsHandler};
use tf_log_parser::{handler, parse_with_handler};

handler!(Handler {
    chat: ChatHandler,
    lobby_settings: LobbySettingsHandler
});

fn main() -> Result<(), MainError> {
    let path = args().skip(1).next().expect("No path provided");
    let content = fs::read_to_string(path)?;

    let HandlerOutput {
        chat,
        lobby_settings,
    } = parse_with_handler::<Handler>(&content)?;

    if let Ok(Some(settings)) = lobby_settings {
        println!("Lobby settings: {:#?}", settings);
        println!();
    }

    for message in chat {
        println!("{}: {}", message.name, message.message);
    }
    Ok(())
}
