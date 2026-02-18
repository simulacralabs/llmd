#!/usr/bin/env bash
# release.sh — interactive release wizard for llmd
#
# Assumes ./scripts/check.sh has already been run and passed.
# Guides you through: version bump, changelog, commit, tag, push, publish.
#
# Usage: ./scripts/release.sh

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CARGO_TOML="$REPO_ROOT/Cargo.toml"

# --- colour helpers ---

green()  { printf '\033[32m%s\033[0m\n' "$*"; }
yellow() { printf '\033[33m%s\033[0m\n' "$*"; }
red()    { printf '\033[31m%s\033[0m\n' "$*"; }
bold()   { printf '\033[1m%s\033[0m\n' "$*"; }
dim()    { printf '\033[2m%s\033[0m\n' "$*"; }

confirm() {
  local prompt="$1"
  local response
  printf '%s [y/N] ' "$prompt"
  read -r response
  [[ "$response" =~ ^[Yy]$ ]]
}

# --- version helpers ---

current_version() {
  grep '^version' "$CARGO_TOML" | head -1 | sed 's/.*= *"\(.*\)"/\1/'
}

bump_patch() {
  local v="$1"
  local major minor patch
  IFS='.' read -r major minor patch <<< "$v"
  echo "${major}.${minor}.$((patch + 1))"
}

bump_minor() {
  local v="$1"
  local major minor
  IFS='.' read -r major minor _ <<< "$v"
  echo "${major}.$((minor + 1)).0"
}

bump_major() {
  local v="$1"
  local major
  IFS='.' read -r major _ _ <<< "$v"
  echo "$((major + 1)).0.0"
}

set_version() {
  local new_version="$1"
  sed -i "s/^version = \".*\"/version = \"${new_version}\"/" "$CARGO_TOML"
}

# --- git helpers ---

git_tag_exists() {
  git -C "$REPO_ROOT" tag --list "v$1" | grep -q .
}

# --- main ---

cd "$REPO_ROOT"

echo ""
bold "═══════════════════════════════════════"
bold "  llmd release wizard"
bold "═══════════════════════════════════════"
echo ""

# --- step 0: pre-flight ---

bold "Step 0 — Pre-flight checks"
echo ""

GIT_STATUS=$(git status --porcelain 2>/dev/null || echo "")
if [[ -n "$GIT_STATUS" ]]; then
  yellow "  Warning: uncommitted changes detected:"
  dim "$GIT_STATUS"
  echo ""
  if ! confirm "  Continue anyway? (uncommitted files won't affect the tag, but it's untidy)"; then
    red "Aborted. Commit or stash your changes first."
    exit 1
  fi
fi

GIT_BRANCH=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "unknown")
if [[ "$GIT_BRANCH" != "master" && "$GIT_BRANCH" != "main" ]]; then
  yellow "  Warning: you are on branch '${GIT_BRANCH}', not master/main."
  if ! confirm "  Release from this branch?"; then
    red "Aborted."
    exit 1
  fi
fi

green "  Pre-flight OK"
echo ""

# --- step 1: confirm checks passed ---

bold "Step 1 — Confirm checks"
echo ""
echo "  Have you run ./scripts/check.sh and confirmed all checks pass?"
if ! confirm "  Yes, all checks passed"; then
  yellow "  Run ./scripts/check.sh first, resolve any issues, then re-run this script."
  exit 1
fi
echo ""

# --- step 2: choose version ---

bold "Step 2 — Version"
echo ""

CURRENT=$(current_version)
SUGGEST_PATCH=$(bump_patch "$CURRENT")
SUGGEST_MINOR=$(bump_minor "$CURRENT")
SUGGEST_MAJOR=$(bump_major "$CURRENT")

echo "  Current version : ${CURRENT}"
echo ""
echo "  Suggested bumps:"
echo "    [1] patch  →  ${SUGGEST_PATCH}  (bug fix, docs, minor improvement)"
echo "    [2] minor  →  ${SUGGEST_MINOR}  (new feature, backwards-compatible)"
echo "    [3] major  →  ${SUGGEST_MAJOR}  (breaking change)"
echo "    [4] custom →  enter manually"
echo ""
printf '  Choose [1/2/3/4]: '
read -r choice

case "$choice" in
  1) NEW_VERSION="$SUGGEST_PATCH" ;;
  2) NEW_VERSION="$SUGGEST_MINOR" ;;
  3) NEW_VERSION="$SUGGEST_MAJOR" ;;
  4)
    printf '  Enter version (without v prefix): '
    read -r NEW_VERSION
    ;;
  *)
    red "  Invalid choice. Aborted."
    exit 1
    ;;
esac

echo ""
echo "  New version will be: ${NEW_VERSION}"
if ! confirm "  Confirm version ${NEW_VERSION}?"; then
  red "Aborted."
  exit 1
fi

if git_tag_exists "$NEW_VERSION"; then
  red "  Tag v${NEW_VERSION} already exists. Choose a different version."
  exit 1
fi

echo ""

# --- step 3: update Cargo.toml ---

bold "Step 3 — Update Cargo.toml"
echo ""
set_version "$NEW_VERSION"
cargo build -q 2>&1
green "  Cargo.toml updated to ${NEW_VERSION}, Cargo.lock refreshed."
echo ""

# --- step 4: final build + test ---

bold "Step 4 — Final build and test"
echo ""
echo "  Running cargo test..."
if cargo test 2>&1 | tail -5; then
  green "  Tests passed."
else
  red "  Tests failed. Fix them before releasing."
  set_version "$CURRENT"
  cargo build -q 2>&1
  exit 1
fi
echo ""

# --- step 5: commit ---

bold "Step 5 — Commit"
echo ""
git diff --stat
echo ""
COMMIT_MSG="chore: release v${NEW_VERSION}"
echo "  Commit message: \"${COMMIT_MSG}\""
if confirm "  Commit Cargo.toml and Cargo.lock?"; then
  git add Cargo.toml Cargo.lock
  git commit -m "$COMMIT_MSG"
  green "  Committed."
else
  yellow "  Skipped commit. You can commit manually before pushing."
fi
echo ""

# --- step 6: tag ---

bold "Step 6 — Tag"
echo ""
echo "  Tag: v${NEW_VERSION}"
if confirm "  Create and push tag v${NEW_VERSION}?"; then
  git tag "v${NEW_VERSION}"
  green "  Tag v${NEW_VERSION} created."

  if git remote get-url origin &>/dev/null; then
    echo "  Pushing tag to origin..."
    git push origin "v${NEW_VERSION}"
    green "  Tag pushed."
  else
    yellow "  No 'origin' remote — tag created locally only."
    yellow "  Push manually: git push origin v${NEW_VERSION}"
  fi
else
  yellow "  Skipped tagging. Tag manually: git tag v${NEW_VERSION}"
fi
echo ""

# --- step 7: push branch ---

bold "Step 7 — Push branch"
echo ""
if git remote get-url origin &>/dev/null; then
  if confirm "  Push ${GIT_BRANCH} to origin?"; then
    git push origin "$GIT_BRANCH"
    green "  Branch pushed."
  else
    yellow "  Skipped. Push manually: git push origin ${GIT_BRANCH}"
  fi
else
  yellow "  No remote configured. Skipping push."
fi
echo ""

# --- step 8: publish ---

bold "Step 8 — Publish to crates.io"
echo ""
echo "  This will publish llmd v${NEW_VERSION} publicly."
echo "  Crates.io releases are permanent (you can yank but not delete)."
echo ""
if confirm "  Publish llmd v${NEW_VERSION} to crates.io now?"; then
  echo ""
  echo "  Publishing..."
  if cargo publish; then
    echo ""
    green "  ┌─────────────────────────────────────────────┐"
    green "  │  llmd v${NEW_VERSION} published to crates.io!  │"
    green "  └─────────────────────────────────────────────┘"
    echo ""
    echo "  View at: https://crates.io/crates/llmd"
    echo "  Install: cargo install llmd"
  else
    red "  cargo publish failed. Check the output above."
    exit 1
  fi
else
  yellow "  Skipped publish. Run manually: cargo publish"
fi

echo ""
bold "═══════════════════════════════════════"
bold "  Release complete: v${NEW_VERSION}"
bold "═══════════════════════════════════════"
echo ""
dim "  Next steps:"
dim "  - Create a GitHub release at https://github.com/simulacralabs/llmd/releases/new"
dim "  - Tag: v${NEW_VERSION}"
dim "  - Verify: cargo install llmd in a clean environment"
echo ""
