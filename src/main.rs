mod commands;
mod discovery;
mod llmd_dir;
mod markdown;

use anyhow::Result;
use clap::{Parser, Subcommand};

use commands::{
    build::BuildArgs, compose::ComposeArgs, init::InitArgs, read::ReadArgs, search::SearchArgs,
    serve::ServeArgs,
};

#[derive(Parser)]
#[command(
    name = "llmd",
    about = "Context management for agentic development",
    long_about = "llmd manages a .llmd/ directory — a persistent, machine-readable knowledge base \
                  that AI agents can read, search, and compose into task-specific context documents.",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Initialise a .llmd/ directory, discovering existing agent config files
    Init(InitArgs),
    /// Read a file or section from .llmd/
    Read(ReadArgs),
    /// Compose a task-context document from .llmd/ content
    Compose(ComposeArgs),
    /// Search for text across all .llmd/ files
    Search(SearchArgs),
    /// Generate an mdbook from .llmd/ and serve it locally
    Serve(ServeArgs),
    /// Build a static mdbook site from .llmd/
    Build(BuildArgs),
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Init(args) => commands::init::run(args),
        Command::Read(args) => commands::read::run(args),
        Command::Compose(args) => commands::compose::run(args),
        Command::Search(args) => commands::search::run(args),
        Command::Serve(args) => commands::serve::run(args),
        Command::Build(args) => commands::build::run(args),
    }
}
