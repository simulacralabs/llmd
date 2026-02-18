//! `llmd search` — full-text search across all .llmd/ files.
//!
//! Returns matching lines with file path, line number, and configurable context.

use anyhow::{Context, Result};
use clap::Parser;
use std::collections::BTreeSet;
use std::fs;

use crate::llmd_dir;

#[derive(Parser)]
pub struct SearchArgs {
    /// Text or regex pattern to search for
    pub query: String,

    /// Number of context lines to show before and after each match (default: 2)
    #[arg(long, short, default_value = "2")]
    pub context: usize,

    /// Search only within this subdirectory of .llmd/ (e.g. "imported")
    #[arg(long, short = 'd')]
    pub dir: Option<String>,
}

pub fn run(args: SearchArgs) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let llmd = llmd_dir::find(&cwd)?;

    let search_root = match &args.dir {
        Some(sub) => llmd.join(sub),
        None => llmd.clone(),
    };

    if !search_root.exists() {
        anyhow::bail!("Search directory does not exist: {}", search_root.display());
    }

    let re = regex_lite::Regex::new(&args.query)
        .with_context(|| format!("Invalid search pattern: {}", args.query))?;

    let files = llmd_dir::list_all_files(&search_root);
    if files.is_empty() {
        eprintln!("No .md files found in {}", search_root.display());
        return Ok(());
    }

    let mut total_matches = 0usize;

    for file_path in &files {
        let rel = file_path.strip_prefix(&llmd).unwrap_or(file_path);
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let lines: Vec<&str> = content.lines().collect();
        let mut context_indices = BTreeSet::new();
        let mut match_indices = BTreeSet::new();

        for (i, line) in lines.iter().enumerate() {
            if re.is_match(line) {
                match_indices.insert(i);
                total_matches += 1;
                let start = i.saturating_sub(args.context);
                let end = (i + args.context).min(lines.len().saturating_sub(1));
                for j in start..=end {
                    context_indices.insert(j);
                }
            }
        }

        if !context_indices.is_empty() {
            println!("\n{}:", rel.display());
            let mut prev: Option<usize> = None;
            for &idx in &context_indices {
                if let Some(p) = prev
                    && idx > p + 1
                {
                    println!("  ...");
                }
                let marker = if match_indices.contains(&idx) {
                    ">"
                } else {
                    " "
                };
                println!("  {marker} {:4}: {}", idx + 1, lines[idx]);
                prev = Some(idx);
            }
        }
    }

    if total_matches == 0 {
        eprintln!("No matches found for \"{}\"", args.query);
    } else {
        eprintln!("\n{total_matches} match(es) found.");
    }

    Ok(())
}
