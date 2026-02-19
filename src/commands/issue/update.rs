//! `llmd issue update`

use anyhow::{Context, Result};
use clap::Parser;
use std::fs;

use crate::issues::frontmatter::parse_issue;
use crate::issues::models::Label;
use crate::issues::{file_ops, resolve_issue_path, write_issue};
use crate::llmd_dir;

#[derive(Parser)]
pub struct UpdateArgs {
    pub id_or_slug: String,

    #[arg(long)]
    pub status: Option<String>,

    #[arg(long)]
    pub priority: Option<String>,

    #[arg(long)]
    pub assignee: Option<String>,

    #[arg(long)]
    pub milestone: Option<String>,

    #[arg(long)]
    pub points: Option<u32>,

    #[arg(long)]
    pub due: Option<String>,

    #[arg(long)]
    pub add_dep: Option<u32>,

    #[arg(long)]
    pub add_label: Option<String>,

    #[arg(long)]
    pub parent: Option<u32>,

    #[arg(long)]
    pub add_comment: Option<String>,

    #[arg(long)]
    pub author: Option<String>,
}

pub fn run(args: UpdateArgs) -> Result<()> {
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

    let id: u32 = path
        .file_stem()
        .and_then(|s| s.to_str())
        .and_then(|s| s.split('-').next())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let mut issue = parse_issue(&content, id).context("Failed to parse issue")?;

    if let Some(s) = args.status {
        issue.status = s;
    }
    if let Some(p) = args.priority {
        issue.priority = p;
    }
    if let Some(a) = args.assignee {
        issue.assignee = Some(a);
    }
    if let Some(m) = args.milestone {
        issue.milestone = Some(m);
    }
    if let Some(pt) = args.points {
        issue.points = Some(pt);
    }
    if let Some(d) = args.due {
        issue.due = Some(d);
    }
    if let Some(dep) = args.add_dep
        && !issue.dependencies.contains(&dep)
    {
        issue.dependencies.push(dep);
        issue.dependencies.sort();
    }
    if let Some(ref l) = args.add_label {
        let (name, color) = match l.split_once(':') {
            Some((n, c)) => (n.to_string(), Some(c.to_string())),
            None => (l.clone(), None),
        };
        if !issue.labels.iter().any(|x| x.name == name) {
            issue.labels.push(Label { name, color });
        }
    }
    if let Some(p) = args.parent {
        issue.parent = Some(p);
    }
    if let Some(ref comment_text) = args.add_comment {
        let author = args
            .author
            .unwrap_or_else(|| std::env::var("USER").unwrap_or_else(|_| "unknown".to_string()));
        let date = file_ops::now_iso();
        append_comment(&mut issue.body, &author, &date, comment_text);
    }

    issue.updated_at = file_ops::now_iso();
    write_issue(&issues_dir, &issue)?;

    eprintln!("Updated issue #{}", id);
    Ok(())
}

fn append_comment(body: &mut String, author: &str, date: &str, text: &str) {
    let entry = format!(
        "- author: \"{}\"\n  date: \"{}\"\n  body: \"{}\"\n",
        author.replace('\\', "\\\\").replace('"', "\\\""),
        date,
        text.replace('\\', "\\\\").replace('"', "\\\"")
    );

    if !body.contains("## Comments") {
        if !body.trim().is_empty() {
            body.push_str("\n\n");
        }
        body.push_str("## Comments\n\n```yaml\n");
        body.push_str(&entry);
        body.push_str("```\n");
        return;
    }

    if let Some(idx) = body.find("```yaml") {
        let after_yaml = idx + 7;
        if let Some(close) = body[after_yaml..].find("\n```") {
            let insert_at = after_yaml + close;
            body.insert_str(insert_at, &entry);
            return;
        }
    }
    if let Some(idx) = body.rfind("```") {
        body.insert_str(idx, &entry);
    } else {
        body.push_str(&entry);
    }
}
