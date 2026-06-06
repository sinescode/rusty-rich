#!/usr/bin/env bash
# ci-driver.sh — Dead-simple CI driver for rusty-rich.
#
# Just run it. No arguments needed.
#
#   ./ci-driver.sh            Show dashboard (status + jobs + recent history)
#   ./ci-driver.sh run        Trigger CI on the current branch
#   ./ci-driver.sh watch [id] Watch a CI run live, then show dashboard
#   ./ci-driver.sh log [id]   Show test output from a CI run
#   ./ci-driver.sh why [id]   Show WHY the latest CI failed (errors)
#
# Requires: gh CLI authenticated. That's it.

set -euo pipefail
REPO="sinescode/rusty-rich"
WF="ci.yml"
SELF="${BASH_SOURCE[0]:-$0}"

# ── helpers ────────────────────────────────────────────────────────────────

BOLD=$'\033[1m'; GREEN=$'\033[1;32m'; RED=$'\033[1;31m'; YELLOW=$'\033[1;33m'
CYAN=$'\033[1;36m'; DIM=$'\033[2m'; RESET=$'\033[0m'

# Use ESC byte for ANSI stripping
ESC=$(printf '\033')

say() { printf "%s%s%s\n" "${2:-}" "$1" "$RESET"; }

latest_run() { gh run list -R "$REPO" -w "$WF" -L1 --json databaseId -q '.[0].databaseId'; }

die() { say "$1" "$RED"; exit "${2:-1}"; }

check_prereqs() {
  if ! gh auth status &>/dev/null; then
    die "gh CLI not authenticated. Run: gh auth login" 2
  fi
}

# ── dashboard (default, no args) ───────────────────────────────────────────

cmd_dashboard() {
  check_prereqs

  local id branch title conclusion
  id=$(gh run list -R "$REPO" -w "$WF" -L1 --json databaseId,headBranch,displayTitle,conclusion -q '
    .[0] | "\(.databaseId)|\(.headBranch)|\(.displayTitle)|\(.conclusion // "pending")"')
  IFS='|' read -r _rid branch title conclusion <<< "$id"

  # -- header --
  echo ""
  say " rusty-rich CI" "$BOLD$CYAN"
  echo ""

  # -- latest run status badge --
  case "$conclusion" in
    success) say "  ✓  Latest CI passed" "$GREEN" ;;
    failure) say "  ✗  Latest CI failed" "$RED" ;;
    *)       say "  ◌  Latest CI $conclusion" "$YELLOW" ;;
  esac
  echo "  ${DIM}branch:${RESET} $branch  ${DIM}title:${RESET} ${title:0:60}"
  echo "  ${DIM}url:${RESET}    https://github.com/$REPO/actions/runs/$_rid"
  echo ""

  # -- job summary --
  say " Jobs" "$BOLD"
  gh run view "$_rid" -R "$REPO" --json jobs -q '
    .jobs[] | "  \(if .conclusion == "success" then "✓" elif .conclusion == "failure" then "✗" else "◌" end)  \(.name)"'
  echo ""

  # -- failure insight (if CI failed) --
  if [ "$conclusion" = "failure" ]; then
    say " Why it failed" "$BOLD$RED"
    gh run view "$_rid" -R "$REPO" --log 2>/dev/null \
      | sed -E "s/(\^\[\[|\x1b\[)[0-9;]*m//g" \
      | grep -oP 'error\[[^]]+\]:[^"]*' \
      | sed 's/^[^e]*error/error/' \
      | sort -u | head -10 \
      | while read -r line; do echo "  ${RED}→${RESET} $line"; done
    echo ""
    echo "  ${DIM}Full details:${RESET} $SELF why $_rid"
    echo ""
  fi

  # -- quick actions --
  say " Quick actions" "$BOLD"
  echo "  ${DIM}$SELF${RESET}          show this dashboard"
  echo "  ${DIM}$SELF run${RESET}      trigger CI on current branch"
  echo "  ${DIM}$SELF watch${RESET}    watch latest CI run live"
  echo "  ${DIM}$SELF log${RESET}      show test output from latest CI"
  echo "  ${DIM}$SELF why${RESET}      show WHY the latest CI failed"
  echo ""
}

# ── run ────────────────────────────────────────────────────────────────────

cmd_run() {
  check_prereqs
  local branch
  branch=$(git branch --show-current)
  say "Triggering CI for branch: $branch" "$CYAN"
  gh workflow run "$WF" -R "$REPO" --ref "$branch"
  sleep 3
  local id
  id=$(latest_run)
  say "CI triggered! https://github.com/$REPO/actions/runs/$id" "$GREEN"
  echo ""
  echo "  To watch: $SELF watch $id"
}

# ── watch ──────────────────────────────────────────────────────────────────

cmd_watch() {
  check_prereqs
  local id="${1:-$(latest_run)}"
  say "Watching run $id ..." "$CYAN"
  gh run watch "$id" -R "$REPO" --exit-status 2>/dev/null || true
  echo ""
  cmd_dashboard
}

# ── log ────────────────────────────────────────────────────────────────────

cmd_log() {
  check_prereqs
  local id="${1:-$(latest_run)}"
  say "Test output — run $id" "$BOLD"
  echo ""
  gh run view "$id" -R "$REPO" --log 2>/dev/null \
    | grep -E '(test result:|running [0-9]+ tests|Doc-tests|error\[|Compiling rusty-rich)' \
    | grep -v '^Lint' | tail -25
}

# ── why (what failed?) ──────────────────────────────────────────────────────

cmd_why() {
  check_prereqs
  local id="${1:-$(latest_run)}"

  local conclusion
  conclusion=$(gh run view "$id" -R "$REPO" --json conclusion -q '.conclusion')

  say "Failure diagnosis — run $id" "$BOLD"
  echo ""

  if [ "$conclusion" = "success" ]; then
    say "  ✓ CI passed — nothing to diagnose." "$GREEN"
    return
  fi

  # -- which jobs failed --
  say " Failed jobs" "$RED"
  gh run view "$id" -R "$REPO" --json jobs -q '
    .jobs[] | select(.conclusion == "failure") | "  ✗  \(.name)"'
  echo ""

  # -- extract unique errors from logs, strip ANSI --
  say " Errors (unique)" "$RED"
  local errors
  errors=$(gh run view "$id" -R "$REPO" --log 2>/dev/null \
    | grep -oP 'error\[[^]]+\][^"]*' \
    | sed "s/${ESC}\[[0-9;]*m//g; s/^[^e]*error/error/" \
    | sort -u | head -12 || true)

  if [ -z "$errors" ]; then
    echo "  (no Rust compiler errors — may be fmt/clippy/doc warning)"
  else
    echo "$errors" | while read -r line; do
      echo "  ${RED}→${RESET} $line"
    done
  fi
  echo ""

  # -- fmt specific --
  if gh run view "$id" -R "$REPO" --log 2>/dev/null | grep -q 'diff --git'; then
    say " Lint: cargo fmt diff detected" "$YELLOW"
    echo "  Run: cargo fmt --all && git push"
    echo ""
  fi

  echo "  ${DIM}Full log:${RESET} $SELF log $id"
  echo "  ${DIM}CI URL:${RESET}   https://github.com/$REPO/actions/runs/$id"
  echo ""
}

# ── dispatch ───────────────────────────────────────────────────────────────

check_prereqs

case "${1:-}" in
  run)     cmd_run ;;
  watch)   cmd_watch "${2:-}" ;;
  log)     cmd_log "${2:-}" ;;
  why)     cmd_why "${2:-}" ;;
  -h|--help|help)
    echo "Usage: $SELF [run|watch|log|why]"
    echo ""
    echo "No arguments → dashboard (CI status + jobs + failure insight + actions)"
    echo "  run         Trigger CI on the current branch"
    echo "  watch [id]  Watch a CI run, then show dashboard"
    echo "  log [id]    Show test output from a CI run"
    echo "  why [id]    Diagnose WHY the CI failed (errors, fmt diffs)"
    ;;
  "")      cmd_dashboard ;;
  *)       die "Unknown: $1. Try: $SELF" ;;
esac
