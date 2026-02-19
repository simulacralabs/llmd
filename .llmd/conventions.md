# Conventions

## Error Handling

Use `anyhow` exclusively. The return type of every fallible function is `anyhow::Result<T>` or `anyhow::Result<()>`. Do not define custom error types.

Every `?` that could yield a confusing error message must be followed by `.context("…")` (static message) or `.with_context(|| format!("…", …))` (dynamic message). The message must be user-facing and actionable — it becomes part of the error chain printed to the user.

Good examples from the codebase:
```rust
fs::create_dir_all(&imported).context("Failed to create .llmd/imported/")?;
fs::copy(&file.path, &dest).with_context(|| {
    format!("Failed to copy {} to .llmd/imported/", file.path.display())
})?;
let content = fs::read_to_string(&path)
    .with_context(|| format!("Cannot read {}", path.display()))?;
```

Use `anyhow::bail!("…")` for early exits with a plain error message (no `?` operator needed). Use `anyhow::bail!` instead of `return Err(anyhow::anyhow!("…"))`.

## Stdout vs Stderr

- **stdout** — primary command output only: the content the user or downstream pipeline expects to consume (e.g. composed document, file contents, bootstrap prompt, section index in compose).
- **stderr** — all diagnostic output: progress messages, warnings, token counts, match counts, paths of written files. Use `eprintln!` for these. Never use `println!` for diagnostic output.

This rule ensures commands are composable with pipes:
```sh
llmd bootstrap | claude
llmd compose "add caching" > context.md
llmd read catme --tokens  # token count on stderr, content on stdout
```

## Module Organisation

Each command is a self-contained module in `src/commands/<name>.rs`. The module contains:
1. The `Args` struct with `#[derive(Parser)]`.
2. The `pub fn run(args: <Name>Args) -> Result<()>` entry point.
3. All private helper functions used only by that command.

When two commands share logic, make the shared function `pub` in the module that owns it and call it from the other. Example: `serve.rs` calls `build::generate_mdbook()` rather than duplicating it.

Shared utilities that are not command-specific live in top-level modules:
- `src/llmd_dir.rs` — `.llmd/` directory location and file listing
- `src/discovery.rs` — agent file discovery
- `src/markdown.rs` — pure markdown parsing/extraction

## CLI Argument Definitions

Use clap's derive API exclusively. All argument structs use `#[derive(Parser)]`. Never use clap's builder (`.command()` / `.arg()`) API.

Flag naming convention:
- Use kebab-case for multi-word flags: `--show-existing`, `--no-open`.
- Short flags are single characters: `-s`, `-g`, `-T`, `-i`, `-I`, `-o`, `-f`.
- Flags that accept values use `value_name` to document the type in `--help` output: `#[arg(long, value_name = "FILE")]`.
- Boolean flags that disable a default are prefixed with `no-`: `--no-open`.

## Naming Conventions

- **Types** (structs, enums): `PascalCase` — `InitArgs`, `IndexedSection`, `DiscoveredFile`.
- **Functions and variables**: `snake_case` — `run_interactive_mode`, `build_section_index`, `catme_excerpt`.
- **Constants and statics**: `SCREAMING_SNAKE_CASE` — `FIXED_PATHS`.
- **Modules and files**: `snake_case` — `llmd_dir.rs`, `bootstrap.rs`.
- Command `Args` structs are named `<Command>Args` — `ReadArgs`, `ComposeArgs`, `BuildArgs`.

## Tests

Tests live in the same file as the code they test, in an inline `#[cfg(test)]` module at the bottom of the file. There are currently no integration tests and no separate `tests/` directory.

Only `src/markdown.rs` has tests (the pure functions are the most testable). Command functions that mix I/O with logic are not currently tested.

When adding tests, follow the existing pattern in `src/markdown.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_<function_name>() {
        // ...
    }
}
```

Test function names use the `test_<function_name>` pattern.

## Rust Edition and Language Features

The crate uses Rust edition 2024. The following edition-2024 / recent stable features are in use and should be treated as idiomatic:

- `let-else` expressions
- `if let` chains (`if let Some(x) = foo && x > 0`)
- `let … && …` in `if` guards (e.g. in `search.rs` and `read.rs`)
- `div_ceil()` on integers (in `markdown::estimate_tokens`)

Do not avoid these features for compatibility — the crate targets current stable Rust.

## File I/O Patterns

- Use `std::fs::read_to_string()` to read entire files into strings. `llmd` deals exclusively with small markdown files; streaming is not needed.
- Use `std::fs::write()` to write entire files atomically. Do not open files for incremental writes.
- Propagate all I/O errors with `.context()` / `.with_context()`.
- When copying directories (e.g. `copy_dir_all` in `build.rs`), recurse manually with `fs::read_dir`. Do not add a dependency for this.

## Adding a New Command

1. Create `src/commands/<name>.rs` with an `Args` struct and a `pub fn run(args: Args) -> Result<()>`.
2. Add `pub mod <name>;` to `src/commands/mod.rs`.
3. Add the import to the `use commands::{…}` block in `src/main.rs`.
4. Add the variant to the `Command` enum in `src/main.rs`:
   ```rust
   /// One-line doc comment shown in --help
   <Name>(<Name>Args),
   ```
5. Add the dispatch arm in `main()`:
   ```rust
   Command::<Name>(args) => commands::<name>::run(args),
   ```

## Adding a New Agent File Format to Discovery

Add an entry to the `FIXED_PATHS` static slice in `src/discovery.rs`:
```rust
("<relative-path-from-root>", "<short description of the format>"),
```

For directory-based formats (like `.cursor/rules/`), add a new `read_dir` block in `discovery::discover()` following the existing pattern.
