mod cli;
mod commands;

// Re-export core modules so they are available as crate::... in submodules
pub use pip_rs_core::errors;
pub use pip_rs_core::models;
pub use pip_rs_core::network;
pub use pip_rs_core::resolver;
pub use pip_rs_core::installer;
pub use pip_rs_core::utils;
pub use pip_rs_core::cache;
pub use pip_rs_core::venv;
pub use pip_rs_core::config;

use clap::{Parser, Subcommand};
use std::process;

#[derive(Parser)]
#[command(name = "pip")]
#[command(about = "The fastest pip-compatible package installer", long_about = None)]
#[command(version = "1.0.0")]
struct Cli {
    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Quiet mode - minimal output
    #[arg(short, long, global = true)]
    quiet: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Install packages
    Install {
        /// Package names or requirements to install
        packages: Vec<String>,

        /// Requirements file
        #[arg(short, long)]
        requirements: Option<String>,

        /// Constraints file
        #[arg(short = 'c', long)]
        constraints: Option<String>,

        /// Trusted host (can be specified multiple times)
        #[arg(long)]
        trusted_host: Vec<String>,

        /// Target directory for installation
        #[arg(short, long)]
        target: Option<String>,
    },
    /// Uninstall packages
    Uninstall {
        /// Package names to uninstall
        packages: Vec<String>,

        /// Assume yes to all prompts
        #[arg(short, long)]
        yes: bool,
    },
    /// List installed packages
    List {
        /// Show outdated packages
        #[arg(long)]
        outdated: bool,
    },
    /// Show package information
    Show {
        /// Package name
        package: String,
    },
    /// Search for packages
    Search {
        /// Search query
        query: String,
    },
    /// Check for outdated packages
    Check {
        /// Package name
        #[arg(short, long)]
        package: Option<String>,
    },
    /// Update packages (check for outdated and show upgrade instructions)
    #[command(alias = "upgrade")]
    Update {
        /// Package names to update (if empty, update all outdated)
        packages: Vec<String>,
    },
    /// Generate requirements.txt from installed packages
    Freeze {
        /// Output file (if not specified, prints to stdout)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Download packages without installing
    Download {
        /// Package names or requirements to download
        packages: Vec<String>,

        /// Requirements file
        #[arg(short, long)]
        requirements: Option<String>,

        /// Destination directory for downloads
        #[arg(short, long)]
        destination: Option<String>,
    },
    /// Generate lock file for reproducible installs
    Lock {
        /// Requirements file
        #[arg(short, long)]
        requirements: Option<String>,

        /// Output lock file path
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Display debug information
    Debug,
    /// Generate shell completion
    Completion {
        /// Shell type (bash, zsh, fish, powershell)
        shell: String,
    },
}

/// Initialize logging with appropriate verbosity level
fn init_logging(verbose: bool, quiet: bool) {
    use tracing_subscriber::filter::LevelFilter;
    
    // Set PIP_QUIET env var for progress bar checks
    if quiet {
        // SAFETY: We're setting a single env var at startup before any threads spawn
        unsafe { std::env::set_var("PIP_QUIET", "1") };
    }
    
    let level = if quiet {
        LevelFilter::ERROR
    } else if verbose {
        LevelFilter::DEBUG
    } else {
        // Check RUST_LOG environment variable
        match std::env::var("RUST_LOG") {
            Ok(val) => match val.to_lowercase().as_str() {
                "trace" => LevelFilter::TRACE,
                "debug" => LevelFilter::DEBUG,
                "info" => LevelFilter::INFO,
                "warn" => LevelFilter::WARN,
                "error" => LevelFilter::ERROR,
                _ => LevelFilter::INFO,
            },
            Err(_) => LevelFilter::INFO,
        }
    };

    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_target(verbose)
        .with_thread_ids(verbose)
        .init();
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Initialize logging based on verbose/quiet flags
    init_logging(cli.verbose, cli.quiet);

    let result = match cli.command {
        Commands::Install {
            packages,
            requirements,
            constraints,
            trusted_host,
            target,
        } => {
            commands::install::handle_install(packages, requirements, constraints, trusted_host, target).await
        }
        Commands::Uninstall { packages, yes } => {
            commands::uninstall::handle_uninstall(packages, yes).await
        }
        Commands::List { outdated } => commands::list::handle_list(outdated).await,
        Commands::Show { package } => commands::show::handle_show(&package).await,
        Commands::Search { query } => commands::search::handle_search(&query).await,
        Commands::Check { package } => commands::check::handle_check(package).await,
        Commands::Update { packages } => {
            if packages.is_empty() {
                // Update all outdated packages
                commands::upgrade::handle_upgrade_all().await
            } else {
                // Update specific packages
                commands::upgrade::handle_upgrade_packages(packages).await
            }
        }
        Commands::Freeze { output } => commands::freeze::handle_freeze(output).await,
        Commands::Download {
            packages,
            requirements,
            destination,
        } => commands::download::handle_download(packages, requirements, destination).await,
        Commands::Lock {
            requirements,
            output,
        } => commands::lock::handle_lock(requirements, output).await,
        Commands::Debug => commands::debug::handle_debug().await,
        Commands::Completion { shell } => commands::completion::handle_completion(shell).await,
    };

use errors::format_error_with_suggestion;

// ... (rest of the file)

    match result {
        Ok(code) => process::exit(code),
        Err(e) => {
            eprintln!("{}", format_error_with_suggestion(&e.to_string()));
            eprintln!("Run with RUST_LOG=debug for more details");
            process::exit(1);
        }
    }
}
