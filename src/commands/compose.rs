//! `llmd compose` — compose a task-context document from .llmd/ content.
//!
//! Use `llmd index` first to view the section index, then pass section numbers
//! via `--sections`. With `--issue`, topics are auto-included from the label-to-topics
//! mapping in `.llmd/context-mappings.json` unless `--no-auto-include` is set.

use anyhow::{Context, Result};
use clap::Parser;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::{llmd_dir, markdown};

#[derive(Parser)]
pub struct ComposeArgs {
    /// Section numbers to include (from `llmd index`). Comma-separated, e.g. 1,2,3
    #[arg(long, short = 's', value_delimiter = ',')]
    pub sections: Vec<usize>,

    /// Task description. Included in the output header.
    pub task: Option<String>,

    /// Compose from this issue (id or slug). Auto-includes topics from label mapping unless --no-auto-include
    #[arg(long)]
    pub issue: Option<String>,

    /// Disable auto-inclusion of topics from issue labels (use with --issue when selecting manually)
    #[arg(long)]
    pub no_auto_include: bool,

    /// Explicitly include these topic files by name (no .md, comma-separated)
    #[arg(long, short = 'I', value_delimiter = ',')]
    pub include: Vec<String>,

    /// Read the task description from a file
    #[arg(long, short, value_name = "FILE")]
    pub from: Option<PathBuf>,

    /// Write the composed document to a file instead of stdout
    #[arg(long, short, value_name = "FILE")]
    pub output: Option<PathBuf>,
}

pub fn run(args: ComposeArgs) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let llmd = llmd_dir::find(&cwd)?;

    let catme_path = llmd_dir::catme_path(&llmd);
    let catme =
        fs::read_to_string(&catme_path).context("Cannot read catme.md — run `llmd init` first")?;

    let catme_excerpt = extract_catme_excerpt(&catme);
    let all_files = llmd_dir::list_all_files(&llmd);
    let index = build_section_index(&llmd, &all_files);

    // Resolve sections from --sections (indices into the index)
    let chosen_sections = resolve_sections_from_indices(&index, &args.sections)?;

    // Auto-include topics from issue labels when --issue is set and --no-auto-include is not
    let mut include_topics = args.include.clone();
    let header = if let Some(ref id_or_slug) = args.issue {
        let (issue_header, auto_topics) =
            load_issue_context(&llmd, id_or_slug, args.no_auto_include)?;
        if !args.no_auto_include {
            include_topics.extend(auto_topics);
            include_topics.sort();
            include_topics.dedup();
        }
        issue_header
    } else {
        let task = load_task(&args).unwrap_or_default();
        if task.is_empty() {
            String::from("# Task Context\n\n")
        } else {
            format!("# Task Context\n\n## Task\n\n{task}\n\n")
        }
    };

    let doc = build_document(
        &header,
        &catme_excerpt,
        &include_topics,
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

/// Prints the section index to stdout. Used by `llmd index`.
pub fn print_section_index(llmd: &Path) -> Result<()> {
    let all_files = llmd_dir::list_all_files(llmd);
    let index = build_section_index(llmd, &all_files);

    if index.is_empty() {
        eprintln!("No sections found in .llmd/. Add topic files first.");
        return Ok(());
    }

    eprintln!("Available sections — use with `llmd compose --sections <nums>`:\n");
    for (i, section) in index.iter().enumerate() {
        println!("[{}] {}", i + 1, section.label);
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

/// Builds a flat ordered list of all H2/H3 headings from all non-catme, non-imported, non-issues files.
fn build_section_index(llmd: &std::path::Path, all_files: &[PathBuf]) -> Vec<IndexedSection> {
    let catme = llmd_dir::catme_path(llmd);
    let imported = llmd.join("imported");
    let issues = llmd_dir::issues_path(llmd);
    let mut index = Vec::new();

    for file_path in all_files {
        if *file_path == catme || file_path.starts_with(&imported) || file_path.starts_with(&issues)
        {
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

/// Resolves section indices (1-based) to IndexedSection entries.
fn resolve_sections_from_indices(
    index: &[IndexedSection],
    indices: &[usize],
) -> Result<Vec<IndexedSection>> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for &n in indices {
        if n >= 1 && n <= index.len() && seen.insert(n) {
            let i = n - 1;
            result.push(IndexedSection {
                label: index[i].label.clone(),
                file: index[i].file.clone(),
                heading: index[i].heading.clone(),
            });
        } else if n >= 1 && n <= index.len() {
            // duplicate, skip
        } else {
            anyhow::bail!(
                "Section index {n} is out of range (1–{}). Run `llmd index` to see available sections.",
                index.len()
            );
        }
    }
    Ok(result)
}

/// Loads context-mappings.json. Returns label -> topics map.
fn load_context_mappings(llmd: &Path) -> HashMap<String, Vec<String>> {
    let path = llmd.join("context-mappings.json");
    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return HashMap::new(),
    };
    #[derive(serde::Deserialize)]
    struct Mappings {
        label_to_topics: Option<HashMap<String, Vec<String>>>,
    }
    match serde_json::from_str::<Mappings>(&content) {
        Ok(m) => m.label_to_topics.unwrap_or_default(),
        Err(_) => HashMap::new(),
    }
}

/// Loads issue context: formatted header and auto-included topics from label mapping.
fn load_issue_context(
    llmd: &Path,
    id_or_slug: &str,
    no_auto_include: bool,
) -> Result<(String, Vec<String>)> {
    let issues_dir = llmd_dir::issues_path(llmd);
    if !issues_dir.is_dir() {
        anyhow::bail!(
            ".llmd/issues/ not found. Run `llmd issue init` to create the issue tracker."
        );
    }

    let issue_path = resolve_issue_file(&issues_dir, id_or_slug)
        .with_context(|| format!("Issue \"{id_or_slug}\" not found in .llmd/issues/"))?;

    let content = fs::read_to_string(&issue_path)
        .with_context(|| format!("Cannot read {}", issue_path.display()))?;

    let (labels, header) = parse_issue_frontmatter(&content, id_or_slug);

    let auto_topics = if no_auto_include {
        Vec::new()
    } else {
        let mapping = load_context_mappings(llmd);
        let mut topics = Vec::new();
        for label in &labels {
            if let Some(mapped) = mapping.get(label) {
                topics.extend(mapped.iter().cloned());
            }
        }
        topics.sort();
        topics.dedup();
        topics
    };

    Ok((header, auto_topics))
}

/// Resolves id or slug to an issue file path.
fn resolve_issue_file(issues_dir: &Path, id_or_slug: &str) -> Option<PathBuf> {
    let entries = fs::read_dir(issues_dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let stem = path.file_stem().and_then(|s| s.to_str())?;
        if let Ok(id) = id_or_slug.parse::<u32>()
            && stem.starts_with(&format!("{id:03}-"))
        {
            return Some(path);
        }
        if stem.ends_with(&format!("-{id_or_slug}")) || stem == id_or_slug {
            return Some(path);
        }
    }
    None
}

/// Parses issue frontmatter to extract labels and build a formatted header.
fn parse_issue_frontmatter(content: &str, id_or_slug: &str) -> (Vec<String>, String) {
    let (labels, title) = extract_labels_and_title_from_frontmatter(content);
    let body = extract_issue_body(content);

    let labels_str = if labels.is_empty() {
        String::new()
    } else {
        format!(" · {}", labels.join(", "))
    };

    let header = format!(
        r#"# Context: #{id_or_slug} {title}

## Issue

**#{id_or_slug}**{labels_str}

{body}

"#,
        title = title.unwrap_or_else(|| "Untitled".to_string()),
        labels_str = labels_str,
        body = body.unwrap_or_default()
    );

    (labels, header)
}

/// Extracts label names and title from YAML frontmatter.
fn extract_labels_and_title_from_frontmatter(content: &str) -> (Vec<String>, Option<String>) {
    let labels = extract_labels_from_frontmatter(content);
    let title = content
        .strip_prefix("---\n")
        .and_then(|s| s.split("\n---").next())
        .and_then(|fm| {
            fm.lines()
                .find(|l| l.trim_start().starts_with("title:"))
                .and_then(|l| l.split_once(':'))
                .map(|(_, v)| v.trim().trim_matches('"').trim_matches('\'').to_string())
        })
        .filter(|s| !s.is_empty());
    (labels, title)
}

/// Extracts label names from YAML frontmatter. Supports both `labels: [a, b]` and `labels:\n  - name: a`.
fn extract_labels_from_frontmatter(content: &str) -> Vec<String> {
    let Some(fm) = content
        .strip_prefix("---\n")
        .and_then(|s| s.split("\n---").next())
    else {
        return Vec::new();
    };

    let mut labels = Vec::new();
    let mut in_labels = false;
    let mut indent = 0;

    for line in fm.lines() {
        if let Some(rest) = line.strip_prefix("labels:") {
            let rest = rest.trim();
            if let Some(bracket) = rest.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                for part in bracket.split(',') {
                    let label = part.trim().trim_matches('"').trim_matches('\'').to_string();
                    if !label.is_empty() {
                        labels.push(label);
                    }
                }
            }
            in_labels = true;
            indent = line.len() - line.trim_start().len();
        } else if in_labels {
            let line_indent = line.len() - line.trim_start().len();
            if line_indent <= indent && !line.trim().is_empty() {
                in_labels = false;
            } else if line.contains("name:")
                && let Some(after) = line.split("name:").nth(1)
            {
                let name = after
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string();
                if !name.is_empty() {
                    labels.push(name);
                }
            }
        }
    }
    labels
}

/// Extracts the body (markdown after frontmatter) from issue content.
fn extract_issue_body(content: &str) -> Option<String> {
    content
        .strip_prefix("---\n")
        .and_then(|s| s.split("\n---\n").nth(1))
        .map(|s| s.trim().to_string())
}

// --- Document assembly ---

fn build_document(
    header: &str,
    catme_excerpt: &str,
    include_files: &[String],
    llmd: &std::path::Path,
    sections: &[IndexedSection],
) -> Result<String> {
    let mut doc = String::new();

    doc.push_str(header);
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
