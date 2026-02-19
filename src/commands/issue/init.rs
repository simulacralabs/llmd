//! `llmd issue init`

use anyhow::{Context, Result};
use clap::Parser;
use std::fs;

use crate::issues::{load_config, save_config};
use crate::llmd_dir;

#[derive(Parser)]
pub struct InitArgs {}

pub fn run(_args: InitArgs) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let llmd = llmd_dir::find(&cwd)?;
    let issues_dir = llmd_dir::issues_path(&llmd);

    if issues_dir.exists() {
        let config = load_config(&issues_dir)?;
        eprintln!(
            ".llmd/issues/ already exists (next_id: {}).",
            config.next_id
        );
        return Ok(());
    }

    fs::create_dir_all(&issues_dir).context("Failed to create .llmd/issues/")?;
    let config = crate::issues::Config::default();
    save_config(&issues_dir, &config)?;

    eprintln!("Initialised .llmd/issues/ at {}", issues_dir.display());
    Ok(())
}
