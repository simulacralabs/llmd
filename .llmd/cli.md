# CLI Commands

## `llmd init [--update] [ROOT]`

Initialises a `.llmd/` directory at `ROOT` (default: current directory).

**What it does:**
1. Creates `.llmd/` and `.llmd/imported/`.
2. Calls `discovery::discover(root)` to find all known agent config files in the project tree.
3. Copies each discovered file into `.llmd/imported/`, flattening nested paths into hyphenated names (e.g. `.github/copilot-instructions.md` becomes `github-copilot-instructions.md`).
4. Generates a skeleton `catme.md` with placeholder `<!-- … -->` comments and a Navigation section listing the imported files.

**Flags:**
- `--update` — re-scan and regenerate `catme.md` without deleting existing `.llmd/` topic files. Without this flag, running `llmd init` on an existing `.llmd/` directory exits with an error: `.llmd/ already exists. Use --update to refresh it without losing existing files.`

**After init:** run `llmd bootstrap` to get a prompt that fills in the placeholders.

**Path flattening rule:** implemented in `flatten_name()` in `init.rs`. Strips leading `.` from path components and joins them with `-`. The `.llmd/` directory itself is not included.

## `llmd bootstrap [--show-existing]`

Prints a comprehensive bootstrap prompt to stdout. Designed to be piped to an agentic CLI:

```sh
llmd bootstrap | claude
llmd bootstrap | codex
llmd bootstrap > bootstrap-prompt.md   # review first
```

The prompt instructs the agent to read the entire codebase, then write `catme.md` and all appropriate topic files to `.llmd/`, with specific, non-placeholder content.

**Flags:**
- `--show-existing` — appends the current contents of `catme.md` to the prompt. Useful when re-bootstrapping after partial edits; the agent sees what is already filled in.

**Implementation:** `build_prompt()` in `bootstrap.rs` produces the full prompt as a static format string. The prompt body is the same text that was used to generate the current `.llmd/` content — it is self-describing and self-referential.

## `llmd read <FILE> [OPTIONS]`

Reads a file from `.llmd/` and prints it to stdout.

**File resolution order** (implemented in `resolve_file()` in `read.rs`):
1. Exact match: `llmd.join(name)` — e.g. `llmd read catme.md`
2. With `.md` extension: `llmd.join(format!("{name}.md"))` — e.g. `llmd read catme` resolves to `catme.md`, `llmd read architecture` resolves to `architecture.md`
3. Special alias: `"catme"` is normalised to `"catme.md"` before resolution.
4. Inside `imported/`: `llmd.join("imported").join(name)`
5. Error with message: `File "{name}" not found in .llmd/. Run llmd search {name} to find it, or llmd read catme to browse available docs.`

**Options:**
- `--section <HEADING>` / `-s <HEADING>` — extract one section. Uses `markdown::extract_section()`: case-insensitive substring match on the heading text. Returns from the matched heading to the next heading of equal or higher depth.
- `--grep <PATTERN>` / `-g <PATTERN>` — filter to lines matching a regex-lite regex, with 2 lines of context on each side. Discontinuous groups are separated by `...`.
- `--lines <START:END>` / `-l <START:END>` — return lines `START` through `END` (1-indexed, inclusive). Uses `markdown::window()`.
- `--tokens` / `-T` — print `~N tokens` to stderr before the content. Uses `markdown::estimate_tokens()` (1 token ≈ 4 chars).

**Output:** printed to stdout. A trailing newline is always ensured.

**Examples:**
```sh
llmd read catme
llmd read architecture --section "Data Flow"
llmd read conventions --grep "anyhow" --context 3
llmd read cli --lines 10:50
llmd read catme --tokens
```

## `llmd index`

Prints the numbered section index to stdout. No interaction. Use this first, then pass section numbers to `llmd compose --sections`.

```
Available sections — use with `llmd compose --sections <nums>`:

[1] architecture > Overview
[2] architecture > Entry Point and Dispatch
...
```

## `llmd compose [OPTIONS] [TASK]`

Builds a context document from explicitly chosen sections and/or an issue. Two-step flow: run `llmd index` first, then `llmd compose --sections <nums>`.

**With `--issue`:** when composing from an issue, topics are auto-included from `.llmd/context-mappings.json` based on the issue's labels, unless `--no-auto-include` is set. The mapping format:

```json
{
  "label_to_topics": {
    "auth-flow": ["auth-flow", "api-standards", "conventions"],
    "parser": ["architecture", "conventions"]
  }
}
```

**Options:**
- `-s, --sections <NUMS>` — section numbers from `llmd index` (comma-separated)
- `--issue <ID|SLUG>` — compose from an issue; auto-includes topics from label mapping
- `--no-auto-include` — disable auto-inclusion when using `--issue` (select manually with `--sections`)
- `-I, --include <TOPIC,...>` — explicitly include these topic files in full (comma-separated, no `.md` extension)
- `--from <FILE>` / `-f <FILE>` — read the task description from a file
- `--output <FILE>` / `-o <FILE>` — write the document to a file instead of stdout

**Section index exclusions:** `catme.md`, `imported/`, and `issues/` are excluded from the index.

**Examples:**
```sh
llmd index
llmd compose --sections 1,2,5 "add error handling to the auth module"
llmd compose --issue 3
llmd compose --issue 3 --no-auto-include --sections 2,4,6
llmd compose --from task.md --output context.md
```

## `llmd search <QUERY> [OPTIONS]`

Full-text regex search across all `.md` files in `.llmd/`.

**Output format** (one block per matching file):
```
<relative-path-from-llmd>:
  >    5: <matching line>
       6: <context line>
  >    9: <another match>
  ...
```
`>` marks matching lines; space marks context lines. Discontinuous groups are separated by `  ...`.

**Options:**
- `--context <N>` / `-c <N>` — lines of context before and after each match (default: 2).
- `--dir <SUBDIR>` / `-d <SUBDIR>` — restrict search to a subdirectory of `.llmd/`. Example: `--dir imported` searches only `.llmd/imported/`.

**Summary line:** printed to stderr after all results: `N match(es) found.` or `No matches found for "…"`.

**Examples:**
```sh
llmd search "error handling"
llmd search "TODO|FIXME" --context 0
llmd search "session" --dir imported
llmd search "anyhow" --context 1
```

## `llmd build [--output <DIR>]`

Builds a static mdbook site from `.llmd/`.

**Prerequisites:** `mdbook` must be installed (`cargo install mdbook`). The command checks with `mdbook --version` and exits with an error if not found.

**Process:**
1. Calls `build::generate_mdbook()` to create a temporary mdbook project at `.llmd/.mdbook/`.
2. Runs `mdbook build .llmd/.mdbook/`.
3. Copies the built `book/` output to `.llmd/book/` (default) or `--output`.

**Options:**
- `--output <DIR>` / `-o <DIR>` — destination directory for the built HTML site (default: `.llmd/book/`).

**Generated artifacts** (do not commit these):
- `.llmd/.mdbook/` — temporary mdbook project (auto-recreated on each build)
- `.llmd/book/` — built static site

## `llmd serve [--port <PORT>] [--no-open]`

Builds an mdbook from `.llmd/` and serves it locally with live reload.

**Prerequisites:** same as `llmd build` — `mdbook` must be installed.

**Process:** calls `build::generate_mdbook()` then runs `mdbook serve --port <PORT> .llmd/.mdbook/`. The `mdbook serve` process takes over the terminal (blocking) until Ctrl+C.

**Options:**
- `--port <PORT>` / `-p <PORT>` — port to listen on (default: 3000).
- `--no-open` — do not open a browser tab automatically.

**Example:**
```sh
llmd serve
llmd serve --port 8080 --no-open
```
