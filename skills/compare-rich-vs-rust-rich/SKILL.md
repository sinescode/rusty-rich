---
name: compare-rich-vs-rust-rich
version: 1.0.0
description: Deep comparison of Python Rich vs Rust-Rich to find logic/power gaps. Use this whenever the user wants to compare two codebases, find missing features, audit a port/reimplementation, or analyze parity between a source library and its Rust translation. Triggers on phrases like "compare", "gap analysis", "find what's missing", "parity check", "port audit", "what's different between", "logic gaps".
---

# compare-rich-vs-rust-rich

Deep comparison workflow — maps two codebases, finds gaps, and produces a structured report.

## What it does

Runs a multi-phase workflow that:
1. **Map source** — catalog all modules, classes, functions, capabilities
2. **Map target** — catalog all equivalent modules with the same detail
3. **Compare side-by-side** — classify every feature as IMPLEMENTED / PARTIAL / MISSING
4. **Synthesize report** — executive summary, severity tables, scorecard, recommendations

Each gap classified by severity (HIGH/MEDIUM/LOW) and type (LOGIC vs POWER).

## How to use

Call the bundled workflow:
```
Workflow({ scriptPath: "<skill-dir>/workflow.js" })
```

For different libraries, edit the agent prompts to point at new source/target paths.

## Output

- Executive Summary
- High/Medium/Low-Severity Gap tables
- Logic Gaps (behavioral differences)
- Feature Completeness Scorecard (percentage per module, letter grades)
- Tiered implementation recommendations
