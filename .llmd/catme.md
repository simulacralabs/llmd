# llmd

> **Agent entry point.** Read this file first to orient yourself in the project.

## Project Summary

`llmd` is a context management CLI for agentic development. It manages a `.llmd/` directory — a persistent, machine-readable knowledge base of structured markdown files that AI agents (Claude Code, Cursor, Copilot, Gemini CLI, etc.) read on demand to understand a codebase. Instead of injecting an entire codebase into an agent's context window each session, agents call `llmd read catme` to orient themselves, `llmd compose` to assemble task-specific context, and `llmd search` to find patterns — consuming only what they need. The tool also imports existing agent config files (`AGENTS.md`, `.cursorrules`, `CLAUDE.md`, etc.) into `.llmd/imported/` and can build a browsable mdbook site from the `.llmd/` directory.

## Technology Stack

- **Language:** Rust (edition 2024)
- **CLI framework:** clap 4.5 (derive feature)
- **Error handling:** anyhow 1.0
- **Directory walking:** walkdir 2.5
- **Regex:** regex-lite 0.1
- **Build system:** Cargo
- **Optional runtime dependency:** mdbook (not in Cargo.toml — must be installed separately via `cargo install mdbook` for `llmd build` and `llmd serve`)

## Build & Test

```sh
# build
cargo build

# run tests (unit tests live in src/markdown.rs)
cargo test

# lint (zero warnings policy — -D warnings is enforced)
cargo clippy -- -D warnings

# format check
cargo fmt --check

# install locally from source
cargo install --path .

# pre-release readiness check (build + test + clippy + fmt + publish dry-run)
./scripts/check.sh
```

## Navigation

- [architecture.md](architecture.md) — entry point, module structure, data flow, and how the seven commands are dispatched
- [cli.md](cli.md) — full reference for every CLI command: flags, modes, examples, and output format
- [conventions.md](conventions.md) — Rust code style, error handling patterns, module organisation, and testing approach
- [deployment.md](deployment.md) — release process, `scripts/release.sh`, `scripts/check.sh`, and crates.io publishing

## Rules of Engagement

- Use `anyhow` for all error handling. Never introduce new error types (`thiserror`, custom enums, etc.). Every fallible function returns `anyhow::Result<()>` or `anyhow::Result<T>`.
- Use `.context("…")` and `.with_context(|| format!("…"))` on every `?` that could produce a confusing error message. Error messages must be user-facing and actionable.
- All CLI argument structs use `#[derive(Parser)]` from clap's derive API. Never use clap's builder API.
- Each command lives in its own file under `src/commands/`. Add new commands by: (1) creating `src/commands/<name>.rs`, (2) adding `pub mod <name>;` to `src/commands/mod.rs`, (3) adding the variant to `Command` in `src/main.rs`, and (4) adding the dispatch arm in `main()`.
- The `src/markdown.rs` module is pure logic with no I/O. Keep it that way — pass strings in, get strings out.
- The `src/llmd_dir.rs` module handles all `.llmd/` path resolution. Use `llmd_dir::find()` at the start of every command's `run()` function to locate the `.llmd/` directory.
- The `src/discovery.rs` module defines `FIXED_PATHS` as a static slice. Add new agent file formats there — do not hardcode paths in `init.rs`.
- `llmd` itself has no config file and no persistent state beyond the `.llmd/` directory it manages. Do not introduce runtime configuration files.
- Tests live in the same file as the code they test (inline `#[cfg(test)]` modules), not in a separate `tests/` directory. There are currently no integration tests.
- Edition 2024 Rust features are in use — `let-else`, `if let` chains, and `let … && …` patterns are all idiomatic here.
- Never write to stdout in command implementations except for the primary output (the content the user is requesting). Diagnostic messages go to `eprintln!`.
- The `.llmd/.mdbook/` and `.llmd/book/` directories are generated artifacts. Never commit them. They are recreated on each `llmd build` or `llmd serve` invocation.

## Context Map

- `src/main.rs` → [architecture.md](architecture.md)
- `src/commands/mod.rs` → [architecture.md](architecture.md)
- `src/commands/init.rs` → [cli.md](cli.md)
- `src/commands/bootstrap.rs` → [cli.md](cli.md)
- `src/commands/read.rs` → [cli.md](cli.md)
- `src/commands/compose.rs` → [cli.md](cli.md)
- `src/commands/search.rs` → [cli.md](cli.md)
- `src/commands/build.rs` → [cli.md](cli.md)
- `src/commands/serve.rs` → [cli.md](cli.md)
- `src/llmd_dir.rs` → [architecture.md](architecture.md)
- `src/discovery.rs` → [cli.md](cli.md)
- `src/markdown.rs` → [architecture.md](architecture.md)
- `scripts/check.sh` → [deployment.md](deployment.md)
- `scripts/release.sh` → [deployment.md](deployment.md)
