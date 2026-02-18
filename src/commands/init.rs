//! `llmd init` — initialise a .llmd/ directory for the current project.
//!
//! Scans the project root for known agent markdown files and copies them into
//! .llmd/imported/. Then generates a catme.md navigation index.

use anyhow::{Context, Result, bail};
use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};

use crate::discovery;

#[derive(Parser)]
pub struct InitArgs {
    /// Re-scan and regenerate catme.md without wiping existing .llmd/ files
    #[arg(long)]
    pub update: bool,

    /// Project root to initialise (defaults to current directory)
    #[arg(default_value = ".")]
    pub root: PathBuf,
}

pub fn run(args: InitArgs) -> Result<()> {
    let root = args.root.canonicalize().context("Invalid project root")?;
    let llmd = root.join(".llmd");
    let imported = llmd.join("imported");
    let catme = llmd.join("catme.md");

    if llmd.exists() && !args.update {
        bail!(".llmd/ already exists. Use --update to refresh it without losing existing files.");
    }

    fs::create_dir_all(&imported).context("Failed to create .llmd/imported/")?;

    let discovered = discovery::discover(&root);

    let mut imported_names: Vec<(String, &'static str)> = Vec::new();

    for file in &discovered {
        let dest_name = flatten_name(&file.path, &root);
        let dest = imported.join(&dest_name);

        fs::copy(&file.path, &dest).with_context(|| {
            format!("Failed to copy {} to .llmd/imported/", file.path.display())
        })?;

        let rel = file.path.strip_prefix(&root).unwrap_or(&file.path);
        println!("  imported: {}", rel.display());
        imported_names.push((dest_name, file.format));
    }

    if discovered.is_empty() {
        println!("  No known agent files found — creating a blank catme.md.");
    }

    let catme_content = generate_catme(&root, &imported_names);
    fs::write(&catme, catme_content).context("Failed to write catme.md")?;

    println!("\nInitialised .llmd/ at {}", llmd.display());
    println!("  catme.md written — edit it to describe your project.");
    if !imported_names.is_empty() {
        println!(
            "  {} file(s) imported into .llmd/imported/",
            imported_names.len()
        );
    }

    Ok(())
}

/// Converts a file path like `.github/copilot-instructions.md` into a flat
/// filename `github-copilot-instructions.md` safe for use in imported/.
fn flatten_name(path: &Path, root: &Path) -> String {
    let rel = path.strip_prefix(root).unwrap_or(path);
    rel.components()
        .map(|c| {
            c.as_os_str()
                .to_string_lossy()
                .trim_start_matches('.')
                .to_string()
        })
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Generates the content of catme.md.
fn generate_catme(root: &Path, imported: &[(String, &'static str)]) -> String {
    let project_name = root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("this project");

    let mut out = String::new();

    out.push_str(&format!("# {project_name}\n\n"));
    out.push_str(
        "> **Agent entry point.** Read this file first to orient yourself in the project.\n\n",
    );

    out.push_str("## Project Summary\n\n");
    out.push_str("<!-- Describe what this project does and why it exists. One paragraph. -->\n\n");

    out.push_str("## Technology Stack\n\n");
    out.push_str("<!-- List the primary language, frameworks, and key dependencies. -->\n\n");

    out.push_str("## Build & Test\n\n");
    out.push_str("<!-- Exact commands to build, test, and lint the project. -->\n\n");
    out.push_str("```sh\n# build\n# test\n# lint\n```\n\n");

    out.push_str("## Navigation\n\n");
    out.push_str("Topic documentation lives in `.llmd/`. Start here, then follow links.\n\n");

    if !imported.is_empty() {
        out.push_str("### Imported Agent Config Files\n\n");
        out.push_str("These files were discovered in your project and imported automatically:\n\n");
        for (name, description) in imported {
            out.push_str(&format!(
                "- [imported/{name}](imported/{name}) — {description}\n"
            ));
        }
        out.push('\n');
    }

    out.push_str("## Rules of Engagement\n\n");
    out.push_str(
        "<!-- List architectural constraints and \"don't touch\" zones for agents. -->\n\n",
    );

    out.push_str("## Context Map\n\n");
    out.push_str("<!-- Map project modules to their documentation. Example:\n");
    out.push_str("- `src/auth/` → [auth-flow.md](auth-flow.md)\n");
    out.push_str("- `src/api/` → [api-standards.md](api-standards.md)\n");
    out.push_str("-->\n");

    out
}
