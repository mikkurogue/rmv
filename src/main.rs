use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use rand::seq::IndexedRandom;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

mod cli;

/// Available progress bar colors
const COLORS: &[&str] = &["red", "green", "yellow", "blue", "cyan", "magenta", "white"];

fn count_files(path: &Path) -> usize {
    WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .count()
}

fn delete_with_progress(
    path: &Path,
    pb: &ProgressBar,
    config: ParsedConfig,
) -> std::io::Result<()> {
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

struct ParsedConfig {
    verbose: bool,
    show_current: bool,
}

fn main() {
    let args = cli::Args::parse();
    let path = PathBuf::from(args.path);

    if !path.exists() {
        eprintln!("No such file or directory: {}", path.display());
        std::process::exit(0);
    }

    let total_files = count_files(&path);
    let pb = ProgressBar::new(total_files as u64);

    let color = if !args.color.is_empty() {
        &args.color.as_str()
    } else {
        let mut rng = rand::rng();
        COLORS.choose(&mut rng).unwrap_or(&"blue")
    };

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

    if let Err(e) = delete_with_progress(
        &path,
        &pb,
        ParsedConfig {
            verbose: args.verbose,
            show_current: args.show_current,
        },
    ) {
        eprintln!("Error: {}: {}", path.display(), e);
        std::process::exit(0);
    }
}
