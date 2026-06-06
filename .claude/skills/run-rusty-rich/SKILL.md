---
name: run-rusty-rich
description: Run, build, test, or check CI status for rusty-rich. Use when asked to run, start, build, test, verify, or check CI for this Rust crate.
---

# run-rusty-rich

rusty-rich is a Rust library crate — a Rich-text terminal formatting port. No GUI, no server. Everything goes through **GitHub Actions CI** (no local Rust required).

**Driver:** `.claude/skills/run-rusty-rich/ci-driver.sh` — just run it.

## Prerequisites

```bash
gh auth status
```

## Run

```bash
# Dashboard — CI status, jobs, actions (default, no args needed)
.claude/skills/run-rusty-rich/ci-driver.sh

# Trigger CI on the current branch
.claude/skills/run-rusty-rich/ci-driver.sh run

# Watch a run live, then show dashboard
.claude/skills/run-rusty-rich/ci-driver.sh watch

# Show test output from latest (or specific) CI run
.claude/skills/run-rusty-rich/ci-driver.sh log
.claude/skills/run-rusty-rich/ci-driver.sh log 26939266143
```

## Examples (human path)

If you have a Rust toolchain:

```bash
cargo run --example view_all    # all box styles, panels, tables
cargo run --example demo        # interactive feature tour
```

## Gotchas

- **`cargo fmt` is enforced** — any formatting diff fails lint. Run `cargo fmt --all` before pushing.
- **Clippy warnings are errors** — `-D warnings` in CI means any lint fails the build.
- **Doc links must be exact** — `[diagnose()]` not `[diagnose]`. Invalid doc links fail the Docs job.
- **Feature-gated doc tests** — doc tests behind `#[cfg(feature = "...")]` guards change count between `--all-features` (56 tests) and `--no-default-features` (54 tests). CI catches missing guards.
- **The latest CI is currently failing** (run 27056312593) — `cargo fmt` diff in the `compare_rich.py` removal commit. A clean `cargo fmt --all` + push should fix it.

## Troubleshooting

| Symptom | Fix |
|---------|-----|
| Lint fails with diff | `cargo fmt --all` and push |
| Clippy fails | `cargo clippy -- -D warnings` locally |
| Docs fail with broken link | Fix module/function path in doc comment |
| Build fails on one OS | Check for OS-specific code (paths, platform deps) |
| `cargo deny` fails | New advisory — update dep or add exception to `deny.toml` |
| `gh run watch` hangs | CI takes ~5-8 min; use `gh run view --log` to check progress |
