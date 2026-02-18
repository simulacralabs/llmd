//! `llmd read` — read a file or section from .llmd/.
//!
//! Supports reading the full file, a specific heading section, a line range,
//! or a grep-filtered view. Optionally prints a token count estimate first.

use anyhow::{bail, Context, Result};
use clap::Parser;
use std::fs;
use std::path::PathBuf;

use crate::{llmd_dir, markdown};

#[derive(Parser)]
pub struct ReadArgs {
    /// File or topic name to read. Use "catme" to read catme.md.
    /// Relative paths are resolved inside .llmd/; omit the .md extension.
    pub file: String,

    /// Extract only the section under this heading (case-insensitive substring match)
    #[arg(long, short)]
    pub section: Option<String>,

    /// Filter output to lines matching this regex pattern (with 2 lines context)
    #[arg(long, short)]
    pub grep: Option<String>,

    /// Show only lines in this range, e.g. "10:50" (1-indexed, inclusive)
    #[arg(long, short, value_name = "START:END")]
    pub lines: Option<String>,

    /// Print an estimated token count before the content
    #[arg(long, short = 'T')]
    pub tokens: bool,
}

pub fn run(args: ReadArgs) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let llmd = llmd_dir::find(&cwd)?;

    let path = resolve_file(&llmd, &args.file)?;
    let content =
        fs::read_to_string(&path).with_context(|| format!("Cannot read {}", path.display()))?;

    let mut output = content.clone();

    if let Some(section) = &args.section {
        output = markdown::extract_section(&output, section)
            .with_context(|| format!("Section \"{section}\" not found in {}", path.display()))?;
    }

    if let Some(range) = &args.lines {
        let (start, end) = parse_line_range(range)?;
        output = markdown::window(&output, start, end);
    }

    if let Some(pattern) = &args.grep {
        output = grep_lines(&output, pattern)?;
    }

    if args.tokens {
        let count = markdown::estimate_tokens(&output);
        eprintln!("~{count} tokens");
    }

    print!("{output}");
    if !output.ends_with('\n') {
        println!();
    }

    Ok(())
}

/// Resolves a user-supplied file name to an absolute path inside .llmd/.
fn resolve_file(llmd: &std::path::Path, name: &str) -> Result<PathBuf> {
    let name = if name == "catme" { "catme.md" } else { name };

    let direct = llmd.join(name);
    if direct.exists() {
        return Ok(direct);
    }

    let with_ext = llmd.join(format!("{name}.md"));
    if with_ext.exists() {
        return Ok(with_ext);
    }

    let in_imported = llmd.join("imported").join(name);
    if in_imported.exists() {
        return Ok(in_imported);
    }

    bail!(
        "File \"{name}\" not found in .llmd/. \
         Run `llmd search {name}` to find it, or `llmd read catme` to browse available docs."
    )
}

/// Parses a "start:end" line range string.
fn parse_line_range(range: &str) -> Result<(usize, usize)> {
    let parts: Vec<&str> = range.splitn(2, ':').collect();
    if parts.len() != 2 {
        bail!("Line range must be in the form START:END, e.g. --lines 10:50");
    }
    let start: usize = parts[0].parse().context("Invalid start line number")?;
    let end: usize = parts[1].parse().context("Invalid end line number")?;
    if start == 0 {
        bail!("Line numbers are 1-indexed; start must be >= 1");
    }
    if start > end {
        bail!("Start line must be <= end line");
    }
    Ok((start, end))
}

/// Filters `content` to lines matching `pattern`, with 2 lines of context each side.
fn grep_lines(content: &str, pattern: &str) -> Result<String> {
    let re = regex_lite::Regex::new(pattern)
        .with_context(|| format!("Invalid regex pattern: {pattern}"))?;

    let lines: Vec<&str> = content.lines().collect();
    let mut matched_indices = std::collections::BTreeSet::new();

    for (i, line) in lines.iter().enumerate() {
        if re.is_match(line) {
            let start = i.saturating_sub(2);
            let end = (i + 2).min(lines.len().saturating_sub(1));
            for j in start..=end {
                matched_indices.insert(j);
            }
        }
    }

    if matched_indices.is_empty() {
        return Ok(String::new());
    }

    let mut result = String::new();
    let mut prev: Option<usize> = None;
    for &idx in &matched_indices {
        if let Some(p) = prev {
            if idx > p + 1 {
                result.push_str("...\n");
            }
        }
        result.push_str(lines[idx]);
        result.push('\n');
        prev = Some(idx);
    }

    Ok(result)
}
