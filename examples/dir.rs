use main_error::MainError;
use std::env::args;
use std::ffi::OsStr;
use std::fs;
use tf_log_parser::parse;
use walkdir::WalkDir;

fn main() -> Result<(), MainError> {
    let path = args().nth(1).expect("No path provided");
    for entry in WalkDir::new(path) {
        let entry = entry?;
        let path = entry.path();
        if path.extension() == Some(OsStr::new("log")) {
            print!("{} - ", path.display());
            let input = match fs::read_to_string(path) {
                Ok(input) => input,
                Err(e) => {
                    println!("failed to read file: {}", e);
                    continue;
                }
            };
            let (output, _) = parse(&input)?;
            println!("{} messages", output.chat.len());
        }
    }

    Ok(())
}
