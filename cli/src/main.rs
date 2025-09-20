#![allow(clippy::needless_return)]

mod dependencies;
mod macros;
mod tasks;

use std::io;

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{
    generate,
    shells::{Bash, Fish, Zsh},
};
use dependencies::check_dependencies;
use tasks::*;
use tracing::error;
use tracing_subscriber::{fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt};

/// Webapp Template CLI
#[derive(Parser)]
struct Args {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize the project
    Init,

    /// Run the project in development mode
    Dev,
    /// Run the project in production mode
    Prod,
    /// Run the project in test mode
    Test,
    /// Run the project in benchmark mode
    Bench,
    /// Check if the project compiles and passes linters
    Check,
    /// Bundle the project
    Bundle,

    /// Format the codebase
    Format,
    /// Run diesel (DB management).
    #[command(trailing_var_arg = true)]
    Diesel {
        #[arg(allow_hyphen_values = true)]
        query: Vec<String>,
    },
    /// Export API bindings to the client.
    ExportBindings,
    /// Generate pwa artifacts.
    GeneratePwaAssets,

    /// Open the server documentation.
    ServerDocs,
    /// Show some stats about the project.
    Stats,

    /// Generate shell completions
    Completions {
        #[arg(value_enum)]
        shell: Shell,
    },
}

/// Supported shells.
#[derive(Clone, clap::ValueEnum)]
enum Shell {
    Bash,
    Zsh,
    Fish,
}

fn main() {
    let args = Args::parse();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .without_time()
                .with_span_events(FmtSpan::ENTER | FmtSpan::CLOSE)
                .with_writer(std::io::stdout),
        )
        .init();

    // make sure dependencies are installed
    if let Err(e) = check_dependencies() {
        error!("{}", e);
        std::process::exit(1);
    }

    // execute command
    let result = match args.command {
        Commands::Init => init(),

        Commands::Dev => dev(),
        Commands::Prod => prod(),
        Commands::Test => test(),
        Commands::Bench => bench(),
        Commands::Check => check(),
        Commands::Bundle => bundle(),

        Commands::Format => format(),
        Commands::Diesel { query } => diesel(query.join(" ")),
        Commands::ExportBindings => export_bindings(),
        Commands::GeneratePwaAssets => generate_pwa_assets(),

        Commands::ServerDocs => server_docs(),
        Commands::Stats => stats(),

        Commands::Completions { shell } => {
            let mut cmd = Args::command();
            let bin_name = cmd.get_name().to_string();
            match shell {
                Shell::Bash => generate(Bash, &mut cmd, bin_name, &mut io::stdout()),
                Shell::Zsh => generate(Zsh, &mut cmd, bin_name, &mut io::stdout()),
                Shell::Fish => generate(Fish, &mut cmd, bin_name, &mut io::stdout()),
            };
            return;
        }
    };

    // check if command executed successfully
    if let Err(e) = result {
        error!("{}", e);
        std::process::exit(1);
    }
}
