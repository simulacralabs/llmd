//! Hand-rolled YAML frontmatter parser for issue files.
//!
//! Handles scalars, flat arrays, and label objects. No serde_yaml dependency.

use std::collections::HashMap;

use crate::issues::models::{Issue, Label};

/// Parses an issue file (--- ... ---\n\nbody) into an Issue.
pub fn parse_issue(content: &str, id: u32) -> Option<Issue> {
    let (fm, body) = split_frontmatter(content)?;
    let mut map = parse_frontmatter_map(fm);

    let title = map
        .remove("title")
        .unwrap_or_else(|| "Untitled".to_string());
    let slug = map.remove("slug").unwrap_or_else(|| slugify(&title));
    let issue_type = map.remove("type").unwrap_or_else(|| "task".to_string());
    let status = map.remove("status").unwrap_or_else(|| "open".to_string());
    let priority = map
        .remove("priority")
        .unwrap_or_else(|| "medium".to_string());
    let assignee = map
        .remove("assignee")
        .filter(|s| !s.is_empty() && s != "null");
    let milestone = map.remove("milestone").filter(|s| !s.is_empty());
    let parent = map.remove("parent").and_then(|s| s.parse().ok());
    let created_at = map.remove("created_at").unwrap_or_default();
    let updated_at = map
        .remove("updated_at")
        .unwrap_or_else(|| created_at.clone());
    let due = map.remove("due").filter(|s| !s.is_empty());
    let points = map.remove("points").and_then(|s| s.parse().ok());

    let labels = parse_labels(fm);
    let dependencies = parse_array_field(fm, "dependencies");
    let epic_children = parse_array_field(fm, "epic_children");

    Some(Issue {
        id,
        title,
        slug,
        issue_type,
        status,
        priority,
        labels,
        assignee,
        milestone,
        parent,
        dependencies,
        epic_children,
        points,
        due,
        created_at,
        updated_at,
        body: body.trim().to_string(),
    })
}

fn split_frontmatter(content: &str) -> Option<(&str, &str)> {
    let content = content.strip_prefix("---\n")?;
    let (fm, rest) = content.split_once("\n---")?;
    let body = rest.strip_prefix('\n').unwrap_or(rest);
    Some((fm.trim(), body))
}

fn parse_frontmatter_map(fm: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let mut key = String::new();
    let mut value = String::new();
    let mut in_key = true;

    for line in fm.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if line.starts_with(|c: char| c.is_alphabetic() || c == '_') {
            if !key.is_empty() {
                map.insert(
                    key.clone(),
                    value
                        .trim()
                        .trim_matches('"')
                        .trim_matches('\'')
                        .to_string(),
                );
            }
            if let Some((k, v)) = line.split_once(':') {
                key = k.trim().to_string();
                value = v.trim().trim_matches('"').trim_matches('\'').to_string();
                in_key = false;
            }
        } else if line.starts_with('-') && !key.is_empty() {
            // Array element or nested object - skip for simple scalar map
            continue;
        } else if !in_key && !value.is_empty() && line.starts_with(' ') {
            value.push(' ');
            value.push_str(line.trim());
        }
    }
    if !key.is_empty() {
        map.insert(
            key,
            value
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_string(),
        );
    }
    map
}

fn parse_labels(fm: &str) -> Vec<Label> {
    let mut labels = Vec::new();
    let mut in_labels = false;
    let mut base_indent = 0usize;

    let lines: Vec<&str> = fm.lines().collect();
    let mut i = 0usize;

    while i < lines.len() {
        let line = lines[i];
        if line.trim_start().starts_with("labels:") {
            let rest = line.split_once(':').map(|(_, r)| r.trim()).unwrap_or("");
            if let Some(bracket) = rest.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                for part in bracket.split(',') {
                    let name = part.trim().trim_matches('"').trim_matches('\'').to_string();
                    if !name.is_empty() {
                        labels.push(Label { name, color: None });
                    }
                }
            }
            in_labels = true;
            base_indent = line.len() - line.trim_start().len();
            i += 1;
            continue;
        }
        if in_labels {
            let trimmed = line.trim_start();
            let line_indent = line.len() - trimmed.len();
            if line_indent <= base_indent && !trimmed.is_empty() && !trimmed.starts_with('-') {
                in_labels = false;
                i += 1;
                continue;
            }
            if trimmed.starts_with("- name:") || trimmed.starts_with("-  name:") {
                let name = trimmed
                    .split("name:")
                    .nth(1)
                    .and_then(|s| s.split_whitespace().next())
                    .map(|s| s.trim_matches('"').trim_matches('\'').to_string())
                    .unwrap_or_default();
                let mut color = None;
                if i + 1 < lines.len() {
                    let next = lines[i + 1];
                    if next.trim_start().starts_with("color:") {
                        color = next
                            .split("color:")
                            .nth(1)
                            .and_then(|s| s.split_whitespace().next())
                            .map(|s| s.trim_matches('"').to_string());
                        i += 1;
                    }
                }
                if !name.is_empty() {
                    labels.push(Label { name, color });
                }
            }
        }
        i += 1;
    }
    labels
}

fn parse_array_field(fm: &str, field: &str) -> Vec<u32> {
    let mut result = Vec::new();
    for line in fm.lines() {
        if line.trim_start().starts_with(&format!("{field}:")) {
            let rest = line.split_once(':').map(|(_, r)| r.trim()).unwrap_or("");
            if let Some(bracket) = rest.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                for part in bracket.split(',') {
                    if let Ok(n) = part.trim().parse::<u32>() {
                        result.push(n);
                    }
                }
            }
            break;
        }
    }
    result
}

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

/// Serializes an Issue to frontmatter + body for writing.
pub fn serialize_issue(issue: &Issue) -> String {
    let mut out = String::from("---\n");
    out.push_str(&format!("id: {}\n", issue.id));
    out.push_str(&format!("title: \"{}\"\n", escape_yaml_str(&issue.title)));
    out.push_str(&format!("slug: \"{}\"\n", issue.slug));
    out.push_str(&format!("type: {}\n", issue.issue_type));
    out.push_str(&format!("status: {}\n", issue.status));
    out.push_str(&format!("priority: {}\n", issue.priority));

    if issue.labels.is_empty() {
        out.push_str("labels: []\n");
    } else {
        out.push_str("labels:\n");
        for l in &issue.labels {
            if let Some(ref c) = l.color {
                out.push_str(&format!(
                    "  - name: \"{}\"\n    color: \"{}\"\n",
                    escape_yaml_str(&l.name),
                    c
                ));
            } else {
                out.push_str(&format!("  - name: \"{}\"\n", escape_yaml_str(&l.name)));
            }
        }
    }

    if let Some(ref a) = issue.assignee {
        out.push_str(&format!("assignee: \"{}\"\n", escape_yaml_str(a)));
    } else {
        out.push_str("assignee: null\n");
    }
    if let Some(ref m) = issue.milestone {
        out.push_str(&format!("milestone: \"{}\"\n", escape_yaml_str(m)));
    }
    if let Some(p) = issue.parent {
        out.push_str(&format!("parent: {}\n", p));
    } else {
        out.push_str("parent: null\n");
    }
    if issue.dependencies.is_empty() {
        out.push_str("dependencies: []\n");
    } else {
        out.push_str(&format!("dependencies: {:?}\n", issue.dependencies));
    }
    if !issue.epic_children.is_empty() {
        out.push_str(&format!("epic_children: {:?}\n", issue.epic_children));
    }
    if let Some(pt) = issue.points {
        out.push_str(&format!("points: {}\n", pt));
    }
    if let Some(ref d) = issue.due {
        out.push_str(&format!("due: \"{}\"\n", d));
    }
    out.push_str(&format!("created_at: \"{}\"\n", issue.created_at));
    out.push_str(&format!("updated_at: \"{}\"\n", issue.updated_at));
    out.push_str("---\n\n");
    out.push_str(&issue.body);
    if !issue.body.is_empty() && !issue.body.ends_with('\n') {
        out.push('\n');
    }
    out
}

fn escape_yaml_str(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}
