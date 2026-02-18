# Releasing llmd to crates.io

This document covers the complete workflow for versioning, building, tagging, and publishing a release.

---

## Prerequisites

- `cargo login` has already been run (credentials stored in `~/.cargo/credentials.toml`)
- You are on the `master` branch with a clean working tree
- All changes are committed and tests pass: `cargo test`

---

## Release Workflow

### 1. Choose a version

`llmd` follows [Semantic Versioning](https://semver.org/):

| Change | Version bump | Example |
|--------|-------------|---------|
| Bug fix, docs, minor improvement | Patch | `0.1.0` → `0.1.1` |
| New command or feature, backwards-compatible | Minor | `0.1.0` → `0.2.0` |
| Breaking CLI change (renamed flag, removed command) | Major | `0.1.0` → `1.0.0` |

The first public release is `0.1.0`. The planned stable release is `1.0.0`.

---

### 2. Update the version in Cargo.toml

Edit the `version` field in `Cargo.toml`:

```toml
[package]
version = "0.1.0"
```

Then update `Cargo.lock` by running any cargo command:

```sh
cargo build
```

---

### 3. Verify the package

Do a dry run to catch any packaging issues before publishing:

```sh
cargo publish --dry-run
```

This checks:
- All files listed in the package are present
- `Cargo.toml` metadata is valid (description, license, repository, keywords)
- The crate compiles cleanly from the packaged source

If you want to inspect exactly what will be uploaded:

```sh
cargo package --list
```

---

### 4. Run tests one final time

```sh
cargo test
cargo clippy -- -D warnings
```

---

### 5. Commit and tag

```sh
git add Cargo.toml Cargo.lock
git commit -m "chore: release v0.1.0"
git tag v0.1.0
git push origin master --tags
```

The tag format is `vMAJOR.MINOR.PATCH` (e.g. `v0.1.0`). This is the conventional format and is what crates.io links to as the release tag.

---

### 6. Publish to crates.io

```sh
cargo publish
```

The crate will appear at `https://crates.io/crates/llmd` within a few minutes.

> **Note:** crates.io releases are permanent. You can yank a version (`cargo yank --version 0.1.0`) to prevent new projects from depending on it, but the version itself cannot be deleted.

---

## Publishing v0.1.0 Right Now

Run these commands in order from the project root:

```sh
# 1. Confirm tests pass
cargo test

# 2. Set version to 0.1.0 in Cargo.toml (already set), rebuild lock file
cargo build

# 3. Dry run
cargo publish --dry-run

# 4. Commit
git add Cargo.toml Cargo.lock
git commit -m "chore: release v0.1.0"

# 5. Tag
git tag v0.1.0
git push origin master --tags

# 6. Publish
cargo publish
```

---

## After Publishing

- Check the crate page: `https://crates.io/crates/llmd`
- Verify `cargo install llmd` works from a clean environment
- Create a GitHub release at `https://github.com/simulacralabs/llmd/releases/new`
  - Tag: `v0.1.0`
  - Title: `llmd v0.1.0`
  - Body: summarise the changes (use `git log` for reference)

---

## Yanking a Bad Release

If a release has a critical bug:

```sh
cargo yank --version 0.1.0
```

This prevents new `Cargo.toml` files from resolving to that version. Existing users who already have it pinned are unaffected. Publish a patched `0.1.1` as the fix.

---

## Version History

| Version | Status | Notes |
|---------|--------|-------|
| `1.0.0` | current (unreleased) | Full feature set as described in plan |
| `0.1.0` | planned first release | — |

> Update this table after each release.
