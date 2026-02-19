# Deployment

## Release Process

`llmd` is published to [crates.io](https://crates.io/crates/llmd). The release process is fully scripted.

**Before releasing:**
```sh
# Run the pre-release check (build + test + clippy + fmt + publish dry-run)
./scripts/check.sh
```

If all checks pass, run the release script:
```sh
./scripts/release.sh
```

`release.sh` is interactive — it will prompt for confirmation before tagging and publishing.

## `scripts/check.sh`

Runs five checks in sequence and reports results:

| Check | Command |
|-------|---------|
| cargo build | `cargo build` |
| cargo test | `cargo test` |
| cargo clippy | `cargo clippy -- -D warnings` |
| cargo fmt | `cargo fmt --check` |
| publish dry-run | `cargo publish --dry-run --allow-dirty` |

**Exit codes:**
- `0` — all checks passed
- `1` — one or more checks failed

**On failure:** writes `check-report.md` to the repo root with the full compiler output for each failed check. This file is designed to be passed to an agent for automated resolution:
```sh
./scripts/check.sh
# if it fails:
cat check-report.md | claude
```

**`--report-only` flag:** always writes `check-report.md` even when all checks pass. Useful for CI or record-keeping.

**Git warnings** (non-blocking, informational):
- Uncommitted changes in the working tree
- No `origin` remote configured

## `scripts/release.sh`

Handles version bumping, git tagging, and `cargo publish`. Read the script directly for the exact interactive prompts and steps:
```sh
cat scripts/release.sh
```

## Installation Methods

**From crates.io (end users):**
```sh
cargo install llmd
```

**From source:**
```sh
git clone https://github.com/simulacralabs/llmd
cd llmd
cargo install --path .
```

## Optional Runtime Dependency: mdbook

`llmd build` and `llmd serve` require `mdbook` at runtime. It is not declared in `Cargo.toml` because it is a separate binary. Users must install it separately:

```sh
cargo install mdbook
```

Both commands call `ensure_mdbook()` before doing any work. If `mdbook --version` fails, the user gets:
```
`mdbook` is not installed. Install it with:
  cargo install mdbook
```

## Generated Artifacts

The following paths are created at runtime and must not be committed:

- `.llmd/.mdbook/` — temporary mdbook project directory, recreated on each `llmd build` or `llmd serve`
- `.llmd/book/` — built static HTML site from `llmd build`
- `check-report.md` — written by `./scripts/check.sh` on failure (or with `--report-only`)

Ensure these are in `.gitignore` for projects using `llmd`:
```
.llmd/.mdbook/
.llmd/book/
check-report.md
```
