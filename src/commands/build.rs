//! `llmd build` — build a static mdbook site from .llmd/.
//!
//! Generates a temporary mdbook project (book.toml + src/) from the .llmd/
//! directory, then calls `mdbook build`. Output goes to .llmd/book/ by default.
//! Requires mdbook to be installed: `cargo install mdbook`

use anyhow::{Context, Result, bail};
use clap::Parser;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::issues;
use crate::llmd_dir;

#[derive(Parser)]
pub struct BuildArgs {
    /// Output directory for the built site (default: .llmd/book)
    #[arg(long, short, value_name = "DIR")]
    pub output: Option<PathBuf>,
}

pub fn run(args: BuildArgs) -> Result<()> {
    ensure_mdbook()?;

    let cwd = std::env::current_dir()?;
    let llmd = llmd_dir::find(&cwd)?;
    let book_dir = generate_mdbook(&llmd)?;

    let status = Command::new("mdbook")
        .arg("build")
        .arg(&book_dir)
        .status()
        .context("Failed to run `mdbook build`")?;

    if !status.success() {
        bail!("`mdbook build` exited with status {status}");
    }

    let built = book_dir.join("book");
    let dest = args.output.unwrap_or_else(|| llmd.join("book"));

    if dest != built {
        copy_dir_all(&built, &dest).context("Failed to copy built site to output directory")?;
        fs::remove_dir_all(&built).ok();
    }

    eprintln!("Built mdbook site at {}", dest.display());
    Ok(())
}

/// Generates a temporary mdbook project layout from the .llmd/ directory.
///
/// Returns the path to the generated mdbook project root (inside .llmd/.mdbook/).
/// This is also called by `serve` to reuse the same generation logic.
pub fn generate_mdbook(llmd: &Path) -> Result<PathBuf> {
    let book_root = llmd.join(".mdbook");
    let src_dir = book_root.join("src");
    fs::create_dir_all(&src_dir).context("Failed to create mdbook src/ directory")?;

    let all_files = llmd_dir::list_all_files(llmd);
    let catme = llmd_dir::catme_path(llmd);

    let mut summary = String::from("# Summary\n\n");

    if catme.exists() {
        fs::copy(&catme, src_dir.join("catme.md")).context("Failed to copy catme.md")?;
        summary.push_str("- [Overview](catme.md)\n");
    }

    let mut by_dir: BTreeMap<String, Vec<PathBuf>> = BTreeMap::new();

    for file in &all_files {
        if *file == catme {
            continue;
        }
        if file.starts_with(llmd.join(".mdbook")) || file.starts_with(llmd.join("book")) {
            continue;
        }
        let rel = file.strip_prefix(llmd).unwrap_or(file);
        let dir_key = rel
            .parent()
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        by_dir.entry(dir_key).or_default().push(file.clone());
    }

    for (dir_key, files) in &by_dir {
        if !dir_key.is_empty() {
            summary.push_str(&format!("\n## {dir_key}\n\n"));
        }
        if dir_key == "issues" {
            let issues_dir = llmd_dir::issues_path(llmd);
            if issues_dir.is_dir() {
                let roadmap_md = generate_roadmap_page(llmd, &issues_dir)?;
                let issues_src = src_dir.join("issues");
                fs::create_dir_all(&issues_src).context("Failed to create issues src dir")?;
                fs::write(issues_src.join("roadmap.md"), &roadmap_md)
                    .context("Failed to write roadmap.md")?;
                summary.push_str("- [Roadmap](issues/roadmap.md)\n");
            }
        }
        for file in files {
            let rel = file.strip_prefix(llmd).unwrap_or(file);
            let title = file
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("untitled");
            let rel_str = rel.display().to_string();
            let dest = src_dir.join(rel);
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent).ok();
            }
            fs::copy(file, &dest).with_context(|| format!("Failed to copy {}", file.display()))?;
            if dir_key == "issues" && title == "roadmap" {
                continue;
            }
            summary.push_str(&format!("- [{title}]({rel_str})\n"));
        }
    }

    fs::write(src_dir.join("SUMMARY.md"), &summary).context("Failed to write SUMMARY.md")?;

    let project_name = llmd
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("project");

    let book_toml = format!(
        "[book]\ntitle = \"{project_name} — llmd\"\nsrc = \"src\"\n\n\
         [output.html]\nno-section-label = true\n"
    );
    fs::write(book_root.join("book.toml"), book_toml).context("Failed to write book.toml")?;

    Ok(book_root)
}

fn generate_roadmap_page(_llmd: &Path, issues_dir: &Path) -> Result<String> {
    let issues_map = issues::load_all_issues(issues_dir).unwrap_or_default();
    let now = issues::now_iso();
    let date = now.split('T').next().unwrap_or("");

    let mut md = format!("# Roadmap\n\nGenerated from .llmd/issues/ — {date}\n\n");

    let mut by_milestone: std::collections::BTreeMap<String, Vec<&issues::Issue>> =
        std::collections::BTreeMap::new();
    for issue in issues_map.values() {
        let m = issue
            .milestone
            .clone()
            .unwrap_or_else(|| "_no_milestone".to_string());
        by_milestone.entry(m).or_default().push(issue);
    }

    for (milestone, mut list) in by_milestone {
        let m_label = if milestone == "_no_milestone" {
            "No milestone"
        } else {
            &milestone
        };
        list.sort_by_key(|i| i.id);
        md.push_str(&format!("## {m_label}\n\n"));

        let epics: Vec<_> = list.iter().filter(|i| i.issue_type == "epic").collect();
        let rest: Vec<_> = list.iter().filter(|i| i.issue_type != "epic").collect();

        if !epics.is_empty() {
            md.push_str("### Epics\n\n");
            for i in epics {
                let pts = i.points.map(|p| format!("{}pts", p)).unwrap_or_default();
                let due = i.due.as_deref().unwrap_or("");
                let meta = [&pts as &str, due]
                    .iter()
                    .filter(|s| !s.is_empty())
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(" · ");
                md.push_str(&format!(
                    "- **[#{} {}]({:03}-{}.md)** `epic` {}\n",
                    i.id, i.title, i.id, i.slug, meta
                ));
                for child_id in &i.epic_children {
                    if let Some(c) = issues_map.get(child_id) {
                        let assignee = c.assignee.as_deref().unwrap_or("—");
                        md.push_str(&format!(
                            "  - [#{} {}]({:03}-{}.md) `{}` · {} · {}\n",
                            c.id, c.title, c.id, c.slug, c.issue_type, c.status, assignee
                        ));
                    }
                }
            }
            md.push('\n');
        }

        if !rest.is_empty() {
            md.push_str("### Open Issues\n\n");
            md.push_str("| # | Title | Type | Priority | Assignee |\n");
            md.push_str("|---|-------|------|----------|----------|\n");
            for i in rest {
                let assignee = i.assignee.as_deref().unwrap_or("—");
                md.push_str(&format!(
                    "| {} | {} | {} | {} | {} |\n",
                    i.id, i.title, i.issue_type, i.priority, assignee
                ));
            }
            md.push('\n');
        }
    }

    let closed = issues_map.values().filter(|i| i.status == "closed").count();
    let total = issues_map.len();
    let pct = if total > 0 { (closed * 100) / total } else { 0 };
    let bar_len = 10;
    let filled = (pct * bar_len) / 100;
    let bar: String = "█".repeat(filled) + &"░".repeat(bar_len - filled);
    md.push_str(&format!(
        "### Progress\n\n{}/{} closed ({}%)\n\n{}",
        closed, total, pct, bar
    ));
    Ok(md)
}

fn ensure_mdbook() -> Result<()> {
    if Command::new("mdbook").arg("--version").output().is_err() {
        bail!("`mdbook` is not installed. Install it with:\n  cargo install mdbook");
    }
    Ok(())
}

fn copy_dir_all(src: &Path, dest: &Path) -> Result<()> {
    fs::create_dir_all(dest)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let dest_path = dest.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_all(&entry.path(), &dest_path)?;
        } else {
            fs::copy(entry.path(), dest_path)?;
        }
    }
    Ok(())
}
