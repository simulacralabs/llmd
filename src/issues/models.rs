//! Data structures for the issue tracker.

use serde::{Deserialize, Serialize};

/// Issue tracker config stored in .llmd/issues/config.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub next_id: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self { next_id: 1 }
    }
}

/// A label on an issue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Label {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

/// A comment on an issue.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub author: String,
    pub date: String,
    pub body: String,
}

/// Parsed issue from frontmatter + body.
#[derive(Debug, Clone)]
pub struct Issue {
    pub id: u32,
    pub title: String,
    pub slug: String,
    pub issue_type: String,
    pub status: String,
    pub priority: String,
    pub labels: Vec<Label>,
    pub assignee: Option<String>,
    pub milestone: Option<String>,
    pub parent: Option<u32>,
    pub dependencies: Vec<u32>,
    pub epic_children: Vec<u32>,
    pub points: Option<u32>,
    pub due: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub body: String,
}
