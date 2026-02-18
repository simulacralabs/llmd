//! `llmd build` — build a static mdbook site from .llmd/.
//!
//! Generates a temporary mdbook project (book.toml + src/) from the .llmd/
//! directory, then calls `mdbook build`. Output goes to .llmd/book/ by default.
//! Requires mdbook to be installed: `cargo install mdbook`

use anyhow::{bail, Context, Result};
use clap::Parser;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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
            fs::copy(file, &dest)
                .with_context(|| format!("Failed to copy {}", file.display()))?;
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
