//! `llmd compose` — compose a task-context document from .llmd/ content.
//!
//! Two modes:
//!
//! **Keyword mode** (default):
//!   Takes a task description and keyword-matches it against .llmd/ headings.
//!   Produces a composed document automatically.
//!
//! **Interactive mode** (`--interactive`):
//!   Prints a numbered index of every available section across all topic files.
//!   Reads a comma- or newline-separated list of section numbers from stdin.
//!   Composes the document from the explicitly chosen sections only.
//!
//!   This is designed for agents: the index output is compact (one line per section),
//!   allowing the agent to make informed choices without loading all file content first.

use anyhow::{Context, Result};
use clap::Parser;
use std::collections::HashSet;
use std::fs;
use std::io::BufRead;
use std::path::PathBuf;

use crate::{llmd_dir, markdown};

#[derive(Parser)]
pub struct ComposeArgs {
    /// The task description. Used for keyword matching in default mode.
    /// Pass "-" to read from stdin. Ignored in --interactive mode.
    pub task: Option<String>,

    /// Explicitly include these topic files by name (no .md, comma-separated)
    #[arg(long, short = 'I', value_delimiter = ',')]
    pub include: Vec<String>,

    /// Read the task description from a file
    #[arg(long, short, value_name = "FILE")]
    pub from: Option<PathBuf>,

    /// Write the composed document to a file instead of stdout
    #[arg(long, short, value_name = "FILE")]
    pub output: Option<PathBuf>,

    /// Interactive mode: print available sections, read choices from stdin,
    /// compose from explicit selections rather than keyword matching
    #[arg(long, short = 'i')]
    pub interactive: bool,
}

pub fn run(args: ComposeArgs) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let llmd = llmd_dir::find(&cwd)?;

    let catme_path = llmd_dir::catme_path(&llmd);
    let catme =
        fs::read_to_string(&catme_path).context("Cannot read catme.md — run `llmd init` first")?;

    let catme_excerpt = extract_catme_excerpt(&catme);
    let all_files = llmd_dir::list_all_files(&llmd);

    // Build a flat index of every section across all topic files.
    // Excludes catme.md (always included in full as the project overview)
    // and imported/ files (raw config, not structured topic docs).
    let index = build_section_index(&llmd, &all_files);

    let chosen_sections: Vec<IndexedSection> = if args.interactive {
        run_interactive_mode(&index)?
    } else {
        let task = load_task(&args)?;
        let keywords = extract_keywords(&task);
        keyword_match_sections(&index, &keywords, &args.include)
    };

    let task = load_task(&args).unwrap_or_default();
    let doc = build_document(
        &task,
        &catme_excerpt,
        &args.include,
        &llmd,
        &chosen_sections,
    )?;

    match &args.output {
        Some(out_path) => {
            fs::write(out_path, &doc)
                .with_context(|| format!("Cannot write to {}", out_path.display()))?;
            eprintln!("Wrote context document to {}", out_path.display());
        }
        None => print!("{doc}"),
    }

    Ok(())
}

// --- Section index ---

/// A single entry in the section index: a heading from a topic file.
struct IndexedSection {
    /// Display label: "topic-file > Heading Text"
    label: String,
    /// The file this section lives in
    file: PathBuf,
    /// The heading text (used to extract the section)
    heading: String,
}

/// Builds a flat ordered list of all H2/H3 headings from all non-catme, non-imported files.
fn build_section_index(llmd: &std::path::Path, all_files: &[PathBuf]) -> Vec<IndexedSection> {
    let catme = llmd_dir::catme_path(llmd);
    let imported = llmd.join("imported");
    let mut index = Vec::new();

    for file_path in all_files {
        if *file_path == catme || file_path.starts_with(&imported) {
            continue;
        }
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let file_label = file_path
            .strip_prefix(llmd)
            .unwrap_or(file_path)
            .with_extension("")
            .display()
            .to_string();

        for (depth, heading) in markdown::list_headings(&content) {
            // Include H2 and H3 only — H1 is the file title (too broad),
            // H4+ are too granular for context selection.
            if depth == 2 || depth == 3 {
                index.push(IndexedSection {
                    label: format!("{file_label} > {heading}"),
                    file: file_path.clone(),
                    heading: heading.clone(),
                });
            }
        }
    }

    index
}

// --- Interactive mode ---

/// Prints the section index and reads the user/agent's choices from stdin.
///
/// Output format (one line per section, to stdout):
///   [1] catme > Project Summary
///   [2] api-standards > Endpoints
///   ...
///
/// Input: a comma- or newline-separated list of numbers, terminated by a blank line or EOF.
/// Example: "1,3,7" or "1\n3\n7\n\n"
fn run_interactive_mode(index: &[IndexedSection]) -> Result<Vec<IndexedSection>> {
    if index.is_empty() {
        eprintln!("No sections found in .llmd/. Add topic files first.");
        return Ok(Vec::new());
    }

    // Print the index to stdout so the agent can read it
    println!(
        "Available sections — enter numbers to include (comma or newline separated, blank line to finish):\n"
    );
    for (i, section) in index.iter().enumerate() {
        println!("[{}] {}", i + 1, section.label);
    }
    println!();

    // Read choices from stdin
    let stdin = std::io::stdin();
    let mut chosen_indices: Vec<usize> = Vec::new();
    let mut seen = HashSet::new();

    for line in stdin.lock().lines() {
        let line = line.context("Failed to read stdin")?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            break;
        }
        for part in trimmed.split(',') {
            let part = part.trim();
            if let Ok(n) = part.parse::<usize>() {
                if n >= 1 && n <= index.len() && seen.insert(n) {
                    chosen_indices.push(n - 1); // convert to 0-indexed
                } else if n < 1 || n > index.len() {
                    eprintln!(
                        "  warning: {n} is out of range (1–{}), skipped",
                        index.len()
                    );
                }
            }
        }
    }

    Ok(chosen_indices
        .into_iter()
        .map(|i| IndexedSection {
            label: index[i].label.clone(),
            file: index[i].file.clone(),
            heading: index[i].heading.clone(),
        })
        .collect())
}

// --- Keyword mode ---

/// Returns all sections from the index whose headings contain any keyword.
/// Also includes sections from explicitly requested `--include` files.
fn keyword_match_sections(
    index: &[IndexedSection],
    keywords: &[String],
    include_files: &[String],
) -> Vec<IndexedSection> {
    let mut seen_labels = HashSet::new();
    let mut result = Vec::new();

    for section in index {
        let heading_lower = section.heading.to_lowercase();
        let file_stem = section
            .file
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        let keyword_match = keywords
            .iter()
            .any(|kw| heading_lower.contains(kw.as_str()));
        let explicit_match = include_files.iter().any(|f| f == file_stem);

        if (keyword_match || explicit_match) && seen_labels.insert(section.label.clone()) {
            result.push(IndexedSection {
                label: section.label.clone(),
                file: section.file.clone(),
                heading: section.heading.clone(),
            });
        }
    }

    result
}

// --- Document assembly ---

fn build_document(
    task: &str,
    catme_excerpt: &str,
    include_files: &[String],
    llmd: &std::path::Path,
    sections: &[IndexedSection],
) -> Result<String> {
    let mut doc = String::new();

    if !task.is_empty() {
        doc.push_str("# Task Context\n\n## Task\n\n");
        doc.push_str(task);
        if !task.ends_with('\n') {
            doc.push('\n');
        }
        doc.push('\n');
    } else {
        doc.push_str("# Task Context\n\n");
    }

    doc.push_str("## Project Overview\n\n");
    doc.push_str(catme_excerpt);
    doc.push('\n');

    // Explicitly included full files (--include flag)
    for topic in include_files {
        let path = llmd.join(format!("{topic}.md"));
        if path.exists() {
            let content =
                fs::read_to_string(&path).with_context(|| format!("Cannot read {topic}.md"))?;
            doc.push_str(&format!("## {topic}\n\n"));
            doc.push_str(&content);
            if !content.ends_with('\n') {
                doc.push('\n');
            }
            doc.push('\n');
        }
    }

    if !sections.is_empty() {
        doc.push_str("## Relevant Sections\n\n");
        // Group sections by file to avoid repeated file headers
        let mut current_file: Option<&PathBuf> = None;
        for section in sections {
            if Some(&section.file) != current_file {
                let label = section
                    .file
                    .strip_prefix(llmd)
                    .unwrap_or(&section.file)
                    .with_extension("")
                    .display()
                    .to_string();
                doc.push_str(&format!("### {label}\n\n"));
                current_file = Some(&section.file);
            }
            let content = match fs::read_to_string(&section.file) {
                Ok(c) => c,
                Err(_) => continue,
            };
            if let Some(text) = markdown::extract_section(&content, &section.heading) {
                doc.push_str(&text);
                if !text.ends_with('\n') {
                    doc.push('\n');
                }
                doc.push('\n');
            }
        }
    }

    Ok(doc)
}

// --- Helpers ---

fn load_task(args: &ComposeArgs) -> Result<String> {
    if let Some(path) = &args.from {
        return fs::read_to_string(path)
            .with_context(|| format!("Cannot read task file {}", path.display()));
    }
    match &args.task {
        Some(t) if t == "-" => {
            let mut buf = String::new();
            std::io::stdin().read_line(&mut buf)?;
            Ok(buf)
        }
        Some(t) => Ok(t.clone()),
        None => Ok(String::new()),
    }
}

fn extract_catme_excerpt(catme: &str) -> String {
    let summary = markdown::extract_section(catme, "Project Summary");
    let stack = markdown::extract_section(catme, "Technology Stack");
    let build = markdown::extract_section(catme, "Build");

    match (summary, stack, build) {
        (Some(s), Some(t), Some(b)) => format!("{s}\n\n{t}\n\n{b}\n"),
        (Some(s), Some(t), None) => format!("{s}\n\n{t}\n"),
        (Some(s), None, _) => format!("{s}\n"),
        _ => catme.lines().take(40).collect::<Vec<_>>().join("\n") + "\n",
    }
}

fn extract_keywords(task: &str) -> Vec<String> {
    let mut seen = HashSet::new();
    task.split_whitespace()
        .map(|w| {
            w.trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase()
        })
        .filter(|w| w.len() > 3)
        .filter(|w| seen.insert(w.clone()))
        .collect()
}
