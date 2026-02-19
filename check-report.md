# llmd Pre-Release Check Report

**Generated:** 2026-02-19 03:51 UTC
**Version:** `0.2.0`
**Branch:** `main`
**Result:** 2 check(s) failed, 3 passed

---

## Check Summary

| Check | Result |
|-------|--------|
| `cargo build` | ✅ Passed |
| `cargo test` | ✅ Passed |
| `cargo clippy -D warn` | ❌ Failed |
| `cargo fmt --check` | ❌ Failed |
| `publish dry-run` | ✅ Passed |

---

## Git Warnings

These are not blocking failures but should be resolved before publishing:

- **Uncommitted changes** — commit everything before publishing.

```
 M .llmd/architecture.md
 M .llmd/catme.md
 M .llmd/cli.md
 M .llmd/conventions.md
 M Cargo.lock
 M Cargo.toml
 M README.md
 M src/commands/build.rs
 M src/commands/compose.rs
 M src/commands/mod.rs
 M src/llmd_dir.rs
 M src/main.rs
?? .llmd/context-mappings.json
?? .llmd/issues/
?? src/commands/index.rs
?? src/commands/issue/
?? src/issues/
?? task/issue_tracker.md
```

---

## Failed Checks — Details

Each section below contains the full compiler/tool output for a failed check.
Provide this report to an agent to resolve the issues.

### `cargo clippy -D warn`

```
    Checking llmd v0.2.0 (/home/simulacralabs/projects/ai/llmd)
error: unused import: `serialize_issue`
  --> src/issues/mod.rs:11:36
   |
11 | pub use frontmatter::{parse_issue, serialize_issue};
   |                                    ^^^^^^^^^^^^^^^
   |
   = note: `-D unused-imports` implied by `-D warnings`
   = help: to override `-D warnings` add `#[allow(unused_imports)]`

error: unused import: `has_cycle`
  --> src/issues/mod.rs:12:28
   |
12 | pub use graph::{epic_tree, has_cycle, ready_tasks};
   |                            ^^^^^^^^^

error: unused imports: `Comment` and `Label`
  --> src/issues/mod.rs:13:18
   |
13 | pub use models::{Comment, Config, Issue, Label};
   |                  ^^^^^^^                 ^^^^^

error: function `has_cycle` is never used
  --> src/issues/graph.rs:38:8
   |
38 | pub fn has_cycle(issues: &HashMap<u32, Issue>) -> bool {
   |        ^^^^^^^^^
   |
   = note: `-D dead-code` implied by `-D warnings`
   = help: to override `-D warnings` add `#[allow(dead_code)]`

error: function `cycle_dfs` is never used
  --> src/issues/graph.rs:50:4
   |
50 | fn cycle_dfs(
   |    ^^^^^^^^^

error: struct `Comment` is never constructed
  --> src/issues/models.rs:27:12
   |
27 | pub struct Comment {
   |            ^^^^^^^

error: function `issues_path` is never used
  --> src/llmd_dir.rs:34:8
   |
34 | pub fn issues_path(llmd: &Path) -> PathBuf {
   |        ^^^^^^^^^^^

error: calling `push_str()` using a single-character string literal
   --> src/commands/build.rs:188:13
    |
188 |             md.push_str("\n");
    |             ^^^^^^^^^^^^^^^^^ help: consider using `push` with a character literal: `md.push('\n')`
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.93.0/index.html#single_char_add_str
    = note: `-D clippy::single-char-add-str` implied by `-D warnings`
    = help: to override `-D warnings` add `#[allow(clippy::single_char_add_str)]`

error: calling `push_str()` using a single-character string literal
   --> src/commands/build.rs:202:13
    |
202 |             md.push_str("\n");
    |             ^^^^^^^^^^^^^^^^^ help: consider using `push` with a character literal: `md.push('\n')`
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.93.0/index.html#single_char_add_str

error: this `if` statement can be collapsed
   --> src/commands/compose.rs:259:9
    |
259 | /         if let Ok(id) = id_or_slug.parse::<u32>() {
260 | |             if stem.starts_with(&format!("{id:03}-")) {
261 | |                 return Some(path);
262 | |             }
263 | |         }
    | |_________^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.93.0/index.html#collapsible_if
    = note: `-D clippy::collapsible-if` implied by `-D warnings`
    = help: to override `-D warnings` add `#[allow(clippy::collapsible_if)]`
help: collapse nested if block
    |
259 ~         if let Ok(id) = id_or_slug.parse::<u32>()
260 ~             && stem.starts_with(&format!("{id:03}-")) {
261 |                 return Some(path);
262 ~             }
    |

error: stripping a prefix manually
   --> src/commands/compose.rs:328:24
    |
328 |             let rest = line["labels:".len()..].trim();
    |                        ^^^^^^^^^^^^^^^^^^^^^^^
    |
note: the prefix was tested here
   --> src/commands/compose.rs:327:9
    |
327 |         if line.starts_with("labels:") {
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.93.0/index.html#manual_strip
    = note: `-D clippy::manual-strip` implied by `-D warnings`
    = help: to override `-D warnings` add `#[allow(clippy::manual_strip)]`
help: try using the `strip_prefix` method
    |
327 ~         if let Some(<stripped>) = line.strip_prefix("labels:") {
328 ~             let rest = <stripped>.trim();
    |

error: this `if` statement can be collapsed
   --> src/commands/compose.rs:345:20
    |
345 |               } else if line.contains("name:") {
    |  ____________________^
346 | |                 if let Some(after) = line.split("name:").nth(1) {
347 | |                     let name = after.trim().split_whitespace().next().unwrap_or("").trim_matches('"').trim_matches('\'').to_string();
348 | |                     if !name.is_empty() {
...   |
352 | |             }
    | |_____________^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.93.0/index.html#collapsible_if
help: collapse nested if block
    |
345 ~             } else if line.contains("name:")
346 ~                 && let Some(after) = line.split("name:").nth(1) {
347 |                     let name = after.trim().split_whitespace().next().unwrap_or("").trim_matches('"').trim_matches('\'').to_string();
...
350 |                     }
351 ~                 }
    |

error: found call to `str::trim` before `str::split_whitespace`
   --> src/commands/compose.rs:347:38
    |
347 |                     let name = after.trim().split_whitespace().next().unwrap_or("").trim_matches('"').trim_matches('\'').to_string();
    |                                      ^^^^^^^ help: remove `trim()`
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.93.0/index.html#trim_split_whitespace
    = note: `-D clippy::trim-split-whitespace` implied by `-D warnings`
    = help: to override `-D warnings` add `#[allow(clippy::trim_split_whitespace)]`

error: this `map_or` can be simplified
  --> src/commands/issue/ready.rs:39:13
   |
39 |             args.type_filter.as_ref().map_or(true, |t| i.issue_type == *t)
   |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.93.0/index.html#unnecessary_map_or
   = note: `-D clippy::unnecessary-map-or` implied by `-D warnings`
   = help: to override `-D warnings` add `#[allow(clippy::unnecessary_map_or)]`
help: use is_none_or instead
   |
39 -             args.type_filter.as_ref().map_or(true, |t| i.issue_type == *t)
39 +             args.type_filter.as_ref().is_none_or(|t| i.issue_type == *t)
   |

error: this `map_or` can be simplified
  --> src/commands/issue/ready.rs:40:20
   |
40 |                 && args.milestone.as_ref().map_or(true, |m| i.milestone.as_deref() == Some(m.as_str()))
   |                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.93.0/index.html#unnecessary_map_or
help: use is_none_or instead
   |
40 -                 && args.milestone.as_ref().map_or(true, |m| i.milestone.as_deref() == Some(m.as_str()))
40 +                 && args.milestone.as_ref().is_none_or(|m| i.milestone.as_deref() == Some(m.as_str()))
   |

error: this `map_or` can be simplified
  --> src/commands/issue/ready.rs:41:20
   |
41 |                 && args.assignee.as_ref().map_or(true, |a| i.assignee.as_deref() == Some(a.as_str()))
   |                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.93.0/index.html#unnecessary_map_or
help: use is_none_or instead
   |
41 -                 && args.assignee.as_ref().map_or(true, |a| i.assignee.as_deref() == Some(a.as_str()))
41 +                 && args.assignee.as_ref().is_none_or(|a| i.assignee.as_deref() == Some(a.as_str()))
   |

error: this `if` statement can be collapsed
  --> src/commands/issue/update.rs:92:5
   |
92 | /     if let Some(dep) = args.add_dep {
93 | |         if !issue.dependencies.contains(&dep) {
94 | |             issue.dependencies.push(dep);
95 | |             issue.dependencies.sort();
96 | |         }
97 | |     }
   | |_____^
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.93.0/index.html#collapsible_if
help: collapse nested if block
   |
92 ~     if let Some(dep) = args.add_dep
93 ~         && !issue.dependencies.contains(&dep) {
94 |             issue.dependencies.push(dep);
95 |             issue.dependencies.sort();
96 ~         }
   |

error: this `if` statement can be collapsed
  --> src/issues/file_ops.rs:74:9
   |
74 | /         if let Ok(id) = id_or_slug.parse::<u32>() {
75 | |             if stem.starts_with(&format!("{id:03}-")) {
76 | |                 return Some(path);
77 | |             }
78 | |         }
   | |_________^
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.93.0/index.html#collapsible_if
help: collapse nested if block
   |
74 ~         if let Ok(id) = id_or_slug.parse::<u32>()
75 ~             && stem.starts_with(&format!("{id:03}-")) {
76 |                 return Some(path);
77 ~             }
   |

error: this expression creates a reference which is immediately dereferenced by the compiler
  --> src/issues/frontmatter.rs:29:31
   |
29 |     let labels = parse_labels(&fm);
   |                               ^^^ help: change this to: `fm`
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.93.0/index.html#needless_borrow
   = note: `-D clippy::needless-borrow` implied by `-D warnings`
   = help: to override `-D warnings` add `#[allow(clippy::needless_borrow)]`

error: this expression creates a reference which is immediately dereferenced by the compiler
  --> src/issues/frontmatter.rs:30:42
   |
30 |     let dependencies = parse_array_field(&fm, "dependencies");
   |                                          ^^^ help: change this to: `fm`
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.93.0/index.html#needless_borrow

error: this expression creates a reference which is immediately dereferenced by the compiler
  --> src/issues/frontmatter.rs:31:43
   |
31 |     let epic_children = parse_array_field(&fm, "epic_children");
   |                                           ^^^ help: change this to: `fm`
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.93.0/index.html#needless_borrow

error: could not compile `llmd` (bin "llmd") due to 21 previous errors
```

### `cargo fmt --check`

```
Diff in /home/simulacralabs/projects/ai/llmd/src/commands/build.rs:142:
     let now = issues::now_iso();
     let date = now.split('T').next().unwrap_or("");
 
-    let mut md = format!(
-        "# Roadmap\n\nGenerated from .llmd/issues/ — {date}\n\n"
-    );
+    let mut md = format!("# Roadmap\n\nGenerated from .llmd/issues/ — {date}\n\n");
 
     let mut by_milestone: std::collections::BTreeMap<String, Vec<&issues::Issue>> =
         std::collections::BTreeMap::new();
Diff in /home/simulacralabs/projects/ai/llmd/src/commands/build.rs:151:
     for issue in issues_map.values() {
-        let m = issue.milestone.clone().unwrap_or_else(|| "_no_milestone".to_string());
+        let m = issue
+            .milestone
+            .clone()
+            .unwrap_or_else(|| "_no_milestone".to_string());
         by_milestone.entry(m).or_default().push(issue);
     }
 
Diff in /home/simulacralabs/projects/ai/llmd/src/commands/build.rs:170:
             for i in epics {
                 let pts = i.points.map(|p| format!("{}pts", p)).unwrap_or_default();
                 let due = i.due.as_deref().unwrap_or("");
-                let meta = [&pts as &str, due].iter().filter(|s| !s.is_empty()).cloned().collect::<Vec<_>>().join(" · ");
+                let meta = [&pts as &str, due]
+                    .iter()
+                    .filter(|s| !s.is_empty())
+                    .cloned()
+                    .collect::<Vec<_>>()
+                    .join(" · ");
                 md.push_str(&format!(
                     "- **[#{} {}]({:03}-{}.md)** `epic` {}\n",
                     i.id, i.title, i.id, i.slug, meta
Diff in /home/simulacralabs/projects/ai/llmd/src/commands/build.rs:209:
     let bar_len = 10;
     let filled = (pct * bar_len) / 100;
     let bar: String = "█".repeat(filled) + &"░".repeat(bar_len - filled);
-    md.push_str(&format!("### Progress\n\n{}/{} closed ({}%)\n\n{}", closed, total, pct, bar));
+    md.push_str(&format!(
+        "### Progress\n\n{}/{} closed ({}%)\n\n{}",
+        closed, total, pct, bar
+    ));
     Ok(md)
 }
 
Diff in /home/simulacralabs/projects/ai/llmd/src/commands/compose.rs:134:
     let mut index = Vec::new();
 
     for file_path in all_files {
-        if *file_path == catme
-            || file_path.starts_with(&imported)
-            || file_path.starts_with(&issues)
+        if *file_path == catme || file_path.starts_with(&imported) || file_path.starts_with(&issues)
         {
             continue;
         }
Diff in /home/simulacralabs/projects/ai/llmd/src/commands/compose.rs:185:
         } else if n >= 1 && n <= index.len() {
             // duplicate, skip
         } else {
-            anyhow::bail!("Section index {n} is out of range (1–{}). Run `llmd index` to see available sections.", index.len());
+            anyhow::bail!(
+                "Section index {n} is out of range (1–{}). Run `llmd index` to see available sections.",
+                index.len()
+            );
         }
     }
     Ok(result)
Diff in /home/simulacralabs/projects/ai/llmd/src/commands/compose.rs:315:
 
 /// Extracts label names from YAML frontmatter. Supports both `labels: [a, b]` and `labels:\n  - name: a`.
 fn extract_labels_from_frontmatter(content: &str) -> Vec<String> {
-    let Some(fm) = content.strip_prefix("---\n").and_then(|s| s.split("\n---").next()) else {
+    let Some(fm) = content
+        .strip_prefix("---\n")
+        .and_then(|s| s.split("\n---").next())
+    else {
         return Vec::new();
     };
 
Diff in /home/simulacralabs/projects/ai/llmd/src/commands/compose.rs:344:
                 in_labels = false;
             } else if line.contains("name:") {
                 if let Some(after) = line.split("name:").nth(1) {
-                    let name = after.trim().split_whitespace().next().unwrap_or("").trim_matches('"').trim_matches('\'').to_string();
+                    let name = after
+                        .trim()
+                        .split_whitespace()
+                        .next()
+                        .unwrap_or("")
+                        .trim_matches('"')
+                        .trim_matches('\'')
+                        .to_string();
                     if !name.is_empty() {
                         labels.push(name);
                     }
Diff in /home/simulacralabs/projects/ai/llmd/src/commands/compose.rs:457:
         _ => catme.lines().take(40).collect::<Vec<_>>().join("\n") + "\n",
     }
 }
-
 
Diff in /home/simulacralabs/projects/ai/llmd/src/commands/issue/list.rs:3:
 use anyhow::Result;
 use clap::Parser;
 
-use crate::issues::{load_all_issues};
+use crate::issues::load_all_issues;
 use crate::llmd_dir;
 
 #[derive(Parser)]
Diff in /home/simulacralabs/projects/ai/llmd/src/commands/issue/list.rs:79:
         println!("{}", serde_json::to_string_pretty(&out)?);
     } else {
         for i in list {
-            let labels: String = i.labels.iter().map(|l| l.name.as_str()).collect::<Vec<_>>().join(", ");
+            let labels: String = i
+                .labels
+                .iter()
+                .map(|l| l.name.as_str())
+                .collect::<Vec<_>>()
+                .join(", ");
             let assignee = i.assignee.as_deref().unwrap_or("—");
             println!(
                 "#{} {} [{}] {} · {} · {}",
Diff in /home/simulacralabs/projects/ai/llmd/src/commands/issue/mentions.rs:27:
         .map(|h| format!("@{}", h))
         .unwrap_or_else(|| "@\\w+".to_string());
 
-    let re = regex_lite::Regex::new(&pattern).unwrap_or_else(|_| regex_lite::Regex::new("@\\w+").unwrap());
+    let re = regex_lite::Regex::new(&pattern)
+        .unwrap_or_else(|_| regex_lite::Regex::new("@\\w+").unwrap());
 
     let entries = fs::read_dir(&issues_dir)?;
     for entry in entries.flatten() {
Diff in /home/simulacralabs/projects/ai/llmd/src/commands/issue/mentions.rs:39:
             Ok(c) => c,
             Err(_) => continue,
         };
-        let comments_section = content
-            .split("## Comments")
-            .nth(1)
-            .unwrap_or("");
+        let comments_section = content.split("## Comments").nth(1).unwrap_or("");
         let yaml_block = comments_section
             .split("```yaml")
             .nth(1)
Diff in /home/simulacralabs/projects/ai/llmd/src/commands/issue/mentions.rs:70:
                     .unwrap_or("?");
                 println!(
                     "#{} {} — {} ({}): {}",
-                    id, stem, author.trim_matches('"'), date.trim_matches('"'), body.trim_matches('"')
+                    id,
+                    stem,
+                    author.trim_matches('"'),
+                    date.trim_matches('"'),
+                    body.trim_matches('"')
                 );
                 break;
             }
Diff in /home/simulacralabs/projects/ai/llmd/src/commands/issue/new.rs:1:
 //! `llmd issue new`
 
-use anyhow::{Context, Result};
-use clap::Parser;
 use crate::issues::models::{Issue, Label};
 use crate::issues::{file_ops, load_config, save_config, write_issue};
 use crate::llmd_dir;
Diff in /home/simulacralabs/projects/ai/llmd/src/commands/issue/new.rs:8:
+use anyhow::{Context, Result};
+use clap::Parser;
 
 fn slugify(s: &str) -> String {
     s.to_lowercase()
Diff in /home/simulacralabs/projects/ai/llmd/src/commands/issue/new.rs:11:
         .chars()
-        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { ' ' })
+        .map(|c| {
+            if c.is_alphanumeric() || c == '-' || c == '_' {
+                c
+            } else {
+                ' '
+            }
+        })
         .collect::<String>()
         .split_whitespace()
         .collect::<Vec<_>>()
Diff in /home/simulacralabs/projects/ai/llmd/src/commands/issue/ready.rs:36:
     let filtered: Vec<_> = ready
         .iter()
         .filter(|i| {
-            args.type_filter.as_ref().map_or(true, |t| i.issue_type == *t)
-                && args.milestone.as_ref().map_or(true, |m| i.milestone.as_deref() == Some(m.as_str()))
-                && args.assignee.as_ref().map_or(true, |a| i.assignee.as_deref() == Some(a.as_str()))
+            args.type_filter
+                .as_ref()
+                .map_or(true, |t| i.issue_type == *t)
+                && args
+                    .milestone
+                    .as_ref()
+                    .map_or(true, |m| i.milestone.as_deref() == Some(m.as_str()))
+                && args
+                    .assignee
+                    .as_ref()
+                    .map_or(true, |a| i.assignee.as_deref() == Some(a.as_str()))
         })
         .collect();
 
Diff in /home/simulacralabs/projects/ai/llmd/src/commands/issue/show.rs:27:
     let path = resolve_issue_path(&issues_dir, &args.id_or_slug)
         .with_context(|| format!("Issue \"{}\" not found", args.id_or_slug))?;
 
-    let content = fs::read_to_string(&path)
-        .with_context(|| format!("Cannot read {}", path.display()))?;
+    let content =
+        fs::read_to_string(&path).with_context(|| format!("Cannot read {}", path.display()))?;
 
     if args.json {
         let issues = load_all_issues(&issues_dir)?;
Diff in /home/simulacralabs/projects/ai/llmd/src/commands/issue/update.rs:59:
     let path = resolve_issue_path(&issues_dir, &args.id_or_slug)
         .with_context(|| format!("Issue \"{}\" not found", args.id_or_slug))?;
 
-    let content = fs::read_to_string(&path)
-        .with_context(|| format!("Cannot read {}", path.display()))?;
+    let content =
+        fs::read_to_string(&path).with_context(|| format!("Cannot read {}", path.display()))?;
 
     let id: u32 = path
         .file_stem()
Diff in /home/simulacralabs/projects/ai/llmd/src/issues/file_ops.rs:23:
 /// Loads config from .llmd/issues/config.json.
 pub fn load_config(issues_dir: &Path) -> Result<Config> {
     let path = config_path(issues_dir);
-    let content = fs::read_to_string(&path)
-        .with_context(|| format!("Cannot read {}. Run `llmd issue init` first.", path.display()))?;
+    let content = fs::read_to_string(&path).with_context(|| {
+        format!(
+            "Cannot read {}. Run `llmd issue init` first.",
+            path.display()
+        )
+    })?;
     serde_json::from_str(&content).context("Invalid config.json")
 }
 
Diff in /home/simulacralabs/projects/ai/llmd/src/issues/file_ops.rs:53:
         let Ok(id) = id_str.parse::<u32>() else {
             continue;
         };
-        let content = fs::read_to_string(&path)
-            .with_context(|| format!("Cannot read {}", path.display()))?;
+        let content =
+            fs::read_to_string(&path).with_context(|| format!("Cannot read {}", path.display()))?;
         if let Some(issue) = parse_issue(&content, id) {
             map.insert(id, issue);
         }
Diff in /home/simulacralabs/projects/ai/llmd/src/issues/frontmatter.rs:11:
     let (fm, body) = split_frontmatter(content)?;
     let mut map = parse_frontmatter_map(fm);
 
-    let title = map.remove("title").unwrap_or_else(|| "Untitled".to_string());
-    let slug = map
-        .remove("slug")
-        .unwrap_or_else(|| slugify(&title));
+    let title = map
+        .remove("title")
+        .unwrap_or_else(|| "Untitled".to_string());
+    let slug = map.remove("slug").unwrap_or_else(|| slugify(&title));
     let issue_type = map.remove("type").unwrap_or_else(|| "task".to_string());
     let status = map.remove("status").unwrap_or_else(|| "open".to_string());
-    let priority = map.remove("priority").unwrap_or_else(|| "medium".to_string());
-    let assignee = map.remove("assignee").filter(|s| !s.is_empty() && s != "null");
+    let priority = map
+        .remove("priority")
+        .unwrap_or_else(|| "medium".to_string());
+    let assignee = map
+        .remove("assignee")
+        .filter(|s| !s.is_empty() && s != "null");
     let milestone = map.remove("milestone").filter(|s| !s.is_empty());
     let parent = map.remove("parent").and_then(|s| s.parse().ok());
     let created_at = map.remove("created_at").unwrap_or_default();
Diff in /home/simulacralabs/projects/ai/llmd/src/issues/frontmatter.rs:25:
-    let updated_at = map.remove("updated_at").unwrap_or_else(|| created_at.clone());
+    let updated_at = map
+        .remove("updated_at")
+        .unwrap_or_else(|| created_at.clone());
     let due = map.remove("due").filter(|s| !s.is_empty());
     let points = map.remove("points").and_then(|s| s.parse().ok());
 
Diff in /home/simulacralabs/projects/ai/llmd/src/issues/frontmatter.rs:70:
         }
         if line.starts_with(|c: char| c.is_alphabetic() || c == '_') {
             if !key.is_empty() {
-                map.insert(key.clone(), value.trim().trim_matches('"').trim_matches('\'').to_string());
+                map.insert(
+                    key.clone(),
+                    value
+                        .trim()
+                        .trim_matches('"')
+                        .trim_matches('\'')
+                        .to_string(),
+                );
             }
             if let Some((k, v)) = line.split_once(':') {
                 key = k.trim().to_string();
Diff in /home/simulacralabs/projects/ai/llmd/src/issues/frontmatter.rs:86:
         }
     }
     if !key.is_empty() {
-        map.insert(key, value.trim().trim_matches('"').trim_matches('\'').to_string());
+        map.insert(
+            key,
+            value
+                .trim()
+                .trim_matches('"')
+                .trim_matches('\'')
+                .to_string(),
+        );
     }
     map
 }
Diff in /home/simulacralabs/projects/ai/llmd/src/issues/frontmatter.rs:174:
 fn slugify(s: &str) -> String {
     s.to_lowercase()
         .chars()
-        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { ' ' })
+        .map(|c| {
+            if c.is_alphanumeric() || c == '-' || c == '_' {
+                c
+            } else {
+                ' '
+            }
+        })
         .collect::<String>()
         .split_whitespace()
         .collect::<Vec<_>>()
Diff in /home/simulacralabs/projects/ai/llmd/src/issues/frontmatter.rs:197:
         out.push_str("labels:\n");
         for l in &issue.labels {
             if let Some(ref c) = l.color {
-                out.push_str(&format!("  - name: \"{}\"\n    color: \"{}\"\n", escape_yaml_str(&l.name), c));
+                out.push_str(&format!(
+                    "  - name: \"{}\"\n    color: \"{}\"\n",
+                    escape_yaml_str(&l.name),
+                    c
+                ));
             } else {
                 out.push_str(&format!("  - name: \"{}\"\n", escape_yaml_str(&l.name)));
             }
Diff in /home/simulacralabs/projects/ai/llmd/src/issues/graph.rs:29:
             "low" => 2,
             _ => 1,
         };
-        pri(&a.priority).cmp(&pri(&b.priority)).then_with(|| a.id.cmp(&b.id))
+        pri(&a.priority)
+            .cmp(&pri(&b.priority))
+            .then_with(|| a.id.cmp(&b.id))
     });
     result
 }
```

---

## Resolution

1. Fix each issue listed above.
2. Re-run `./scripts/check.sh` to verify.
3. Once all checks pass, run `./scripts/release.sh` to publish.
