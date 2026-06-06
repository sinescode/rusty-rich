---
name: run-rusty-rich
description: Build, test, and drive rusty-rich (Rust Rich port) via GitHub Actions CI. Use when asked to run, start, build, test, verify, or check CI for this crate.
---

# run-rusty-rich

rusty-rich is a Rust library crate — a high-fidelity port of Python's [Rich](https://github.com/Textualize/rich) for terminal text formatting, styling, and rendering. It has no persistent server or GUI. The "run" experience is **CI-driven**: push changes, let GitHub Actions build/test/lint/diff across three OSes, and inspect the results.

**Primary driver:** `.claude/skills/run-rusty-rich/ci-driver.sh` — a bash wrapper around `gh` CLI that triggers, monitors, and reports on CI workflows.

## Prerequisites

```bash
# gh CLI must be authenticated
gh auth status
```

No local Rust toolchain required — all build/test/lint happens on GitHub Actions runners.

## Build & Test (agent path — CI-driven)

The driver script wraps every CI operation:

```bash
# Trigger CI on the current branch
.claude/skills/run-rusty-rich/ci-driver.sh run

# Watch a CI run (blocks until complete, shows final results)
.claude/skills/run-rusty-rich/ci-driver.sh watch

# Quick status of the latest CI run
.claude/skills/run-rusty-rich/ci-driver.sh status

# Show per-job results for a specific run
.claude/skills/run-rusty-rich/ci-driver.sh results 26939266143

# Show test output from a CI run
.claude/skills/run-rusty-rich/ci-driver.sh test 26939266143
```

### Without the driver (raw gh commands)

```bash
# Trigger CI
gh workflow run ci.yml --repo sinescode/rusty-rich --ref master

# Watch latest run
gh run watch --repo sinescode/rusty-rich

# View results
gh run view --repo sinescode/rusty-rich --json status,conclusion,jobs \
  --jq '.jobs[] | "\(.name): \(.conclusion)"'

# View logs (grep for test results)
gh run view --repo sinescode/rusty-rich --log | grep -E '(test result:|error\[)'
```

## What CI Checks

The `ci.yml` workflow runs on every push/PR to master:

| Job | Runs on | What it checks |
|-----|---------|----------------|
| Build | ubuntu, windows, macos | `cargo build --workspace` + `--no-default-features` |
| Test | ubuntu | `cargo test --workspace` (both feature sets) |
| Lint | ubuntu | `cargo fmt --check` + `cargo clippy -- -D warnings` |
| Security Audit | ubuntu | `cargo deny check advisories` + `bans licenses sources` |
| Docs | ubuntu | `cargo doc --no-deps --document-private-items` + `RUSTDOCFLAGS="-D warnings"` |

A separate `security-audit.yml` runs on Cargo.toml/Lock changes and weekly via cron.

### Expected test counts (healthy CI)

From the last passing run (26939266143):

```
all features:     489 unit + 103 battle + 150 box_table + 56 doc = 798 tests
no default feats: 464 unit +  93 battle + 150 box_table + 54 doc = 761 tests
```

Doc test count varies by feature set (syntax-highlighting and markdown gates gate some doc tests).

## Run Examples (human path — secondary)

If a Rust toolchain is available locally:

```bash
# Comprehensive rendering demo (all box styles, panels, tables)
cargo run --example view_all

# Interactive feature tour
cargo run --example demo
```

Both produce ANSI terminal output. Pipe to a file to review: `cargo run --example view_all > /tmp/output.txt`.

## Gotchas

- **`cargo fmt` is enforced.** The lint job fails on any formatting difference. Run `cargo fmt --all` before pushing, or the CI *will* reject it.
- **Clippy warnings are errors.** `-D warnings` means any clippy lint triggers a CI failure.
- **Doc tests are gated by features.** Some doc tests only compile with `syntax-highlighting` or `markdown` enabled. The CI runs both `--all-features` and `--no-default-features` test suites, catching missing `#[cfg]` guards.
- **Doc links must be exact.** `[`module`]` auto-links work only when the path resolves. Function references need `()` — e.g., `[`diagnose()`]` not `[`diagnose`]`. Invalid doc links fail the Docs job.
- **Security audit runs on schedule too.** A weekly `cargo-audit` + `cargo-deny` run catches new advisories even when no code changes.
- **36 compile errors is a real CI failure.** The most recent run (27052893069) had `E0433`, `E0592`, `E0615`, `E0277`, `E0599`, `E0061`, `E0308`, `E0369` — typical of a large refactor without corresponding test/import updates.

## Troubleshooting

| Symptom | Fix |
|---------|-----|
| Lint job fails with diff output | Run `cargo fmt --all` locally and push |
| Clippy fails with warning | Fix the clippy warning; `cargo clippy -- -D warnings` to repro locally |
| Docs job fails with broken link | Check `cargo doc` output; fix module/function path references |
| Test count changes unexpectedly | Feature-gated doc tests may have been affected; check `#[cfg(feature = "...")]` guards |
| Build fails on one OS only | Check for OS-specific code (file paths, platform deps) |
| `cargo deny` fails | A dependency has a new advisory; update or add an exception to `deny.toml` |
| `gh run watch` times out | CI run is slow (~5-8 min); use `gh run view --log` to check progress instead |
