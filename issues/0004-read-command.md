status: done

# 0004 — `read` Command

Implement `llmd read`: token-efficient file reading from `.llmd/`.

## Scope

- `llmd read catme` — shortcut for `catme.md`.
- `--section <heading>` — extract a single H2/H3 section by case-insensitive substring match.
- `--grep <pattern>` — filter output to matching lines with 2 lines of context each side.
- `--lines <start:end>` — read a specific line range (1-indexed, inclusive).
- `--tokens` — print an estimated token count to stderr before content.
- File resolution: try `.llmd/<name>`, `.llmd/<name>.md`, `.llmd/imported/<name>` in order.

## Notes

- Section extraction and grep are applied sequentially: section first, then grep.
  This lets users do `--section "Error Handling" --grep "panic"` to narrow further.
- Token estimate uses the 1-token-per-4-chars heuristic — fast and dependency-free.
