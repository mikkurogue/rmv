use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use rand::Rng;
use std::error::Error;
use std::fs::{self};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;
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

fn create_progress_bar(loader_type: &str, bar_color: &str, total: u64) -> ProgressBar {
    let pb = match loader_type {
        "spinner" => ProgressBar::new_spinner(),
        _ => ProgressBar::new(total),
    };

    if loader_type == "spinner" {
        pb.enable_steady_tick(Duration::from_millis(200));
    }

    let style = match loader_type {
        "spinner" => ProgressStyle::default_spinner()
            .template("{spinner:.bold} {msg} ({elapsed})")
            .unwrap(),
        _ => ProgressStyle::default_bar()
            .template(&format!(
                "{{msg}} [{{bar:40.{}}}] {{pos}}/{{len}} ({{eta}})",
                bar_color
            ))
            .unwrap()
            .progress_chars("▰▰▱▱ "),
    };

    pb.set_style(style);
    pb
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
            let type_str = match file_type {
                t if t.is_file() => "file",
                t if t.is_dir() => "directory",
                _ => "symlink",
            };

            let _ = verbose_log(&format!(
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

fn verbose_log(message: &str) -> Result<(), Box<dyn Error>> {
    std::io::stdout().flush()?;
    println!("{}", message);

    Ok(())
}

struct ParsedConfig {
    verbose: bool,
    show_current: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = cli::Args::parse();
    let path = PathBuf::from(args.path);

    if !path.exists() {
        eprintln!("No such file or directory: {}", path.display());
        std::process::exit(0);
    }

    let total_files = count_files(&path);

    let color: &str = if !args.color.is_empty() {
        &args.color
    } else {
        COLORS[rand::rng().random_range(0..COLORS.len())]
    };

    let pb = create_progress_bar(&args.loader, color, total_files as u64);
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

    if !args.flush {
        pb.finish_with_message("Delete complete");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use tempfile::tempdir;

    #[test]
    fn test_create_progress_bar_with_bar_loader() {
        let pb = create_progress_bar("bar", "green", 100);
        assert_eq!(pb.length(), Some(100));
    }

    #[test]
    fn test_create_progress_bar_with_spinner_loader() {
        let pb = create_progress_bar("spinner", "green", 100);
        // Spinner doesn't have a fixed length
        assert_eq!(pb.length(), None);
    }

    #[test]
    fn test_create_progress_bar_with_unknown_loader_defaults_to_bar() {
        let pb = create_progress_bar("unknown", "blue", 50);
        assert_eq!(pb.length(), Some(50));
    }

    #[test]
    fn test_count_files_single_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        File::create(&file_path).unwrap();

        // count_files counts both the directory and the file
        assert_eq!(count_files(dir.path()), 2);
    }

    #[test]
    fn test_count_files_nested_structure() {
        let dir = tempdir().unwrap();
        let subdir = dir.path().join("subdir");
        fs::create_dir(&subdir).unwrap();
        File::create(subdir.join("file1.txt")).unwrap();
        File::create(subdir.join("file2.txt")).unwrap();
        File::create(dir.path().join("root.txt")).unwrap();

        // dir + subdir + 3 files = 5
        assert_eq!(count_files(dir.path()), 5);
    }

    #[test]
    fn test_count_files_empty_directory() {
        let dir = tempdir().unwrap();
        // Just the directory itself
        assert_eq!(count_files(dir.path()), 1);
    }
}
