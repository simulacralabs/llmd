//! `llmd issue list`

use anyhow::Result;
use clap::Parser;

use crate::issues::load_all_issues;
use crate::llmd_dir;

#[derive(Parser)]
pub struct ListArgs {
    #[arg(long)]
    pub json: bool,

    #[arg(long)]
    pub status: Option<String>,

    #[arg(long, name = "type")]
    pub type_filter: Option<String>,

    #[arg(long)]
    pub milestone: Option<String>,

    #[arg(long)]
    pub assignee: Option<String>,

    #[arg(long)]
    pub epic: Option<u32>,
}

pub fn run(args: ListArgs) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let llmd = llmd_dir::find(&cwd)?;
    let issues_dir = llmd_dir::issues_path(&llmd);

    if !issues_dir.is_dir() {
        anyhow::bail!(".llmd/issues/ not found. Run `llmd issue init` first.");
    }

    let mut issues = load_all_issues(&issues_dir)?;

    if let Some(epic_id) = args.epic {
        issues.retain(|_, i| i.parent == Some(epic_id));
    }
    if let Some(ref status) = args.status {
        issues.retain(|_, i| i.status == *status);
    }
    if let Some(ref t) = args.type_filter {
        issues.retain(|_, i| i.issue_type == *t);
    }
    if let Some(ref m) = args.milestone {
        issues.retain(|_, i| i.milestone.as_deref() == Some(m.as_str()));
    }
    if let Some(ref a) = args.assignee {
        issues.retain(|_, i| i.assignee.as_deref() == Some(a.as_str()));
    }

    let mut list: Vec<_> = issues.values().collect();
    list.sort_by_key(|i| i.id);

    if args.json {
        let out: Vec<serde_json::Value> = list
            .iter()
            .map(|i| {
                serde_json::json!({
                    "id": i.id,
                    "title": i.title,
                    "slug": i.slug,
                    "type": i.issue_type,
                    "status": i.status,
                    "priority": i.priority,
                    "labels": i.labels.iter().map(|l| serde_json::json!({"name": l.name, "color": l.color})).collect::<Vec<_>>(),
                    "assignee": i.assignee,
                    "milestone": i.milestone,
                    "parent": i.parent,
                    "dependencies": i.dependencies,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        for i in list {
            let labels: String = i
                .labels
                .iter()
                .map(|l| l.name.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            let assignee = i.assignee.as_deref().unwrap_or("—");
            println!(
                "#{} {} [{}] {} · {} · {}",
                i.id, i.title, i.issue_type, i.status, assignee, labels
            );
        }
    }
    Ok(())
}
