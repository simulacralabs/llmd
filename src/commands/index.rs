//! `llmd index` — print the section index for use with `llmd compose`.
//!
//! Outputs a numbered list of all H2/H3 sections from topic files in .llmd/.
//! The caller (human or agent) uses this to choose section numbers to pass to
//! `llmd compose --sections 1,2,3`.

use anyhow::Result;
use clap::Parser;

use crate::commands::compose;

#[derive(Parser)]
pub struct IndexArgs {}

pub fn run(_args: IndexArgs) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let llmd = crate::llmd_dir::find(&cwd)?;

    compose::print_section_index(&llmd)
}
