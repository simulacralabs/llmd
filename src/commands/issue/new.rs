//! `llmd issue new`

use crate::issues::models::{Issue, Label};
use crate::issues::{file_ops, load_config, save_config, write_issue};
use crate::llmd_dir;
use anyhow::{Context, Result};
use clap::Parser;

fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
}

#[derive(Parser)]
pub struct NewArgs {
    pub title: String,

    #[arg(long, default_value = "task")]
    pub r#type: String,

    #[arg(long, default_value = "medium")]
    pub priority: String,

    #[arg(long, value_delimiter = ',')]
    pub labels: Vec<String>,

    #[arg(long)]
    pub assignee: Option<String>,

    #[arg(long)]
    pub parent: Option<u32>,

    #[arg(long)]
    pub milestone: Option<String>,

    #[arg(long)]
    pub points: Option<u32>,

    #[arg(long)]
    pub due: Option<String>,

    #[arg(long)]
    pub dep: Vec<u32>,
}

pub fn run(args: NewArgs) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let llmd = llmd_dir::find(&cwd)?;
    let issues_dir = llmd_dir::issues_path(&llmd);

    if !issues_dir.is_dir() {
        anyhow::bail!(".llmd/issues/ not found. Run `llmd issue init` first.");
    }

    let mut config = load_config(&issues_dir)?;
    let id = config.next_id;
    config.next_id += 1;
    save_config(&issues_dir, &config)?;

    let slug = slugify(&args.title);
    let now = file_ops::now_iso();

    let labels: Vec<Label> = args
        .labels
        .iter()
        .map(|s| {
            let (name, color) = match s.split_once(':') {
                Some((n, c)) => (n.to_string(), Some(c.to_string())),
                None => (s.clone(), None),
            };
            Label { name, color }
        })
        .collect();

    let issue = Issue {
        id,
        title: args.title.clone(),
        slug: slug.clone(),
        issue_type: args.r#type,
        status: "open".to_string(),
        priority: args.priority,
        labels,
        assignee: args.assignee,
        milestone: args.milestone,
        parent: args.parent,
        dependencies: args.dep,
        epic_children: vec![],
        points: args.points,
        due: args.due,
        created_at: now.clone(),
        updated_at: now,
        body: String::new(),
    };

    write_issue(&issues_dir, &issue)?;

    if let Some(parent_id) = args.parent {
        add_child_to_epic(&issues_dir, parent_id, id)?;
    }

    eprintln!("Created issue #{} ({})", id, slug);
    Ok(())
}

fn add_child_to_epic(issues_dir: &std::path::Path, epic_id: u32, child_id: u32) -> Result<()> {
    let path = match crate::issues::resolve_issue_path(issues_dir, &epic_id.to_string()) {
        Some(p) => p,
        None => return Ok(()),
    };
    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("Cannot read {}", path.display()))?;
    let Some(mut issue) = crate::issues::parse_issue(&content, epic_id) else {
        return Ok(());
    };
    if !issue.epic_children.contains(&child_id) {
        issue.epic_children.push(child_id);
        issue.epic_children.sort();
        issue.updated_at = file_ops::now_iso();
        write_issue(issues_dir, &issue)?;
    }
    Ok(())
}
