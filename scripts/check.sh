#!/usr/bin/env bash
# check.sh — pre-release readiness check for llmd
#
# Runs: build, tests, clippy, fmt check, publish dry-run.
# On success: prints a green summary and exits 0.
# On failure: writes check-report.md and exits 1.
#
# Usage: ./scripts/check.sh [--report-only]
#   --report-only  Always write check-report.md even when all checks pass

set -uo pipefail

REPORT_ONLY=false
for arg in "$@"; do
  [[ "$arg" == "--report-only" ]] && REPORT_ONLY=true
done

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPORT_FILE="$REPO_ROOT/check-report.md"
CURRENT_VERSION=$(grep '^version' "$REPO_ROOT/Cargo.toml" | head -1 | sed 's/.*= *"\(.*\)"/\1/')
GIT_BRANCH=$(git -C "$REPO_ROOT" rev-parse --abbrev-ref HEAD 2>/dev/null || echo "unknown")
GIT_STATUS=$(git -C "$REPO_ROOT" status --porcelain 2>/dev/null || echo "")
TIMESTAMP=$(date -u +"%Y-%m-%d %H:%M UTC")

# Parallel arrays: check names, pass/fail results, captured output on failure
CHECK_NAMES=()
CHECK_RESULTS=()   # "ok" or "fail"
CHECK_OUTPUTS=()   # empty string on pass, error output on fail

run_check() {
  local name="$1"
  local cmd="$2"
  local output
  CHECK_NAMES+=("$name")
  if output=$(eval "$cmd" 2>&1); then
    echo "  [ok]   $name"
    CHECK_RESULTS+=("ok")
    CHECK_OUTPUTS+=("")
  else
    echo "  [FAIL] $name"
    CHECK_RESULTS+=("fail")
    CHECK_OUTPUTS+=("$output")
  fi
}

# --- run checks ---

echo ""
echo "llmd pre-release check — v${CURRENT_VERSION}"
echo "Branch: ${GIT_BRANCH}  |  ${TIMESTAMP}"
echo "──────────────────────────────────────────────"

cd "$REPO_ROOT"

run_check "cargo build"          "cargo build 2>&1"
run_check "cargo test"           "cargo test 2>&1"
run_check "cargo clippy -D warn" "cargo clippy -- -D warnings 2>&1"
run_check "cargo fmt --check"    "cargo fmt --check 2>&1"
run_check "publish dry-run"      "cargo publish --dry-run --allow-dirty 2>&1"

echo "──────────────────────────────────────────────"

# --- count results ---

FAIL=0
PASS=0
for r in "${CHECK_RESULTS[@]}"; do
  if [[ "$r" == "ok" ]]; then
    PASS=$((PASS + 1))
  else
    FAIL=$((FAIL + 1))
  fi
done

# --- git warnings (informational, not counted as failures) ---

GIT_WARNINGS=()
if [[ -n "$GIT_STATUS" ]]; then
  GIT_WARNINGS+=("**Uncommitted changes** — commit everything before publishing.\n\n\`\`\`\n${GIT_STATUS}\n\`\`\`")
  echo "  [warn] Uncommitted changes in working tree"
fi
if ! git -C "$REPO_ROOT" remote get-url origin &>/dev/null; then
  GIT_WARNINGS+=("**No git remote** — add one before pushing:\n\n\`\`\`sh\ngit remote add origin https://github.com/you/llmd\n\`\`\`")
  echo "  [warn] No git remote 'origin' configured"
fi

# --- outcome ---

echo ""
if [[ $FAIL -eq 0 ]] && ! $REPORT_ONLY; then
  echo "All ${PASS} checks passed. Ready to release."
  echo "Run ./scripts/release.sh to publish."
  echo ""
  exit 0
fi

# --- write report ---

{
  echo "# llmd Pre-Release Check Report"
  echo ""
  echo "**Generated:** ${TIMESTAMP}"
  echo "**Version:** \`${CURRENT_VERSION}\`"
  echo "**Branch:** \`${GIT_BRANCH}\`"
  if [[ $FAIL -eq 0 ]]; then
    echo "**Result:** All ${PASS} checks passed"
  else
    echo "**Result:** ${FAIL} check(s) failed, ${PASS} passed"
  fi
  echo ""
  echo "---"
  echo ""
  echo "## Check Summary"
  echo ""
  echo "| Check | Result |"
  echo "|-------|--------|"
  for i in "${!CHECK_NAMES[@]}"; do
    if [[ "${CHECK_RESULTS[$i]}" == "ok" ]]; then
      echo "| \`${CHECK_NAMES[$i]}\` | ✅ Passed |"
    else
      echo "| \`${CHECK_NAMES[$i]}\` | ❌ Failed |"
    fi
  done
  echo ""

  if [[ ${#GIT_WARNINGS[@]} -gt 0 ]]; then
    echo "---"
    echo ""
    echo "## Git Warnings"
    echo ""
    echo "These are not blocking failures but should be resolved before publishing:"
    echo ""
    for w in "${GIT_WARNINGS[@]}"; do
      echo -e "- $w"
      echo ""
    done
  fi

  if [[ $FAIL -gt 0 ]]; then
    echo "---"
    echo ""
    echo "## Failed Checks — Details"
    echo ""
    echo "Each section below contains the full compiler/tool output for a failed check."
    echo "Provide this report to an agent to resolve the issues."
    echo ""
    for i in "${!CHECK_NAMES[@]}"; do
      if [[ "${CHECK_RESULTS[$i]}" != "ok" ]]; then
        echo "### \`${CHECK_NAMES[$i]}\`"
        echo ""
        echo "\`\`\`"
        echo "${CHECK_OUTPUTS[$i]}"
        echo "\`\`\`"
        echo ""
      fi
    done
    echo "---"
    echo ""
    echo "## Resolution"
    echo ""
    echo "1. Fix each issue listed above."
    echo "2. Re-run \`./scripts/check.sh\` to verify."
    echo "3. Once all checks pass, run \`./scripts/release.sh\` to publish."
  fi

} > "$REPORT_FILE"

if [[ $FAIL -gt 0 ]]; then
  echo "${FAIL} check(s) failed. Full details written to: check-report.md"
  echo ""
  exit 1
else
  echo "All ${PASS} checks passed. Report written to: check-report.md (--report-only)"
  echo ""
  exit 0
fi
