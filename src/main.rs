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

fn delete_with_progress(path: &Path, pb: &ProgressBar, config: Config) -> std::io::Result<()> {
    let is_verbose = config.verbose;
    let show_current = config.show_current;

    for entry in WalkDir::new(path)
        .contents_first(true)
        .into_iter()
        .filter_map(Result::ok)
    {
        let current_path = entry.path();

        let file_type = entry.file_type();

        if is_verbose {
            let type_str = if file_type.is_file() {
                "file"
            } else if file_type.is_dir() {
                "directory"
            } else {
                "symlink"
            };

            verbose_log(&format!(
                "Processing {}: {}",
                type_str,
                current_path.display()
            ));
        }

        if file_type.is_dir() {
            fs::remove_dir(current_path)?;
        } else {
            fs::remove_file(current_path)?;
        }
        if show_current {
            pb.set_message(format!("{}", current_path.display()));
        }

        pb.inc(1);
    }

    Ok(())
}

fn verbose_log(message: &str) {
    std::io::stdout().flush().unwrap();
    print!("{}\n", message);
}

struct Config {
    verbose: bool,
    show_current: bool,
}

fn main() {
    let path = env::args().nth(1).expect("Usage: rmv <path>");
    let path = PathBuf::from(path);

    let is_verbose = env::args().any(|arg| arg == "-v" || arg == "--verbose");
    let show_current = env::args().any(|arg| arg == "-c" || arg == "--current");

    let config = Config {
        verbose: is_verbose,
        show_current,
    };

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

    if let Err(e) = delete_with_progress(&path, &pb, config) {
        eprintln!("Error: {}: {}", path.display(), e);
        std::process::exit(0);
    }
}
