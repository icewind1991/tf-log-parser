use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use main_error::MainError;
use rayon::prelude::*;
use std::env::args;
use std::ffi::OsStr;
use std::fs;
use std::hint::black_box;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use tf_log_parser::parse;
use walkdir::WalkDir;

fn main() -> Result<(), MainError> {
    let path = args().nth(1).expect("No path provided");

    let parse_time = AtomicUsize::default();
    let read_time = AtomicUsize::default();
    let count = AtomicUsize::default();
    let start = Instant::now();

    let style = ProgressStyle::with_template(
        "[{elapsed_precise}] {wide_bar:.cyan/blue} {per_sec:!} {pos:>7}/{len:7}",
    )
    .unwrap();

    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(Duration::from_millis(120));
    pb.set_style(
        ProgressStyle::with_template("[{elapsed_precise}] {spinner:.blue} {msg}")
            .unwrap()
            .tick_strings(&[
                "▹▹▹▹▹",
                "▸▹▹▹▹",
                "▹▸▹▹▹",
                "▹▹▸▹▹",
                "▹▹▹▸▹",
                "▹▹▹▹▸",
                "▪▪▪▪▪",
            ]),
    );
    pb.set_message("Collecting paths...");

    let paths: Vec<_> = WalkDir::new(path)
        .into_iter()
        .flatten()
        .filter(|entry| entry.path().extension() == Some(OsStr::new("log")))
        .collect();

    pb.finish_with_message("Done");

    let bar = ProgressBar::new(paths.len() as u64);
    bar.set_style(style);

    paths.into_par_iter().progress_with(bar).for_each(|entry| {
        let path = entry.path();
        if path.extension() == Some(OsStr::new("log")) {
            // let _ = print!("{} - ", path.display());
            let read_start = Instant::now();
            let input = match fs::read_to_string(path) {
                Ok(input) => input,
                Err(_e) => {
                    // println!("failed to read file: {}", e);
                    return;
                }
            };

            read_time.fetch_add(read_start.elapsed().as_micros() as usize, Ordering::Relaxed);
            let parse_start = Instant::now();
            let (output, _) = match parse(&input) {
                Ok(val) => val,
                Err(e) => {
                    eprintln!("failed to parse {}: {}", path.display(), e);
                    return;
                }
            };

            parse_time.fetch_add(
                parse_start.elapsed().as_micros() as usize,
                Ordering::Relaxed,
            );
            count.fetch_add(1, Ordering::Relaxed);
            black_box(output);
            // let _ = println!("{} messages", output.chat.len());
        }
    });

    let total = start.elapsed();

    let read_time = read_time.load(Ordering::Relaxed);
    let parse_time = parse_time.load(Ordering::Relaxed);

    println!(
        "Parsed {} in {:01}s, spend {:01}s reading files, in {:01}s of real time",
        count.load(Ordering::Relaxed),
        micros_as_sec(read_time),
        micros_as_sec(parse_time),
        micros_as_sec(total.as_micros() as usize)
    );

    Ok(())
}

fn micros_as_sec(micros: usize) -> f32 {
    micros as f32 / 1_000_000.0
}
