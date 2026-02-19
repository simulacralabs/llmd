//! File I/O for the issue tracker.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use iso8601_timestamp::Timestamp;

use super::frontmatter::{parse_issue, serialize_issue};
use super::models::{Config, Issue};

/// Path to config.json inside issues dir.
pub fn config_path(issues_dir: &Path) -> PathBuf {
    issues_dir.join("config.json")
}

/// Returns current timestamp in ISO 8601 format.
pub fn now_iso() -> String {
    Timestamp::now_utc().to_string()
}

/// Loads config from .llmd/issues/config.json.
pub fn load_config(issues_dir: &Path) -> Result<Config> {
    let path = config_path(issues_dir);
    let content = fs::read_to_string(&path).with_context(|| {
        format!(
            "Cannot read {}. Run `llmd issue init` first.",
            path.display()
        )
    })?;
    serde_json::from_str(&content).context("Invalid config.json")
}

/// Saves config to .llmd/issues/config.json.
pub fn save_config(issues_dir: &Path, config: &Config) -> Result<()> {
    let path = config_path(issues_dir);
    let content = serde_json::to_string_pretty(config).context("Failed to serialize config")?;
    fs::write(&path, content).with_context(|| format!("Cannot write {}", path.display()))
}

/// Loads all issues from .llmd/issues/.
pub fn load_all_issues(issues_dir: &Path) -> Result<HashMap<u32, Issue>> {
    let mut map = HashMap::new();
    let entries = fs::read_dir(issues_dir).context("Cannot read issues directory")?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        if path.file_name().and_then(|n| n.to_str()) == Some("config.json") {
            continue;
        }
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let id_str = stem.split('-').next().unwrap_or("");
        let Ok(id) = id_str.parse::<u32>() else {
            continue;
        };
        let content =
            fs::read_to_string(&path).with_context(|| format!("Cannot read {}", path.display()))?;
        if let Some(issue) = parse_issue(&content, id) {
            map.insert(id, issue);
        }
    }
    Ok(map)
}

/// Resolves id or slug to an issue file path.
pub fn resolve_issue_path(issues_dir: &Path, id_or_slug: &str) -> Option<PathBuf> {
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

/// Writes an issue to disk.
pub fn write_issue(issues_dir: &Path, issue: &Issue) -> Result<()> {
    let filename = format!("{:03}-{}.md", issue.id, issue.slug);
    let path = issues_dir.join(&filename);
    let content = serialize_issue(issue);
    fs::write(&path, content).with_context(|| format!("Cannot write {}", path.display()))
}
