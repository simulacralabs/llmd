# llmd

**Context management for agentic development.**

`llmd` manages a `.llmd/` directory — a persistent, machine-readable knowledge base that AI agents (Claude Code, Cursor, Copilot, Gemini CLI, etc.) can read, search, and compose into task-specific context documents.

The core idea: instead of injecting your entire codebase into an agent's context window every session, you maintain a small set of structured markdown files in `.llmd/` that the agent reads on demand. One entry point (`catme.md`), one command to compose a task-context document, and a browsable web view via mdbook.

---

## Installation

```sh
cargo install llmd
```

Or build from source:

```sh
git clone https://github.com/simulacralabs/llmd
cd llmd
cargo install --path .
```

---

## Quick Start

```sh
# 1. Create the .llmd/ directory structure
llmd init

# 2. Populate it — pipe the bootstrap prompt to your agent CLI
llmd bootstrap | claude
llmd bootstrap | codex
llmd bootstrap > bootstrap-prompt.md   # or review before running

# 3. Read the entry point (agents do this first)
llmd read catme

# 4. Compose a task-context document (view index first, then select sections)
llmd index
llmd compose --sections 1,2,3 "add error handling to the auth module" > context.md

# 5. Browse the docs in a browser
llmd serve
```

---

## The .llmd/ Standard

A `.llmd/` directory contains:

```
.llmd/
  catme.md              # Agent entry point — read this first
  <topic>.md            # Topic-specific documentation (one file per concern)
  context-mappings.json # Optional: label-to-topics mapping for llmd compose --issue
  personas/             # Role-specific context (e.g. security-expert.md)
  imported/             # Existing agent config files, auto-imported by llmd init
  issues/               # Issue tracker (created by llmd issue init)
```

### catme.md

The entry point for agents. Sections:

- **Project Summary** — what the project does and why (one paragraph)
- **Technology Stack** — language, frameworks, key dependencies
- **Build & Test** — exact commands to build, test, and lint
- **Navigation** — links to topic docs with one-sentence summaries
- **Rules of Engagement** — architectural constraints and "don't touch" zones
- **Context Map** — maps source directories to their documentation files

### Topic files

Each `.md` file in `.llmd/` covers a single concern (e.g. `api-standards.md`, `auth-flow.md`, `database.md`). Write for agents: use clear H2/H3 headings, imperative language, and minimal working code snippets. Each section should be self-contained.

---

## Commands

### `llmd init [--update] [ROOT]`

Initialise a `.llmd/` directory. Run this first, then run `llmd bootstrap` to populate it.

### `llmd bootstrap [--show-existing]`

Prints a comprehensive prompt to stdout that instructs an LLM to analyse the codebase and write all `.llmd/` content. Pipe it directly to your agent CLI:

```sh
llmd bootstrap | claude
llmd bootstrap | codex
llmd bootstrap > prompt.md   # review before running
```

The prompt instructs the agent to:
- Read the entire codebase before writing anything
- Write `catme.md` (the agent entry point) with all sections filled in
- Write topic files for every major concern (`architecture.md`, `conventions.md`, and others as appropriate)
- Use specific, factual content — no placeholders

Use `--show-existing` to include the current `catme.md` in the prompt, useful when re-bootstrapping after partial edits.

Scans the project for known agent config files and imports them into `.llmd/imported/`:

- `AGENTS.md`, `CLAUDE.md`, `GEMINI.md`, `AGENT.md`, `JULES.md`
- `.cursorrules`, `.cursor/rules/*.md`
- `.github/copilot-instructions.md`, `.github/instructions/*.instructions.md`
- `.windsurfrules`, `.clinerules`, `.builderrules`
- `CONVENTIONS.md`, `SPEC.md`, `PRD.md`, `Plan.md`
- `llms.txt`, `llms-full.txt`

Use `--update` to re-scan without overwriting existing topic files.

### `llmd read <file> [OPTIONS]`

Read a file from `.llmd/`. Use `catme` as a shortcut for `catme.md`. Omit the `.md` extension for topic files.

```sh
llmd read catme
llmd read api-standards
llmd read auth-flow --section "Session Validation"
llmd read database --grep "transaction" --context 3
llmd read api-standards --lines 10:50
llmd read catme --tokens
```

Options:

- `--section <heading>` — extract one heading section (case-insensitive substring match)
- `--grep <pattern>` — filter to lines matching a regex (2 lines of context)
- `--lines <start:end>` — read a line range (1-indexed, inclusive)
- `--tokens` — print estimated token count to stderr before content

### `llmd index`

Print the numbered section index. Use this first to see available sections, then pass section numbers to `llmd compose --sections`.

```sh
llmd index

# Output:
# Available sections — use with `llmd compose --sections <nums>`:
# [1] architecture > Overview
# [2] architecture > Entry Point and Dispatch
# [3] conventions > Error Handling
# ...
```

### `llmd compose [OPTIONS] [TASK]`

Compose a task-context document from `.llmd/` content. Use `llmd index` first to view the section index, then pass section numbers via `--sections`. With `--issue`, topics are auto-included from the label-to-topics mapping in `.llmd/context-mappings.json` unless `--no-auto-include` is set.

```sh
# Manual selection (view index first)
llmd index
llmd compose --sections 1,2,5 "implement rate limiting on the API"

# From an issue — auto-include topics from labels
llmd compose --issue 3 > context.md

# From an issue — manual selection only
llmd compose --issue 3 --no-auto-include --sections 2,4,6 > context.md
```

Options:

- `-s, --sections <nums>` — section numbers from `llmd index` (comma-separated)
- `--issue <id|slug>` — compose from an issue; auto-includes topics from label mapping
- `--no-auto-include` — disable auto-inclusion when using `--issue`
- `-I, --include <topic,...>` — explicitly include these topic files in full (comma-separated, no `.md`)
- `--from <file>` — read the task description from a file
- `--output <file>` — write the composed document to a file instead of stdout

### `llmd search <query> [OPTIONS]`

Full-text regex search across all `.llmd/` files.

```sh
llmd search "error handling"
llmd search "TODO|FIXME" --context 0
llmd search "session" --dir imported
```

Options:

- `--context <n>` — lines of context to show around each match (default: 2)
- `--dir <subdir>` — restrict search to a subdirectory of `.llmd/`

### `llmd build [--output <dir>]`

Build a static mdbook site from `.llmd/`. Output goes to `.llmd/book/` by default.

Requires [mdbook](https://rust-lang.github.io/mdBook/): `cargo install mdbook`

```sh
llmd build
llmd build --output ./docs
```

### `llmd serve [--port <port>]`

Build and serve the mdbook locally. Opens a browser automatically.

```sh
llmd serve
llmd serve --port 8080 --no-open
```

---

## Agent Workflow

A typical agentic session using `llmd`:

```
Agent: llmd read catme
→ Gets project overview and navigation map

Agent: llmd index
→ Sees section index

Agent: llmd compose --sections 2,4,7 "add error handling to auth"
→ Gets a precisely composed context document

# Or with an issue (auto-include from labels):
Agent: llmd compose --issue 3 > context.md

Agent: llmd read auth-flow --section "Error Handling"
→ Gets the specific section it needs, token-efficiently

Agent: (implements the feature)

Agent: llmd search "error handling"
→ Verifies consistency with existing patterns
```

---

## Cross-tool Compatibility

`llmd` works alongside existing agent config files rather than replacing them. Running `llmd init` imports your existing `AGENTS.md`, `CLAUDE.md`, `.cursorrules`, etc. into `.llmd/imported/` so they're accessible through the same interface.

To keep a single source of truth across tools:

```sh
ln -s .llmd/imported/AGENTS.md AGENTS.md
ln -s .llmd/imported/AGENTS.md CLAUDE.md
```

---

## License

MIT
