//! Dependency graph operations: ready tasks, cycle detection, epic tree.

use std::collections::{HashMap, HashSet, VecDeque};

use crate::issues::models::Issue;

/// Returns open issues that have no unresolved dependencies.
/// Excludes epics (they are containers, not actionable).
pub fn ready_tasks(issues: &HashMap<u32, Issue>, exclude_epics: bool) -> Vec<&Issue> {
    let closed: HashSet<u32> = issues
        .values()
        .filter(|i| i.status == "closed")
        .map(|i| i.id)
        .collect();

    let mut result: Vec<&Issue> = issues
        .values()
        .filter(|i| {
            i.status != "closed"
                && (!exclude_epics || i.issue_type != "epic")
                && i.dependencies.iter().all(|dep| closed.contains(dep))
        })
        .collect();

    result.sort_by(|a, b| {
        let pri = |p: &str| match p {
            "high" => 0,
            "medium" => 1,
            "low" => 2,
            _ => 1,
        };
        pri(&a.priority)
            .cmp(&pri(&b.priority))
            .then_with(|| a.id.cmp(&b.id))
    });
    result
}

/// Detects cycles in the dependency + parent graph.
#[allow(dead_code)]
pub fn has_cycle(issues: &HashMap<u32, Issue>) -> bool {
    let mut visited = HashSet::new();
    let mut stack = HashSet::new();

    for &id in issues.keys() {
        if !visited.contains(&id) && cycle_dfs(id, issues, &mut visited, &mut stack) {
            return true;
        }
    }
    false
}

#[allow(dead_code)]
fn cycle_dfs(
    id: u32,
    issues: &HashMap<u32, Issue>,
    visited: &mut HashSet<u32>,
    stack: &mut HashSet<u32>,
) -> bool {
    visited.insert(id);
    stack.insert(id);

    let deps: Vec<u32> = issues
        .get(&id)
        .map(|i| {
            let mut d = i.dependencies.clone();
            if let Some(p) = i.parent {
                d.push(p);
            }
            d
        })
        .unwrap_or_default();

    for dep in deps {
        if !visited.contains(&dep) {
            if cycle_dfs(dep, issues, visited, stack) {
                return true;
            }
        } else if stack.contains(&dep) {
            return true;
        }
    }

    stack.remove(&id);
    false
}

/// Returns the epic tree rooted at the given id (for `llmd issue tree`).
pub fn epic_tree(issues: &HashMap<u32, Issue>, root_id: u32) -> Vec<(u32, usize)> {
    let mut result = Vec::new();
    let mut queue = VecDeque::new();
    queue.push_back((root_id, 0usize));

    while let Some((id, depth)) = queue.pop_front() {
        result.push((id, depth));
        if let Some(issue) = issues.get(&id) {
            for &child_id in &issue.epic_children {
                queue.push_back((child_id, depth + 1));
            }
        }
    }
    result
}
