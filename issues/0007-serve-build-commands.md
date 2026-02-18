status: done

# 0007 — `serve` and `build` Commands

Implement `llmd serve` and `llmd build`: mdbook integration for human-browsable docs.

## Scope

- `generate_mdbook(llms)` in `build.rs` — shared logic:
  - Creates `.llmd/.mdbook/src/` with all `.md` files copied in.
  - Generates `SUMMARY.md` (mdbook table of contents), with `catme.md` as the Overview.
  - Groups files by subdirectory. Skips `.mdbook/` and `book/` from the source scan.
  - Writes `book.toml` with the project name as the book title.
- `llmd build` — calls `mdbook build`, then moves output to `.llmd/book/` (or `--output <dir>`).
- `llmd serve` — calls `mdbook serve --port <port>`. Opens browser by default.
- Both commands check for `mdbook` binary and show an install hint if missing.

## Notes

- `generate_mdbook` is a `pub fn` in `build.rs` and called directly by `serve.rs` via `super::build::generate_mdbook`.
- No bundled mdbook — keeping the binary small and avoiding double maintenance.
