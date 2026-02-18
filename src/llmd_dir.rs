//! Locates and validates the .llmd/ directory.
//!
//! Searches upward from the current working directory to find the project root
//! (identified by the presence of .llmd/, Cargo.toml, package.json, .git, etc.).

use anyhow::{Result, bail};
use std::path::{Path, PathBuf};

/// Resolves the path to the `.llmd/` directory, searching upward from `start`.
///
/// Returns an error if no `.llmd/` directory is found. Use `llmd init` to create one.
pub fn find(start: &Path) -> Result<PathBuf> {
    let mut current = start.to_path_buf();
    loop {
        let candidate = current.join(".llmd");
        if candidate.is_dir() {
            return Ok(candidate);
        }
        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => bail!(
                "No .llmd/ directory found. Run `llmd init` in your project root to create one."
            ),
        }
    }
}

/// Returns the path to `catme.md` inside the given `.llmd/` directory.
pub fn catme_path(llmd: &Path) -> PathBuf {
    llmd.join("catme.md")
}

/// Lists all `.md` files in all subdirectories of the `.llmd/` directory, recursively.
pub fn list_all_files(llmd: &Path) -> Vec<PathBuf> {
    use walkdir::WalkDir;
    WalkDir::new(llmd)
        .into_iter()
        .flatten()
        .filter(|e| {
            e.file_type().is_file() && e.path().extension().and_then(|x| x.to_str()) == Some("md")
        })
        .map(|e| e.path().to_path_buf())
        .collect()
}
