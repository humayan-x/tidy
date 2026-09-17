use clap::Parser;
use tidy::cli::{self, Cli, Commands};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use console::style;

fn main() {
    let cli = Cli::parse();

    // Initialize tracing subscriber
    let filter = if cli.verbose {
        EnvFilter::new("tidy=debug,warn")
    } else {
        EnvFilter::new("tidy=info,warn")
    };

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter)
        .init();

    let result = match &cli.command {
        Commands::Run(args) => cli::run::execute_run(args, cli.config.as_deref()),
        Commands::Watch(args) => cli::watch::execute_watch(args, cli.config.as_deref()),
        Commands::Undo(args) => cli::undo::execute_undo_cmd(args),
        Commands::Status => cli::status::execute_status_cmd(cli.config.as_deref()),
        Commands::Service(args) => tidy::service::execute_service(args, cli.config.as_deref()),
        Commands::Init(args) => cli::config_cmd::execute_init(args.path.as_deref(), args.force),
        Commands::Completions(args) => cli::completions::generate_completions(args.shell),
    };

    if let Err(err) = result {
        eprintln!();
        eprintln!("  {} {}", style("Error:").red().bold(), err);
        eprintln!();
        std::process::exit(1);
    }
}
