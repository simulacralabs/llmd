status: done

# 0005 — `compose` Command

Implement `llmd compose`: produce a single task-context document from `.llmd/` content.

## Scope

- Accept a task description string, `-` for stdin, or `--from <file>`.
- Always include: task header + catme.md excerpt (Project Summary, Technology Stack, Build sections).
- `--include <topic,...>` — explicitly pull in named topic files in full.
- Keyword matching: split task into words >3 chars, find `.llmd/` headings containing those words,
  extract and include those sections.
- `--output <file>` — write to file instead of stdout (e.g. `Plan.md`).
- Skips `catme.md` and `imported/` files from keyword matching (they're orientation/raw config).

## Notes

- Keyword extraction deduplicates and strips punctuation for cleaner matching.
- Intentionally simple matching: no NLP, no embeddings. Fast and transparent.
