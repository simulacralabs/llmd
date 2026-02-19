//! `llmd issue` — issue tracker subcommands.

mod init;
mod list;
mod mentions;
mod new;
mod ready;
mod show;
mod tree;
mod update;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct IssueArgs {
    #[command(subcommand)]
    pub command: IssueCommand,
}

#[derive(Subcommand)]
pub enum IssueCommand {
    /// Create .llmd/issues/ and config.json
    Init(init::InitArgs),
    /// Create a new issue
    New(new::NewArgs),
    /// List issues with optional filters
    List(list::ListArgs),
    /// Show full issue content
    Show(show::ShowArgs),
    /// Update an issue
    Update(update::UpdateArgs),
    /// List open issues with no unresolved dependencies
    Ready(ready::ReadyArgs),
    /// Print epic hierarchy
    Tree(tree::TreeArgs),
    /// List issues with @mentions
    Mentions(mentions::MentionsArgs),
}

pub fn run(args: IssueArgs) -> Result<()> {
    match args.command {
        IssueCommand::Init(a) => init::run(a),
        IssueCommand::New(a) => new::run(a),
        IssueCommand::List(a) => list::run(a),
        IssueCommand::Show(a) => show::run(a),
        IssueCommand::Update(a) => update::run(a),
        IssueCommand::Ready(a) => ready::run(a),
        IssueCommand::Tree(a) => tree::run(a),
        IssueCommand::Mentions(a) => mentions::run(a),
    }
}
