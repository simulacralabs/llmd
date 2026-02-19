//! `llmd issue mentions`

use anyhow::Result;
use clap::Parser;
use std::fs;

use crate::llmd_dir;

#[derive(Parser)]
pub struct MentionsArgs {
    #[arg(required = false)]
    pub handle: Option<String>,
}

pub fn run(args: MentionsArgs) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let llmd = llmd_dir::find(&cwd)?;
    let issues_dir = llmd_dir::issues_path(&llmd);

    if !issues_dir.is_dir() {
        anyhow::bail!(".llmd/issues/ not found. Run `llmd issue init` first.");
    }

    let pattern = args
        .handle
        .as_ref()
        .map(|h| format!("@{}", h))
        .unwrap_or_else(|| "@\\w+".to_string());

    let re = regex_lite::Regex::new(&pattern)
        .unwrap_or_else(|_| regex_lite::Regex::new("@\\w+").unwrap());

    let entries = fs::read_dir(&issues_dir)?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let comments_section = content.split("## Comments").nth(1).unwrap_or("");
        let yaml_block = comments_section
            .split("```yaml")
            .nth(1)
            .and_then(|s| s.split("```").next())
            .unwrap_or("");

        for line in yaml_block.lines() {
            if re.is_match(line) {
                let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                let id = stem.split('-').next().unwrap_or("");
                let body = line
                    .split("body:")
                    .nth(1)
                    .and_then(|s| s.split_whitespace().next())
                    .unwrap_or(line);
                let author = line
                    .split("author:")
                    .nth(1)
                    .and_then(|s| s.split_whitespace().next())
                    .unwrap_or("?");
                let date = line
                    .split("date:")
                    .nth(1)
                    .and_then(|s| s.split_whitespace().next())
                    .unwrap_or("?");
                println!(
                    "#{} {} — {} ({}): {}",
                    id,
                    stem,
                    author.trim_matches('"'),
                    date.trim_matches('"'),
                    body.trim_matches('"')
                );
                break;
            }
        }
    }
    Ok(())
}
