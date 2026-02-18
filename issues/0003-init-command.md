status: done

# 0003 — `init` Command

Implement `llmd init`: create `.llmd/`, import discovered files, generate `catme.md`.

## Scope

- Walk project root using `discovery::discover()`.
- Create `.llmd/` and `.llmd/imported/` directories.
- Copy each discovered file to `.llmd/imported/` with a flattened filename
  (e.g. `.github/copilot-instructions.md` → `github-copilot-instructions.md`).
- Generate `catme.md` with all standard sections: Project Summary, Technology Stack,
  Build & Test, Navigation (with imported file links), Rules of Engagement, Context Map.
- `--update` flag: skip the "already exists" guard, re-scan and overwrite catme.md.

## Notes

- Files are copied (not symlinked) so `.llmd/` is self-contained and portable.
- The `flatten_name` function strips leading dots and joins path components with `-`.
