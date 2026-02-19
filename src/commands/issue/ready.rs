//! `llmd issue ready`

use anyhow::Result;
use clap::Parser;

use crate::issues::{load_all_issues, ready_tasks};
use crate::llmd_dir;

#[derive(Parser)]
pub struct ReadyArgs {
    #[arg(long)]
    pub json: bool,

    #[arg(long, name = "type")]
    pub type_filter: Option<String>,

    #[arg(long)]
    pub milestone: Option<String>,

    #[arg(long)]
    pub assignee: Option<String>,
}

pub fn run(args: ReadyArgs) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let llmd = llmd_dir::find(&cwd)?;
    let issues_dir = llmd_dir::issues_path(&llmd);

    if !issues_dir.is_dir() {
        anyhow::bail!(".llmd/issues/ not found. Run `llmd issue init` first.");
    }

    let issues = load_all_issues(&issues_dir)?;
    let ready = ready_tasks(&issues, true);

    let filtered: Vec<_> = ready
        .iter()
        .filter(|i| {
            args.type_filter.as_ref().is_none_or(|t| i.issue_type == *t)
                && args
                    .milestone
                    .as_ref()
                    .is_none_or(|m| i.milestone.as_deref() == Some(m.as_str()))
                && args
                    .assignee
                    .as_ref()
                    .is_none_or(|a| i.assignee.as_deref() == Some(a.as_str()))
        })
        .collect();

    if args.json {
        let out: Vec<serde_json::Value> = filtered
            .iter()
            .map(|i| {
                serde_json::json!({
                    "id": i.id,
                    "title": i.title,
                    "slug": i.slug,
                    "type": i.issue_type,
                    "status": i.status,
                    "priority": i.priority,
                    "assignee": i.assignee,
                    "milestone": i.milestone,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        for i in filtered {
            let assignee = i.assignee.as_deref().unwrap_or("—");
            println!("#{} {} [{}] · {}", i.id, i.title, i.issue_type, assignee);
        }
    }
    Ok(())
}
