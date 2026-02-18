//! Discovers existing agent markdown files in a project tree.
//!
//! Scans the project root for all well-known agent configuration file formats
//! (AGENTS.md, CLAUDE.md, .cursorrules, etc.) so `llmd init` can import them
//! into .llmd/imported/ automatically.

use std::path::{Path, PathBuf};

/// A discovered agent file — its source path and a short description of its format.
pub struct DiscoveredFile {
    pub path: PathBuf,
    pub format: &'static str,
}

/// File patterns that live at a fixed path relative to the project root.
const FIXED_PATHS: &[(&str, &str)] = &[
    ("AGENTS.md", "AGENTS.md — cross-tool agent instructions"),
    (
        "AGENTS.override.md",
        "AGENTS.override.md — OpenAI Codex overrides",
    ),
    ("CLAUDE.md", "CLAUDE.md — Claude Code configuration"),
    ("GEMINI.md", "GEMINI.md — Google Gemini configuration"),
    ("AGENT.md", "AGENT.md — Google Gemini (alternate name)"),
    ("JULES.md", "JULES.md — Google Jules configuration"),
    ("CONVENTIONS.md", "CONVENTIONS.md — general conventions"),
    ("SPEC.md", "SPEC.md — project specification"),
    ("PRD.md", "PRD.md — product requirements document"),
    ("Plan.md", "Plan.md — agent execution plan"),
    (".cursorrules", ".cursorrules — Cursor rules (legacy)"),
    (".windsurfrules", ".windsurfrules — Windsurf rules"),
    (".clinerules", ".clinerules — Cline rules"),
    (".builderrules", ".builderrules — Builder.io rules"),
    (
        ".github/copilot-instructions.md",
        ".github/copilot-instructions.md — GitHub Copilot instructions",
    ),
    ("llms.txt", "llms.txt — LLM-optimised documentation index"),
    (
        "llms-full.txt",
        "llms-full.txt — complete LLM documentation",
    ),
];

/// Discovers all known agent markdown files under `root`.
///
/// Returns one entry per file that exists on disk. Duplicate filenames (e.g. both
/// AGENTS.md and CLAUDE.md) are each returned as separate entries.
pub fn discover(root: &Path) -> Vec<DiscoveredFile> {
    let mut found = Vec::new();

    for (rel_path, format) in FIXED_PATHS {
        let abs = root.join(rel_path);
        if abs.exists() {
            found.push(DiscoveredFile { path: abs, format });
        }
    }

    // .cursor/rules/ — collect all .md and .mdc files
    let cursor_rules = root.join(".cursor/rules");
    if cursor_rules.is_dir()
        && let Ok(entries) = std::fs::read_dir(&cursor_rules)
    {
        for entry in entries.flatten() {
            let p = entry.path();
            let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext == "md" || ext == "mdc" {
                found.push(DiscoveredFile {
                    path: p,
                    format: ".cursor/rules/ — Cursor scoped rule",
                });
            }
        }
    }

    // .claude/rules/ — scoped Claude Code rules
    let claude_rules = root.join(".claude/rules");
    if claude_rules.is_dir()
        && let Ok(entries) = std::fs::read_dir(&claude_rules)
    {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) == Some("md") {
                found.push(DiscoveredFile {
                    path: p,
                    format: ".claude/rules/ — Claude Code scoped rule",
                });
            }
        }
    }

    // .github/instructions/*.instructions.md — Copilot path-specific instructions
    let gh_instructions = root.join(".github/instructions");
    if gh_instructions.is_dir()
        && let Ok(entries) = std::fs::read_dir(&gh_instructions)
    {
        for entry in entries.flatten() {
            let p = entry.path();
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name.ends_with(".instructions.md") {
                found.push(DiscoveredFile {
                    path: p,
                    format: ".github/instructions/ — Copilot path-specific instructions",
                });
            }
        }
    }

    found
}
