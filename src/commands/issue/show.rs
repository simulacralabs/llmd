//! `llmd issue show`

use anyhow::{Context, Result};
use clap::Parser;
use std::fs;

use crate::issues::{load_all_issues, resolve_issue_path};
use crate::llmd_dir;

#[derive(Parser)]
pub struct ShowArgs {
    pub id_or_slug: String,

    #[arg(long)]
    pub json: bool,
}

pub fn run(args: ShowArgs) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let llmd = llmd_dir::find(&cwd)?;
    let issues_dir = llmd_dir::issues_path(&llmd);

    if !issues_dir.is_dir() {
        anyhow::bail!(".llmd/issues/ not found. Run `llmd issue init` first.");
    }

    let path = resolve_issue_path(&issues_dir, &args.id_or_slug)
        .with_context(|| format!("Issue \"{}\" not found", args.id_or_slug))?;

    let content =
        fs::read_to_string(&path).with_context(|| format!("Cannot read {}", path.display()))?;

    if args.json {
        let issues = load_all_issues(&issues_dir)?;
        let id: u32 = path
            .file_stem()
            .and_then(|s| s.to_str())
            .and_then(|s| s.split('-').next())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        if let Some(issue) = issues.get(&id) {
            let out = serde_json::json!({
                "id": issue.id,
                "title": issue.title,
                "slug": issue.slug,
                "type": issue.issue_type,
                "status": issue.status,
                "priority": issue.priority,
                "labels": issue.labels,
                "assignee": issue.assignee,
                "milestone": issue.milestone,
                "parent": issue.parent,
                "dependencies": issue.dependencies,
                "epic_children": issue.epic_children,
                "points": issue.points,
                "due": issue.due,
                "created_at": issue.created_at,
                "updated_at": issue.updated_at,
            });
            println!("{}", serde_json::to_string_pretty(&out)?);
        }
    } else {
        print!("{content}");
        if !content.ends_with('\n') {
            println!();
        }
    }
    Ok(())
}
