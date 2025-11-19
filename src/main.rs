mod cli;
mod commands;
mod models;
mod network;
mod resolver;
mod utils;

use clap::{Parser, Subcommand};
use std::process;

#[derive(Parser)]
#[command(name = "pip")]
#[command(about = "The Python package installer", long_about = None)]
#[command(version = "0.1.0")]
struct Cli {
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
}

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Install {
            packages,
            requirements,
            target,
        } => {
            commands::install::handle_install(packages, requirements, target).await
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
                // Update specific packages (not implemented yet)
                eprintln!("Updating specific packages not yet implemented");
                Ok(1)
            }
        }
    };

    match result {
        Ok(code) => process::exit(code),
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}
