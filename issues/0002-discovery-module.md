status: done

# 0002 — Discovery Module

Implement `src/discovery.rs`: detect all known agent markdown file formats in a project tree.

## Scope

- `FIXED_PATHS` — a static list of well-known agent config filenames and their format descriptions.
  Covers: AGENTS.md, AGENTS.override.md, CLAUDE.md, GEMINI.md, AGENT.md, JULES.md, CONVENTIONS.md,
  SPEC.md, PRD.md, Plan.md, .cursorrules, .windsurfrules, .clinerules, .builderrules,
  .github/copilot-instructions.md, llms.txt, llms-full.txt.
- Directory scans for `.cursor/rules/`, `.claude/rules/`, and `.github/instructions/`.
- `DiscoveredFile` struct: path + format description string.
- `discover(root)` function — returns all found files.

## Notes

- Deliberately simple: no recursive walk needed, just check specific known locations.
- Format strings are `&'static str` — no allocation overhead.
