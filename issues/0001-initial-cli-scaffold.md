status: done

# 0001 — Initial CLI Scaffold

Set up the Cargo workspace, module structure, and clap-based CLI entry point.

## Scope

- `Cargo.toml` with minimal dependencies (clap, anyhow, walkdir, regex-lite)
- `src/main.rs` — clap `Parser` + `Subcommand` enum dispatching to each command module
- `src/commands/mod.rs` — re-exports all command modules
- Stub modules for all six commands: `init`, `read`, `compose`, `search`, `serve`, `build`

## Notes

- Named each command's arg struct `<Name>Args` (e.g. `InitArgs`) to avoid conflict with `clap::Args` trait.
- Chose `regex-lite` over `grep-searcher`/`grep-regex` to keep the dependency tree minimal — all search operations are in-memory over small `.llmd/` files.
- Dropped `pulldown-cmark`, `serde`, `serde_yml`, and `ignore` — not needed for the line-oriented markdown approach used.
