pub mod completions;
pub mod config_cmd;
pub mod format;
pub mod run;
pub mod status;
pub mod undo;
pub mod watch;

use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "tidy",
    author,
    version,
    about = "A blazing-fast, zero-dependency, local-first file organizer and watcher for Linux and macOS.",
    long_about = None
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Verbose output logging
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Path to custom configuration file
    #[arg(short, long, global = true)]
    pub config: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Execute a single-pass scan and organize cycle
    Run(RunArgs),

    /// Start the long-running resident filesystem watcher daemon
    Watch(WatchArgs),

    /// Roll back the most recent operation batch using the local ledger
    Undo(UndoArgs),

    /// Report watched directories, rule summaries, and recent activity
    Status,

    /// Manage user-level OS background daemons (systemd on Linux, launchd on macOS)
    Service(ServiceArgs),

    /// Scaffold a default configuration file (config.toml)
    Init(InitArgs),

    /// Generate shell completions for Bash, Zsh, Fish, PowerShell, or Elvish
    Completions(CompletionsArgs),
}

#[derive(Args, Debug)]
pub struct InitArgs {
    /// Overwrite configuration file if it already exists
    #[arg(long, short)]
    pub force: bool,

    /// Custom target path to write the configuration file to
    #[arg(long, short)]
    pub path: Option<PathBuf>,
}

#[derive(Args, Debug)]
pub struct CompletionsArgs {
    /// Target shell to generate completions for
    #[arg(value_enum)]
    pub shell: crate::cli::completions::SupportedShell,
}

#[derive(Args, Debug)]
pub struct RunArgs {
    /// Target directory to organize (defaults to current directory)
    #[arg(short, long)]
    pub path: Option<PathBuf>,

    /// Perform a dry-run without moving any files or writing to the database
    #[arg(long)]
    pub dry_run: bool,

    /// Scan target directory recursively
    #[arg(short, long)]
    pub recursive: bool,
}

#[derive(Args, Debug)]
pub struct WatchArgs {
    /// Directory to watch (defaults to user Downloads or current directory)
    #[arg(short, long)]
    pub path: Option<PathBuf>,

    /// Watch target directory recursively
    #[arg(short, long)]
    pub recursive: bool,

    /// Debounce duration in milliseconds (overrides config)
    #[arg(long)]
    pub debounce: Option<u64>,

    /// Perform an initial single-pass scan to organize pre-existing files on startup
    #[arg(long)]
    pub initial_scan: bool,
}

#[derive(Args, Debug)]
pub struct UndoArgs {
    /// Specific Run UUID to roll back (defaults to the most recent run)
    #[arg(long)]
    pub run_id: Option<String>,

    /// Number of recent runs to roll back sequentially
    #[arg(long, short = 'n')]
    pub last: Option<usize>,

    /// Roll back all recorded runs in the ledger
    #[arg(long)]
    pub all: bool,

    /// Preview the undo operations without modifying disk or database
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Args, Debug)]
pub struct ServiceArgs {
    /// Service action to execute
    #[arg(value_enum)]
    pub action: ServiceAction,

    /// Optional target directory to watch in the background service (used on enable)
    #[arg(short, long)]
    pub path: Option<PathBuf>,

    /// Watch target directory recursively in the background service (used on enable)
    #[arg(short, long)]
    pub recursive: bool,

    /// Number of journal log lines to show (for logs action, default: 50)
    #[arg(short = 'n', long, default_value = "50")]
    pub lines: usize,

    /// Follow real-time log output (for logs action)
    #[arg(short, long)]
    pub follow: bool,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServiceAction {
    /// Enable and start the background service
    Enable,
    /// Stop and disable the background service
    Disable,
    /// Query current status of the background service
    Status,
    /// Restart the background service
    Restart,
    /// View background service journal logs
    Logs,
}
