mod commands;
mod discovery;
mod issues;
mod llmd_dir;
mod markdown;

use anyhow::Result;
use clap::{Parser, Subcommand};

use commands::{
    bootstrap::BootstrapArgs, build::BuildArgs, compose::ComposeArgs, index::IndexArgs,
    init::InitArgs, issue::IssueArgs, read::ReadArgs, search::SearchArgs, serve::ServeArgs,
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
    /// Print a prompt that instructs an LLM to populate .llmd/ from the codebase
    Bootstrap(BootstrapArgs),
    /// Read a file or section from .llmd/
    Read(ReadArgs),
    /// Print the section index for use with `llmd compose --sections`
    Index(IndexArgs),
    /// Compose a task-context document from .llmd/ content
    Compose(ComposeArgs),
    /// Search for text across all .llmd/ files
    Search(SearchArgs),
    /// Generate an mdbook from .llmd/ and serve it locally
    Serve(ServeArgs),
    /// Build a static mdbook site from .llmd/
    Build(BuildArgs),
    /// Issue tracker: init, new, list, show, update, ready, tree, mentions
    Issue(IssueArgs),
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Init(args) => commands::init::run(args),
        Command::Bootstrap(args) => commands::bootstrap::run(args),
        Command::Read(args) => commands::read::run(args),
        Command::Index(args) => commands::index::run(args),
        Command::Compose(args) => commands::compose::run(args),
        Command::Search(args) => commands::search::run(args),
        Command::Serve(args) => commands::serve::run(args),
        Command::Build(args) => commands::build::run(args),
        Command::Issue(args) => commands::issue::run(args),
    }
}
