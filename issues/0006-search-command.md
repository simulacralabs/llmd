status: done

# 0006 — `search` Command

Implement `llmd search`: full-text regex search across all `.llmd/` files.

## Scope

- Accept a regex pattern.
- Search all `.md` files in `.llmd/` (recursively).
- Print: filename, then matching lines with line numbers and configurable context (`--context N`, default 2).
- `--dir <subdir>` — restrict search to a subdirectory (e.g. `imported`).
- Matching lines are marked with `>`, context lines with ` `.
- Summary: `N match(es) found.` printed to stderr.

## Notes

- Uses `regex-lite` — fast enough for the small file sizes typical of `.llmd/` directories.
- "No matches" message goes to stderr so stdout can be piped cleanly.
