use indicatif::{ProgressBar, ProgressStyle};
use rand::seq::IndexedRandom;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Available progress bar colors
const COLORS: &[&str] = &["red", "green", "yellow", "blue", "magenta", "cyan"];

fn count_files(path: &Path) -> usize {
    WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .count()
}

fn delete_with_progress(path: &Path, pb: &ProgressBar) -> std::io::Result<()> {
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();
            delete_with_progress(&entry_path, pb)?;
        }
        fs::remove_dir(path)?;
    } else {
        fs::remove_file(path)?;
    }

    pb.inc(1);
    Ok(())
}

fn main() {
    let path = env::args().nth(1).expect("Usage: rmv <path>");
    let path = PathBuf::from(path);

    if !path.exists() {
        eprintln!("Error: Path does not exist: {}", path.display());
        std::process::exit(0);
    }

    let total_files = count_files(&path);
    let pb = ProgressBar::new(total_files as u64);

    let mut rng = rand::rng();
    let color = COLORS.choose(&mut rng).unwrap_or(&"blue");

    pb.set_style(
        ProgressStyle::default_bar()
            .template(&format!(
                "🧹 {{msg}} [{{bar:40.{}/blue}}] {{pos}}/{{len}} ({{eta}}) 🚀",
                color
            ))
            .unwrap()
            .progress_chars("█▓▒░ "),
    );
    pb.set_message("Removing...");

    if let Err(e) = delete_with_progress(&path, &pb) {
        eprintln!("Error deleting: {}: {}", path.display(), e);
        std::process::exit(0);
    }

    pb.finish_with_message("Delete complete");
}
