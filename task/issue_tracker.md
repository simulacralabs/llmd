# Project Specification: Rust Crate for Lightweight Agentic Task Tracker

## Project Overview

### Project Name
`agentic-tracker`

### Description
`agentic-tracker` is a Rust crate that provides a lightweight, flat-file-based task and issue tracking system optimized for agentic development workflows. Inspired by systems like Beads (graph-based dependencies for agents) and Beans (Markdown-driven simplicity), it emphasizes minimalism, context efficiency for LLMs/agents, and zero external dependencies beyond the Rust standard library where possible. The system uses flat Markdown files with compact JSON frontmatter for metadata, stored in a single directory, making it easy for humans and AI agents to read/write/parse without databases or heavy tooling.

The crate can be used as:
- A library for embedding in other Rust applications (e.g., agent frameworks).
- A CLI binary for direct command-line interaction.

It implements the core features from the initial Python prototype (task creation, dependencies, status management, ready-task computation) and extends them with natural additions like epics (hierarchies), task types, structured threading/messaging, milestones, estimates, enhanced labels, agent handoffs, and roadmap generation—all while maintaining flat-file purity and low token/context overhead for agents.

### Goals
- **Minimalism**: No databases (e.g., SQLite), no web servers, no TUI. Use Rust's stdlib for file I/O, JSON (via `serde`), and CLI parsing (via `clap`). Aim for <500 LOC core, extensible modules.
- **Agentic Optimization**: Easy for AI agents to integrate—JSON outputs for queries, dependency graphs for task sequencing, handoff mechanisms for multi-agent coordination.
- **Context Efficiency**: Frontmatter JSON is compact (minimal keys, no deep nesting). Files are human-readable Markdown for descriptions/comments.
- **Portability**: Works in any git repo; versionable files for shared state.
- **Extensibility**: Modular design for future additions without bloat.
- **Performance**: Fast for 1000+ tasks (in-memory parsing on demand).
- **Security**: No network access; local files only. Validate inputs to prevent injection.

### Target Audience
- Developers using agentic workflows (e.g., with Grok, Claude, or custom LLMs).
- Teams wanting a Beads/Beans-like tool but in Rust for performance/safety.
- Solo devs seeking a git-friendly alternative to Jira/Trello.

### Versioning and Licensing
- Initial version: 0.1.0
- License: MIT (permissive for adoption).
- Repository: GitHub (e.g., `ursinaerex/agentic-tracker`).

## Features

### Core Features (From Initial Prototype)
1. **Initialization**: Create `.tracker/` directory and `config.json` for next ID tracking.
2. **Task Creation**: Generate new tasks with auto-incrementing ID, slug from title, default metadata.
3. **Listing Tasks**: Output tasks in table or JSON format, with filters (e.g., by status, priority).
4. **Showing Tasks**: Display full Markdown content of a task.
5. **Updating Tasks**: Modify status, priority, assignee, add dependencies, append comments.
6. **Ready Tasks**: Compute and list open tasks with no unresolved dependencies (on-the-fly graph resolution).
7. **File Format**: Markdown files named `<id>-<slug>.md` with JSON frontmatter (delimited by `---json` and `---`).
   - Metadata keys: `id`, `title`, `slug`, `status` ("open", "in_progress", "closed"), `priority` ("low", "medium", "high"), `labels` (array), `assignee` (string), `created_at`/`updated_at` (ISO timestamps), `dependencies` (array of IDs).

### Extended Features (Natural Progressions)
1. **Epics and Hierarchies**: Add `parent` (single ID for nesting under epics) and optional `children` (derived). CLI commands for tree views and epic-specific ready tasks.
2. **Task Types**: Add `type` ("task", "bug", "feature", "chore", "spike", "epic"). Filterable in lists/ready.
3. **Structured Threading/Messaging**: Comments as YAML-like blocks in the Markdown body (with `date`, `author`, `body`). Support `@mentions` for notifications. CLI for adding comments with auto-timestamp/author.
4. **Milestones/Releases**: Add `milestone` (string, e.g., "v1.2"). Group/filter by milestone; generate progress overviews.
5. **Estimates and Due Dates**: Add `points` (integer) and `due` (ISO date). Sort/filter by due; compute velocity if needed.
6. **Enhanced Labels/Tags**: Labels as array; optional color/category metadata (e.g., `[{"name": "ui", "color": "#ffcc00"}]`).
7. **Agent Handoffs**: Add `handover_to` (string, e.g., "agent-ui") for multi-agent passing. Or use structured comments with `@agent-name`.
8. **Roadmap Generation**: CLI command to aggregate tasks into a single Markdown file, grouped by milestone/epic/type, with progress bars (emoji-based) and summaries.
9. **Additional Utilities**:
   - Cycle detection in dependencies/parents (using graph algorithms).
   - Mentions extraction: List tasks with `@handle`.
   - Aggregation: Dump all tasks to a single JSON for bulk agent consumption.
   - Resolution on Close: Add `resolution` ("done", "wontfix", "duplicate:<id>").

## Architecture

### Directory Structure
- `.tracker/` (root directory in user's project).
  - `config.json`: `{ "next_id": 1 }` (atomic updates via file locking if needed).
  - `<id>-<slug>.md`: One per task (e.g., `001-fix-bug.md`).
- No subdirectories to keep flat.

### Modules
- `lib.rs`: Entry point, exports public API.
- `models.rs`: Structs for Task (serde-serializable), Config.
- `file_ops.rs`: Functions for reading/writing files, parsing frontmatter (custom parser or `serde_json` with string slicing).
- `commands.rs`: CLI subcommands (using `clap`).
- `graph.rs`: Dependency resolution, ready tasks, cycle detection (use `petgraph` or simple vec-based for minimal deps).
- `roadmap.rs`: Generator for aggregated Markdown.
- `utils.rs`: Slug generation, timestamp handling (use `chrono`).

### Dependencies
- Core: `serde` (JSON), `chrono` (dates), `clap` (CLI parsing).
- Optional: `petgraph` (for advanced graphs, if cycles/hierarchies grow complex; fallback to simple DFS).
- Dev: `anyhow` (error handling), `tempfile` (tests).
- No heavy crates like databases or web frameworks.

### Data Flow
1. **Parsing**: Read file, slice frontmatter (find delimiters), deserialize JSON to Task struct. Body as String.
2. **Writing**: Serialize Task to JSON, wrap in delimiters, append body.
3. **Graph Operations**: Load all tasks into a HashMap<ID, Task>, build directed graph (deps as edges), compute ready (in-degree 0 + open), detect cycles.
4. **CLI**: Use `clap` for subcommands; output JSON or text based on flags.
5. **Library Usage**: Public functions like `create_task`, `get_ready_tasks`, `generate_roadmap`.

### Error Handling
- Use `Result` with custom Error enum (e.g., InvalidFrontmatter, TaskNotFound, CycleDetected).
- Graceful failures: e.g., skip invalid files in lists.

### Performance Considerations
- For large sets: Lazy loading (parse only needed files via glob).
- Memory: In-memory for graphs, but limit to essentials.

## CLI Design

### Commands
- `agentic-tracker init`: Set up directory/config.
- `agentic-tracker new <title> [--type <type>] [--priority <pri>] [--labels <comma-sep>] [--assignee <name>] [--parent <id>] [--milestone <name>] [--points <num>] [--due <date>]`: Create task.
- `agentic-tracker list [--json] [--filter-status <status>] [--filter-type <type>] [--filter-milestone <name>] [--sort <due|priority|id>]`: List tasks.
- `agentic-tracker show <id|slug>`: Display full task.
- `agentic-tracker update <id|slug> [--status <status>] [--type <type>] [--priority <pri>] [--assignee <name>] [--add-dep <id>] [--add-label <label>] [--parent <id>] [--milestone <name>] [--points <num>] [--due <date>] [--resolution <res>] [--handover-to <agent>] [--add-comment <text> [--author <name>]]`: Update.
- `agentic-tracker ready [--json] [--type <type>] [--milestone <name>]`: List ready tasks.
- `agentic-tracker tree <id>`: Show hierarchy for epic.
- `agentic-tracker roadmap [--output <file.md>]`: Generate aggregated roadmap.
- `agentic-tracker mentions [@handle]`: List tasks with mentions.
- Global flags: `--dir <path>` (override `.tracker/`), `--json` (machine-readable output).

### Example Usage
```bash
agentic-tracker init
agentic-tracker new "Implement auth" --type feature --priority high --milestone v1.0 --points 5 --due 2026-03-01
agentic-tracker update 1 --status in_progress --add-comment "Started investigation" --author agent-claude
agentic-tracker ready --json
```

## Integration with Agents
- **Direct File Access**: Agents can use Rust code or scripts to read/write files (e.g., via `std::fs`).
- **CLI Wrapping**: In agent prompts: "Run 'agentic-tracker ready --json' to get next task."
- **Library Embedding**: For Rust-based agents, import crate and call `get_ready_tasks(&dir)` directly.
- **Context Efficiency**: JSON outputs are flat arrays/objects; roadmap is a single file for high-level overviews.

## Development Plan

### Milestones
1. **v0.1**: Core features (init, new, list, show, update, ready).
2. **v0.2**: Extensions (epics, types, structured comments).
3. **v0.3**: Milestones, estimates, handoffs, roadmap.
4. **v0.4**: Polish (cycle detection, mentions, tests).

### Testing
- Unit: Parse/write roundtrips, graph algos.
- Integration: CLI end-to-end with temp dirs.
- Coverage: Aim for 90%+.

### Documentation
- README.md: Setup, examples, file format.
- API Docs: Via `cargo doc`.
- Contribution Guide: For extensions.

This spec provides a complete blueprint. If implemented, it would yield a robust, Rust-native alternative to the Python prototype, ready for agentic use. If you need code skeletons or refinements (e.g., specific structs), let me know!