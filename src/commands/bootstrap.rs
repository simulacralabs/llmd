//! `llmd bootstrap` — print a prompt that fills in the .llmd/ directory.
//!
//! After running `llmd init`, the .llmd/ files exist but are empty templates.
//! `llmd bootstrap` prints a comprehensive prompt to stdout that instructs an
//! LLM to analyse the codebase and write the actual content.
//!
//! The prompt is designed to be piped directly to an agentic CLI:
//!
//!   llmd bootstrap | claude
//!   llmd bootstrap | codex
//!   llmd bootstrap > bootstrap-prompt.md   # review before running

use anyhow::{Context, Result};
use clap::Parser;
use std::fs;

use crate::llmd_dir;

#[derive(Parser)]
pub struct BootstrapArgs {
    /// Also include the current catme.md content in the prompt so the agent
    /// can see what has already been filled in (useful for partial bootstraps)
    #[arg(long)]
    pub show_existing: bool,
}

pub fn run(args: BootstrapArgs) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let llmd = llmd_dir::find(&cwd)?;

    let project_root = llmd
        .parent()
        .context("Cannot determine project root from .llmd/ path")?;

    let project_name = project_root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("this project");

    let llmd_rel = llmd
        .strip_prefix(project_root)
        .unwrap_or(&llmd)
        .display()
        .to_string();

    let existing_catme = if args.show_existing {
        let catme_path = llmd_dir::catme_path(&llmd);
        if catme_path.exists() {
            let content =
                fs::read_to_string(&catme_path).context("Cannot read existing catme.md")?;
            format!(
                "\n## Current catme.md\n\nThis is the current state of catme.md. \
                 Fill in every placeholder and correct anything that is wrong or missing.\n\n\
                 ```markdown\n{content}\n```\n"
            )
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let prompt = build_prompt(project_name, &llmd_rel, &existing_catme);
    print!("{prompt}");
    Ok(())
}

fn build_prompt(project_name: &str, llmd_rel: &str, existing_catme: &str) -> String {
    format!(
        r#"# Bootstrap task: generate {llmd_rel}/ documentation for `{project_name}`

You are about to fully populate the `{llmd_rel}/` directory for this project.
This directory is a persistent, machine-readable knowledge base used by AI agents
(via the `llmd` CLI) to understand the codebase without re-scanning it every session.

Your goal: analyse this codebase thoroughly, then write every file described below.
Do not summarise or skip sections. Agents will rely on this content to plan and
implement tasks with zero additional codebase scanning.

---

## What you must do

1. **Read the entire codebase.** Start with any existing README, Cargo.toml / package.json /
   pyproject.toml / go.mod (or equivalent), then read every source file. Understand the
   architecture, patterns, and conventions before writing anything.

2. **Write `{llmd_rel}/catme.md`** — the agent entry point. Every agent reads this first.

3. **Write one topic file per major concern** — e.g. `{llmd_rel}/architecture.md`,
   `{llmd_rel}/api-standards.md`, `{llmd_rel}/data-models.md`, etc. Choose the topics
   that best reflect this codebase. Each file covers exactly one concern.

4. **Do not leave any placeholder comments.** Every section you write must contain real,
   specific information about this codebase. Vague or generic content is worse than nothing —
   it will cause agents to make incorrect assumptions.

---

## File specifications

### `{llmd_rel}/catme.md` — required

The entry point. Agents read this first, then follow links to topic files.

```
# <project name>

> **Agent entry point.** Read this file first to orient yourself in the project.

## Project Summary

One paragraph. What does this project do and why does it exist?
Be specific about the domain problem it solves.

## Technology Stack

List every significant language, framework, and library with its version where
relevant. Include the build system and package manager.

## Build & Test

Exact shell commands. No approximations — an agent must be able to copy-paste these.

```sh
# build
<command>

# run tests
<command>

# lint / typecheck
<command>

# run locally (if applicable)
<command>
```

## Navigation

Brief index of every topic file in {llmd_rel}/, one line each:

- [<filename>.md](<filename>.md) — <one sentence describing what it covers>

## Rules of Engagement

Specific, imperative rules for agents working in this codebase. Examples:
- "Never edit generated files in X directory — they are overwritten by Y"
- "All public API types must implement Trait Z"
- "Error handling uses the anyhow crate — do not introduce new error types"
- "Tests live alongside source files, not in a separate tests/ directory"

Be exhaustive. These rules are what prevent agents from making systematic mistakes.

## Context Map

Maps source directories and key files to the topic doc that covers them:

- `src/foo/` → [foo.md](foo.md)
- `src/bar.rs` → [architecture.md](architecture.md)
```

### Topic files — write as many as the codebase warrants

Each topic file uses this structure:

```
# <Topic>

## <Section>

<Specific, factual content about this codebase. No placeholders.>

## <Section>

...
```

**Required topics** (write these for every project):

- **`architecture.md`** — how the system is structured end-to-end: entry points, major
  modules, data flow, request/response lifecycle (or equivalent). Include a text diagram
  if helpful. Explain *why* the architecture is shaped this way if there are non-obvious
  reasons.

- **`conventions.md`** — code style and patterns used consistently in this codebase:
  naming conventions, file organisation, how errors are handled, how tests are written,
  how configuration is managed, patterns to follow and anti-patterns to avoid.

**Write additional topic files for any of the following that exist in this project:**

| Topic | Filename | When to write it |
|-------|----------|-----------------|
| API / HTTP endpoints | `api.md` | If the project exposes an HTTP or RPC API |
| Data models / schema | `data-models.md` | If there are significant data structures, DB schema, or domain types |
| Authentication & authorisation | `auth.md` | If the project handles auth |
| Database access patterns | `database.md` | If the project uses a database |
| CLI commands | `cli.md` | If the project is a CLI tool |
| Configuration | `configuration.md` | If there is significant runtime configuration |
| Deployment / infrastructure | `deployment.md` | If there is deployment config, Docker, CI/CD, etc. |
| External integrations | `integrations.md` | If the project calls external services or APIs |
| Domain concepts | `domain.md` | If there is significant domain logic that needs explanation |
| Testing strategy | `testing.md` | If the project has a non-obvious test setup |

You may create additional topic files beyond this list if the codebase has significant
concerns not covered above.

---

## Content standards

Write every section as if the reader is an experienced engineer who has never seen this
codebase. Do not assume they know the project history.

**Be specific:**
- Name actual files, functions, structs, modules, and types — not just concepts.
- Give real examples from the code, not hypotheticals.
- Quote exact error messages, configuration keys, and CLI flags where relevant.

**Be complete:**
- Every H2 section in every file must contain real content.
- If a section would be empty, omit that section rather than writing a placeholder.

**Use imperative language for agent instructions:**
- "Use `X` pattern for Y" not "X pattern is used for Y"
- "Add new routes to `router.rs`" not "Routes are in `router.rs`"

**Keep sections self-contained:**
- Each H2 section should make sense when read in isolation.
- Agents extract individual sections — context from adjacent sections is not guaranteed.
{existing_catme}
---

Begin by reading the codebase now, then write each file described above.
Output each file as a complete markdown code block labelled with its path, e.g.:

```markdown:{llmd_rel}/catme.md
# project-name
...
```

Write all files before stopping. Do not ask for clarification — make the best decisions
you can based on what you find in the codebase.
"#
    )
}
