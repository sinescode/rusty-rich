#!/usr/bin/env bash
# ci-driver.sh — Trigger, monitor, and report on rusty-rich CI workflows.
#
# Usage:
#   ./ci-driver.sh run              Trigger a CI run on the current branch
#   ./ci-driver.sh watch [run-id]   Watch the latest (or specific) CI run
#   ./ci-driver.sh results [run-id] Show test/build results for a CI run
#   ./ci-driver.sh test [filter]    Show test output (optionally filter by name)
#   ./ci-driver.sh status           Quick status of the latest CI run
#
# Requires: gh CLI authenticated, in the rusty-rich repo.
# No local cargo — all build/test/lint through GitHub Actions.

set -euo pipefail

REPO="sinescode/rusty-rich"
WF_CI="ci.yml"

# ── helpers ────────────────────────────────────────────────────────────────

_color() {
  case "$1" in
    green)  printf '\033[1;32m%s\033[0m\n' "$2" ;;
    red)    printf '\033[1;31m%s\033[0m\n' "$2" ;;
    yellow) printf '\033[1;33m%s\033[0m\n' "$2" ;;
    cyan)   printf '\033[1;36m%s\033[0m\n' "$2" ;;
    bold)   printf '\033[1m%s\033[0m\n' "$2" ;;
    *)      printf '%s\n' "$2" ;;
  esac
}

_latest_run_id() {
  gh run list --repo "$REPO" --workflow "$WF_CI" --limit 1 --json databaseId --jq '.[0].databaseId'
}

# ── commands ───────────────────────────────────────────────────────────────

cmd_run() {
  local branch
  branch=$(git branch --show-current)
  _color cyan "Triggering CI for branch: $branch"
  gh workflow run "$WF_CI" --repo "$REPO" --ref "$branch"
  sleep 3
  local id
  id=$(_latest_run_id)
  _color green "CI triggered: https://github.com/$REPO/actions/runs/$id"
  echo "Run: $0 watch $id"
}

cmd_watch() {
  local id="${1:-$(_latest_run_id)}"
  _color cyan "Watching CI run: $id"
  gh run watch "$id" --repo "$REPO" --exit-status || true
  cmd_results "$id"
}

cmd_results() {
  local id="${1:-$(_latest_run_id)}"
  _color bold "=== CI Run $id ==="
  echo ""
  gh run view "$id" --repo "$REPO" --json name,status,conclusion,displayTitle,headBranch \
    --jq '"  Branch: \(.headBranch)\n  Title:  \(.displayTitle)\n  Status: \(.status) / \(.conclusion)"'
  echo ""
  _color bold "--- Jobs ---"
  gh run view "$id" --repo "$REPO" --json jobs --jq '
    .jobs[] | "  \(.name): \(.status) → \(.conclusion // "pending")"'
}

cmd_test() {
  local id="${1:-$(_latest_run_id)}"
  _color bold "=== Test Output (run $id) ==="
  gh run view "$id" --repo "$REPO" --log 2>/dev/null | \
    grep -E '(test result:|running [0-9]+ tests|Doc-tests)' | \
    grep -v '^Lint' | \
    tail -20
}

cmd_status() {
  local id
  id=$(_latest_run_id)
  local conclusion
  conclusion=$(gh run view "$id" --repo "$REPO" --json conclusion --jq '.conclusion')
  local name
  name=$(gh run view "$id" --repo "$REPO" --json displayTitle --jq '.displayTitle' | cut -c1-60)

  case "$conclusion" in
    success) _color green "✓ CI passed — $name" ;;
    failure) _color red   "✗ CI failed — $name" ;;
    *)       _color yellow "◌ CI $conclusion — $name" ;;
  esac
  echo "  https://github.com/$REPO/actions/runs/$id"
}

# ── dispatch ───────────────────────────────────────────────────────────────

case "${1:-}" in
  run)     shift; cmd_run "$@" ;;
  watch)   shift; cmd_watch "$@" ;;
  results) shift; cmd_results "$@" ;;
  test)    shift; cmd_test "$@" ;;
  status)  shift; cmd_status "$@" ;;
  *)
    echo "Usage: $0 {run|watch|results|test|status} [run-id]"
    echo ""
    echo "Commands:"
    echo "  run              Trigger CI on the current branch"
    echo "  watch [run-id]   Watch a CI run, then show results"
    echo "  results [run-id] Show job status for a CI run"
    echo "  test [run-id]    Show test output from a CI run"
    echo "  status           Quick pass/fail status of latest CI"
    exit 1
    ;;
esac
