use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    /// Path to the directory or file to delete
    pub path: String,

    /// Color of the progress bar
    #[clap(short, long, default_value = "green")]
    pub color: String,

    /// Show verbose output
    #[clap(short, long)]
    pub verbose: bool,

    /// Show current file being deleted
    #[clap(short = 's', long)]
    pub show_current: bool,
}
