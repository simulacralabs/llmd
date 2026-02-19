# Architecture

## Overview

`llmd` is a single-binary CLI tool with a flat, command-dispatched architecture. There is no server process, no daemon, and no persistent state beyond the `.llmd/` directory in the user's project. Every invocation is stateless: locate `.llmd/`, do work, exit.

```
src/
  main.rs            — CLI definition (Cli struct, Command enum) + dispatch
  commands/
    mod.rs           — re-exports all command modules
    init.rs          — llmd init
    bootstrap.rs     — llmd bootstrap
    read.rs          — llmd read
    compose.rs       — llmd compose
    search.rs        — llmd search
    build.rs         — llmd build  (also used by serve)
    serve.rs         — llmd serve
  llmd_dir.rs        — .llmd/ path resolution and file listing
  discovery.rs       — agent config file discovery for llmd init
  markdown.rs        — pure markdown utilities (no I/O)
```

## Entry Point and Dispatch

`src/main.rs` defines two types and one function:

- `Cli` — top-level `#[derive(Parser)]` struct with a single `command: Command` field.
- `Command` — `#[derive(Subcommand)]` enum with one variant per command, each wrapping the command's `Args` struct (`InitArgs`, `BootstrapArgs`, etc.).
- `main()` — calls `Cli::parse()`, then `match`es on the command variant to call the corresponding `commands::<name>::run(args)`.

All argument parsing is done by clap's derive macro at `Cli::parse()` time. Each command module owns its `Args` struct and its `run()` function. Nothing in `main.rs` does business logic.

## Command Modules

Each command module follows the same structure:

```rust
#[derive(Parser)]
pub struct <Name>Args { /* clap fields */ }

pub fn run(args: <Name>Args) -> Result<()> { /* command logic */ }
```

Private helper functions live in the same file. Commands that share logic call each other's public functions directly: `serve.rs` calls `build::generate_mdbook()` rather than duplicating it.

## `.llmd/` Path Resolution (`src/llmd_dir.rs`)

Every command begins with:

```rust
let cwd = std::env::current_dir()?;
let llmd = llmd_dir::find(&cwd)?;
```

`llmd_dir::find()` walks up the directory tree from `cwd` looking for a `.llmd/` directory. It returns the first one found, or an error with the message:

```
No .llmd/ directory found. Run `llmd init` in your project root to create one.
```

`llmd_dir` also provides:
- `catme_path(llmd)` — returns `llmd.join("catme.md")`
- `list_all_files(llmd)` — walks the `.llmd/` tree with `walkdir`, returns all `.md` files as `Vec<PathBuf>`

## Markdown Utilities (`src/markdown.rs`)

Pure functions, no I/O. All take `&str`, return `String` or `Option<String>`.

- `extract_section(source, section)` — finds the first H2/H3 whose text contains `section` (case-insensitive substring match), returns the raw markdown from that heading to the next heading of equal or higher depth. Returns `None` if not found.
- `list_headings(source)` — returns all headings as `Vec<(depth, text)>`.
- `estimate_tokens(text)` — returns `text.len().div_ceil(4)` (1 token ≈ 4 chars).
- `window(source, start, end)` — returns lines `start..=end` (1-indexed).
- `heading_depth(line)` — private; returns `#` count if line is a valid heading (must have space after `#`s), 0 otherwise.

The heading parser requires a space after the `#` characters (`## Heading` is valid, `##no-space` is not). This matches the CommonMark spec and prevents false positives in code blocks.

## Agent File Discovery (`src/discovery.rs`)

`discovery::discover(root)` scans the project root for known agent configuration files and returns a `Vec<DiscoveredFile>`. Each `DiscoveredFile` has:
- `path: PathBuf` — absolute path to the file
- `format: &'static str` — human-readable description (e.g. `"CLAUDE.md — Claude Code configuration"`)

Discovery covers:
- Fixed paths (checked by `Path::exists()`): `AGENTS.md`, `CLAUDE.md`, `GEMINI.md`, `AGENT.md`, `JULES.md`, `CONVENTIONS.md`, `SPEC.md`, `PRD.md`, `Plan.md`, `.cursorrules`, `.windsurfrules`, `.clinerules`, `.builderrules`, `.github/copilot-instructions.md`, `llms.txt`, `llms-full.txt`
- Directory globs (checked by `read_dir()`): `.cursor/rules/*.md` and `.cursor/rules/*.mdc`, `.claude/rules/*.md`, `.github/instructions/*.instructions.md`

## Data Flow: `llmd compose`

The most complex command. Its data flow:

1. Locate `.llmd/` via `llmd_dir::find()`.
2. Read `catme.md`; extract Project Summary, Technology Stack, and Build sections via `markdown::extract_section()` to form `catme_excerpt`.
3. List all `.md` files via `llmd_dir::list_all_files()`. Exclude `catme.md` and `imported/` files.
4. Build a flat `Vec<IndexedSection>` by calling `markdown::list_headings()` on each file and collecting H2 and H3 headings.
5. In **keyword mode**: call `extract_keywords(task)` (splits on whitespace, strips punctuation, lowercases, deduplicates, filters words ≤ 3 chars), then match keywords against `section.heading.to_lowercase()`.
6. In **interactive mode**: print the numbered index to stdout, read comma/newline-separated numbers from stdin, map back to `IndexedSection` entries.
7. Call `build_document()`: prepend the task description and `catme_excerpt`, then for each chosen section call `markdown::extract_section()` on its file to get content.
8. Write to `args.output` path or print to stdout.

## Data Flow: `llmd build` / `llmd serve`

Both commands call `build::generate_mdbook(llmd)`, which:

1. Creates `.llmd/.mdbook/src/`.
2. Copies `catme.md` as the Overview page.
3. Groups all other `.md` files by their parent directory.
4. Copies each file to `.mdbook/src/` (preserving subdirectory structure).
5. Generates `SUMMARY.md` with the mdbook chapter list.
6. Writes `book.toml` with the project name and `no-section-label = true`.

`build` then runs `mdbook build` and copies the output to `.llmd/book/` (or the `--output` path).  
`serve` runs `mdbook serve` directly on the `.mdbook/` project directory.

Both commands call `ensure_mdbook()` first, which runs `mdbook --version` and returns an actionable error if mdbook is not installed.
