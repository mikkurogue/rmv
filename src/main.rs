use indicatif::{ProgressBar, ProgressStyle};
use rand::seq::IndexedRandom;
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Available progress bar colors

const COLORS: &[&str] = &[
    "red",
    "bright-red",
    "dark-red",
    "green",
    "bright-green",
    "dark-green",
    "yellow",
    "bright-yellow",
    "gold",
    "blue",
    "bright-blue",
    "dark-blue",
    "cyan",
    "bright-cyan",
    "magenta",
    "bright-magenta",
    "purple",
    "orange",
    "bright-orange",
    "pink",
    "hot-pink",
    "white",
    "gray",
    "silver",
];

fn count_files(path: &Path) -> usize {
    WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .count()
}

fn delete_with_progress(path: &Path, pb: &ProgressBar, is_verbose: bool) -> std::io::Result<()> {
    for entry in WalkDir::new(path).into_iter().filter_map(Result::ok) {
        let current_path = entry.path();
        if current_path.is_file() {
            if is_verbose {
                verbose_log(&format!("Removing file: {}", current_path.display()));
            }
            fs::remove_file(current_path)?;
            pb.inc(1);
        } else if current_path.is_dir() {
            if is_verbose {
                verbose_log(&format!("Removing directory: {}", current_path.display()));
            }
            fs::remove_dir(current_path).ok();
            pb.inc(1);
        } else if current_path.is_symlink() {
            if is_verbose {
                verbose_log(&format!("Removing symlink: {}", current_path.display()));
            }
            fs::remove_file(current_path)?;
            pb.inc(1);
        }
    }

    Ok(())
}

fn verbose_log(message: &str) {
    std::io::stdout().flush().unwrap();
    print!("{}\n", message);
}

fn main() {
    let path = env::args().nth(1).expect("Usage: rmv <path>");
    let path = PathBuf::from(path);

    let is_verbose = env::args().any(|arg| arg == "-v" || arg == "--verbose");

    if !path.exists() {
        eprintln!("No such file or directory: {}", path.display());
        std::process::exit(0);
    }

    let total_files = count_files(&path);
    let pb = ProgressBar::new(total_files as u64);

    let mut rng = rand::rng();
    let color = COLORS.choose(&mut rng).unwrap_or(&"blue");

    pb.set_style(
        ProgressStyle::default_bar()
            .template(&format!(
                "{{msg}} [{{bar:40.{}}}] {{pos}}/{{len}} ({{eta}})",
                color
            ))
            .unwrap()
            .progress_chars("▰▰▱▱ "),
    );
    pb.set_message("Removing...");

    if let Err(e) = delete_with_progress(&path, &pb, is_verbose) {
        eprintln!("Error: {}: {}", path.display(), e);
        std::process::exit(0);
    }

    pb.finish_with_message("Removal complete");
}
