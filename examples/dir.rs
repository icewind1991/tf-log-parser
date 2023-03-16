use main_error::MainError;
use std::env::args;
use std::ffi::OsStr;
use std::fs;
use std::io::stdout;
use std::io::Write;
use std::time::{Duration, Instant};
use tf_log_parser::parse;
use walkdir::WalkDir;

fn main() -> Result<(), MainError> {
    let path = args().nth(1).expect("No path provided");

    let mut parse_time = Duration::default();
    let mut count = 0;
    let start = Instant::now();

    let mut stdout = stdout().lock();

    for entry in WalkDir::new(path) {
        let entry = entry?;
        let path = entry.path();
        if path.extension() == Some(OsStr::new("log")) {
            let _ = write!(&mut stdout, "{} - ", path.display());
            let input = match fs::read_to_string(path) {
                Ok(input) => input,
                Err(e) => {
                    println!("failed to read file: {}", e);
                    continue;
                }
            };
            let parse_start = Instant::now();
            let (output, _) = parse(&input)?;
            parse_time += parse_start.elapsed();
            count += 1;
            let _ = writeln!(&mut stdout, "{} messages", output.chat.len());
        }
    }

    let total = start.elapsed();

    println!(
        "Parsed {} in {:01}s with {:01}s of IO overhead",
        count,
        parse_time.as_secs_f32(),
        total.as_secs_f32()
    );

    Ok(())
}
