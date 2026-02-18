//! `llmd serve` — build an mdbook from .llmd/ and open it in a browser.
//!
//! Generates a temporary mdbook project (book.toml + src/) from the .llmd/
//! directory, then calls `mdbook serve`. Requires mdbook to be installed:
//! `cargo install mdbook`

use anyhow::{Context, Result, bail};
use clap::Parser;
use std::process::Command;

use crate::llmd_dir;

#[derive(Parser)]
pub struct ServeArgs {
    /// Port to serve on
    #[arg(long, short, default_value = "3000")]
    pub port: u16,

    /// Do not open the browser automatically after starting the server
    #[arg(long)]
    pub no_open: bool,
}

pub fn run(args: ServeArgs) -> Result<()> {
    ensure_mdbook()?;

    let cwd = std::env::current_dir()?;
    let llmd = llmd_dir::find(&cwd)?;
    let book_dir = super::build::generate_mdbook(&llmd)?;

    eprintln!(
        "Serving .llmd/ at http://localhost:{} — press Ctrl+C to stop",
        args.port
    );

    let mut cmd = Command::new("mdbook");
    cmd.arg("serve")
        .arg("--port")
        .arg(args.port.to_string())
        .arg(&book_dir);

    if !args.no_open {
        cmd.arg("--open");
    }

    let status = cmd.status().context("Failed to run `mdbook serve`")?;
    if !status.success() {
        bail!("`mdbook serve` exited with status {status}");
    }

    Ok(())
}

fn ensure_mdbook() -> Result<()> {
    if Command::new("mdbook").arg("--version").output().is_err() {
        bail!("`mdbook` is not installed. Install it with:\n  cargo install mdbook");
    }
    Ok(())
}
