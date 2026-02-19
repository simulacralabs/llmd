//! `llmd issue tree`

use anyhow::{Context, Result};
use clap::Parser;

use crate::issues::{epic_tree, load_all_issues, resolve_issue_path};
use crate::llmd_dir;

#[derive(Parser)]
pub struct TreeArgs {
    pub id: u32,
}

pub fn run(args: TreeArgs) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let llmd = llmd_dir::find(&cwd)?;
    let issues_dir = llmd_dir::issues_path(&llmd);

    if !issues_dir.is_dir() {
        anyhow::bail!(".llmd/issues/ not found. Run `llmd issue init` first.");
    }

    let _path = resolve_issue_path(&issues_dir, &args.id.to_string())
        .with_context(|| format!("Issue #{} not found", args.id))?;

    let issues = load_all_issues(&issues_dir)?;
    let tree = epic_tree(&issues, args.id);

    for (id, depth) in tree {
        let indent = "  ".repeat(depth);
        if let Some(issue) = issues.get(&id) {
            let assignee = issue.assignee.as_deref().unwrap_or("");
            let assignee_str = if assignee.is_empty() {
                String::new()
            } else {
                format!(" · {}", assignee)
            };
            println!(
                "{}#{} {} [{}] {}{}",
                indent, id, issue.title, issue.issue_type, issue.status, assignee_str
            );
        }
    }
    Ok(())
}
