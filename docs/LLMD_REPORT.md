# llmd: Technical Report and Future Directions

**Version:** 0.2.0  
**Date:** 2026-02-19  
**Purpose:** A detailed technical report on llmd's current functionality, implementation, possible extensions, and architectural recommendations for a hypothetical rebuild.

---

## Executive Summary

`llmd` is a context management CLI for agentic development. It maintains a `.llmd/` directory — a persistent, machine-readable knowledge base of structured markdown files that AI agents read on demand. The tool addresses a core problem: agents have finite context windows, and injecting an entire codebase every session is wasteful and often impossible. Instead, agents use `llmd` to orient (`read catme`), assemble task-specific context (`compose`), search for patterns (`search`), and track work (`issue`). The design prioritises composability (stdout for data, stderr for diagnostics), minimal dependencies, and context efficiency.

---

## 1. Current Functionality

### 1.1 Core Commands

| Command | Purpose |
|---------|---------|
| `llmd init` | Creates `.llmd/` and imports existing agent config files (AGENTS.md, .cursorrules, etc.) into `.llmd/imported/` |
| `llmd bootstrap` | Prints a prompt to stdout that instructs an LLM to analyse the codebase and populate `.llmd/` with catme.md and topic files |
| `llmd read <file>` | Reads a file from `.llmd/` with optional `--section`, `--grep`, `--lines`, `--tokens` |
| `llmd index` | Prints a numbered section index (H2/H3 headings from topic files) for use with `llmd compose` |
| `llmd compose` | Assembles a task-context document from selected sections, optional task/issue, and label-to-topics mapping |
| `llmd search <query>` | Full-text regex search across `.llmd/` with configurable context lines |
| `llmd build` | Generates a static mdbook site from `.llmd/` (requires mdbook) |
| `llmd serve` | Builds and serves the mdbook locally with live reload |

### 1.2 Issue Tracker (`llmd issue`)

| Subcommand | Purpose |
|------------|---------|
| `init` | Creates `.llmd/issues/` and `config.json` |
| `new <title>` | Creates an issue with `--type`, `--priority`, `--labels`, `--assignee`, `--parent`, `--milestone`, `--points`, `--due`, `--dep` |
| `list` | Lists issues with `--json`, `--status`, `--type`, `--milestone`, `--assignee`, `--epic` |
| `show <id\|slug>` | Shows full issue or `--json` (frontmatter only) |
| `update <id\|slug>` | Updates fields; supports `--add-comment` with `--author` |
| `ready` | Lists open issues with no unresolved dependencies |
| `tree <id>` | Prints epic hierarchy |
| `mentions [@handle]` | Lists issues whose comments contain @mentions |

### 1.3 Key Features

- **Two-step compose flow:** `llmd index` → caller selects sections → `llmd compose --sections 1,2,5`
- **Issue-based compose:** `llmd compose --issue 3` auto-includes topics from `.llmd/context-mappings.json` (label → topics) unless `--no-auto-include`
- **Roadmap in serve/build:** The roadmap page is generated and injected into the mdbook; `llmd serve` always shows an up-to-date roadmap
- **Epic hierarchy:** Issues can have `parent` and `epic_children`; `llmd issue tree` renders the hierarchy
- **Structured comments:** Comments live in a `## Comments` section as a fenced YAML block; `@mentions` are extractable
- **Label colours:** Labels support optional `color`; rendered as coloured badges in the roadmap HTML

---

## 2. Implementation Overview

### 2.1 Architecture

```
src/
  main.rs              # CLI entry, Command enum, dispatch
  commands/
    mod.rs             # Re-exports all command modules
    init.rs            # Creates .llmd/, imports agent files, generates catme
    bootstrap.rs       # Builds and prints bootstrap prompt
    read.rs           # File/section/grep/line-range reading
    index.rs          # Delegates to compose::print_section_index
    compose.rs        # Section index, document assembly, issue context, label mapping
    search.rs         # Regex search with context
    build.rs          # mdbook generation, roadmap page, copy to output
    serve.rs          # Delegates to build::generate_mdbook, runs mdbook serve
    issue/
      mod.rs          # Issue subcommand dispatch
      init.rs, new.rs, list.rs, show.rs, update.rs, ready.rs, tree.rs, mentions.rs
  issues/
    mod.rs            # Re-exports
    models.rs         # Config, Issue, Label, Comment
    frontmatter.rs    # Hand-rolled YAML frontmatter parser/serialiser (no serde_yaml)
    graph.rs          # ready_tasks, epic_tree, has_cycle (cycle detection)
    file_ops.rs       # load_config, save_config, load_all_issues, resolve_issue_path, write_issue
  llmd_dir.rs         # find(), catme_path(), issues_path(), list_all_files()
  markdown.rs         # extract_section, list_headings, estimate_tokens, window
  discovery.rs        # Discovers AGENTS.md, .cursorrules, etc.
```

### 2.2 Design Patterns

- **Single responsibility:** Each command in its own file; `main.rs` stays flat
- **Path resolution:** `llmd_dir::find(&cwd)` at the start of every command; `.llmd/` is discovered by walking upward
- **Error handling:** `anyhow` everywhere; `.context()` and `.with_context()` on every `?`
- **Composability:** Primary output to stdout; diagnostics to stderr; no interactive prompts (except `--from -` for task from stdin)
- **Minimal dependencies:** No serde_yaml (hand-rolled frontmatter parser); no chrono (iso8601-timestamp); no petgraph (simple DFS in graph.rs)

### 2.3 Data Flow

**Compose flow:**
1. `llmd index` → `build_section_index()` → excludes catme, imported/, issues/ → prints numbered list
2. `llmd compose --sections 1,2,5` → `resolve_sections_from_indices()` → `build_document()` → header + catme excerpt + include files + sections
3. With `--issue 3` → `load_issue_context()` → parses issue frontmatter, loads `context-mappings.json`, maps labels to topics → auto-includes those topics unless `--no-auto-include`

**Build/serve flow:**
1. `generate_mdbook()` → copies all `.llmd/` files to `.mdbook/src/`, groups by directory
2. For `issues/` dir → `generate_roadmap_page()` → reads issues, groups by milestone, outputs epics + open issues table + progress bar
3. Writes `SUMMARY.md`, `book.toml` → runs `mdbook build` or `mdbook serve`

### 2.4 Issue File Format

- **Location:** `.llmd/issues/NNN-slug.md`
- **Frontmatter:** YAML between `---` fences; hand-parsed in `frontmatter.rs`
- **Fields:** id, title, slug, type, status, priority, labels (array of {name, color?}), assignee, milestone, parent, dependencies, epic_children, points, due, created_at, updated_at
- **Body:** Markdown; `## Comments` section with fenced YAML block for structured comments

---

## 3. Possible Extensions and Next Steps

### 3.1 Agent Workflow Enhancements

| Extension | Description |
|-----------|-------------|
| **Personas** | `.llmd/personas/` with role-specific context (e.g. `security-expert.md`); `llmd compose --persona security` |
| **Task templates** | Predefined compose templates: `llmd compose --template bugfix` → auto-selects Error Handling, conventions |
| **Diff-aware compose** | `llmd compose --changed` → include only sections related to files changed in working tree (requires git integration) |
| **Token budget** | `llmd compose --max-tokens 8000` → stop adding sections when estimate exceeds budget |

### 3.2 Issue Tracker Enhancements

| Extension | Description |
|-----------|-------------|
| **Bulk operations** | `llmd issue update --status closed 1 2 3` |
| **Export/import** | Export to GitHub Issues, Linear, Jira; import from same |
| **Webhooks / notifications** | On `--add-comment` with @mention, trigger external notification (e.g. Slack) |
| **Time tracking** | Optional `--log-time` on update; aggregate in roadmap |
| **Custom fields** | Extensible frontmatter; `context-mappings.json` could define custom fields for compose |

### 3.3 Documentation and Discovery

| Extension | Description |
|-----------|-------------|
| **Auto-refresh bootstrap** | `llmd bootstrap --watch` → re-run when source files change |
| **Section-level search** | `llmd search "auth" --sections` → return section indices, not just file:line |
| **Embedding-based search** | Optional semantic search when embeddings exist (e.g. from previous `llmd index` run with vector store) |
| **Doc coverage** | `llmd coverage` → report which source files/dirs have no mapped topic doc |

### 3.4 Integration

| Extension | Description |
|-----------|-------------|
| **IDE plugin** | VS Code / Cursor extension: sidebar for `.llmd/`, quick compose from current file |
| **CI integration** | `llmd compose --issue $ISSUE_ID` in CI to generate context for agent runs |
| **Git hooks** | Pre-commit: ensure new source files have Context Map entries |

---

## 4. Rebuild Recommendations

If rebuilding llmd from scratch, the following changes would improve clarity, maintainability, and extensibility.

### 4.1 Architecture

**Current:** Monolithic binary; all commands in one crate.

**Recommendation:** Consider a plugin or subcommand registry pattern. Commands could be registered at compile time but live in separate modules with a trait (`trait Command { fn run(&self) -> Result<()>; }`). This would make it easier to add third-party or experimental commands without touching `main.rs`. Alternatively, keep the current structure but document the "add a command" checklist more prominently — it is already clean.

**Current:** Compose has its own `resolve_issue_file` and `extract_labels_from_frontmatter` — duplicated logic with `issues::` module.

**Recommendation:** Refactor compose to use `issues::resolve_issue_path` and `issues::parse_issue` (or a lighter `parse_issue_frontmatter_only`) from the issues module. Single source of truth for issue resolution and frontmatter parsing.

### 4.2 Configuration

**Current:** No llmd config file; `context-mappings.json` is the only user-configurable JSON.

**Recommendation:** A minimal `.llmd/config.json` or `llmd.toml` at project root could hold:
- Default compose options (e.g. always include `conventions`)
- Persona definitions
- Label-to-topics mapping (move out of `context-mappings.json` or keep both)
- Path overrides (e.g. `llmd_dir = ".docs"`)

This would require careful design to avoid scope creep — the "no config file" simplicity is a feature.

### 4.3 Compose UX

**Current:** `--sections` is required for manual mode; with `--issue` and no `--no-auto-include`, sections can be empty (auto-include only). If user passes `--issue 3` and no `--sections`, they get auto-included topics but no explicit sections. The interaction between `--sections`, `--include`, and `--issue` is subtle.

**Recommendation:**
- Make the behaviour explicit: `llmd compose --issue 3` = header + catme + auto-included topics from labels. No sections.
- `llmd compose --issue 3 --sections 1,2,3` = same + those sections (additive).
- `llmd compose --issue 3 --no-auto-include --sections 1,2,3` = header + catme + sections only.
- Document this clearly in `--help` and README.

### 4.4 Issue Tracker

**Current:** Hand-rolled frontmatter parser; ~100 lines, handles only the needed types.

**Recommendation:** The hand-rolled parser is a deliberate choice to avoid serde_yaml. If the format grows (custom fields, nested structures), consider adding serde_yaml with a clear migration path — or keep the parser and extend it. The current approach is maintainable for the foreseeable future.

**Current:** `epic_children` is denormalised — updated when `--parent` is set. Risk of orphaned references if files are edited manually.

**Recommendation:** Add a `llmd issue validate` command that checks consistency (e.g. every `epic_children` entry has a matching `parent` on the child). Could run on `load_all_issues` in debug builds or as an explicit opt-in.

### 4.5 Testing

**Current:** Unit tests only in `markdown.rs`; no integration tests.

**Recommendation:**
- Add integration tests in `tests/` that:
  - Create a temp `.llmd/` with known content
  - Run `llmd read`, `llmd compose`, `llmd issue list` and assert output
- Add `--dry-run` or similar for compose to print what would be included without writing

### 4.6 Bootstrap

**Current:** Bootstrap prompt does not mention `.llmd/issues/` or `context-mappings.json`.

**Recommendation:** Extend the bootstrap prompt to describe:
- The issue tracker format (`llmd issue init`, file structure)
- Optional `context-mappings.json` for label-to-topics
- When to create initial issues (e.g. from a TODO list or PRD)

### 4.7 Scope and Philosophy

**Current:** llmd is "context management for agentic development" — documentation + issue tracking + compose.

**Recommendation:** Stay focused. Do not add:
- Code generation
- Test execution
- Deployment orchestration
- Real-time collaboration

The value is in being the *single place* for agents to get context. Compose + issues + serve is the right scope. Any new feature should answer: "Does this help an agent understand the codebase or plan work?"

---

## 5. Summary

llmd is a well-designed, minimal CLI that solves a real problem: agents need context without re-scanning entire codebases. The implementation is clean, uses few dependencies, and follows consistent patterns. The issue tracker integration is tight — compose, serve, and build all see issues. The main gaps are: duplicated logic between compose and issues, limited testing, and bootstrap not covering the issue tracker. A rebuild would benefit from refactoring duplicated code, adding integration tests, and clarifying compose behaviour — but the core architecture is sound and worth preserving.
