# Changelog

## 0.4.2 (2026-06-04)

### Security (7 vulnerabilities fixed)

- **VULN-001** — Removed unmaintained `atty` dependency; migrated to `std::io::IsTerminal` (Rust 1.70+)
- **VULN-002** — Fixed ANSI escape injection via literal text; `print_str()` now strips raw `\x1b[` sequences
- **VULN-003** — Replaced regex-based `strip_ansi_escapes` with hand-written FSM covering OSC/DCS sequences
- **VULN-004** — Made `Live` struct thread-safe with `Arc<Mutex<Option<DynRenderable>>>` + `Arc<AtomicUsize>`
- **VULN-006** — Fixed `get_console()` poisoned mutex panic; now uses `unwrap_or_else(|e| e.into_inner())`
- **VULN-007** — Sanitized `$PAGER` command injection via `split_pager_command()` (program + args separation)
- **VULN-009** — Removed regex recompilation DoS vector in pager; uses FSM-based ANSI stripping

### Fixed (10 bugs resolved)

- **BUG-001** — Removed duplicate `CONCEAL` emission in `Style::to_ansi()`
- **BUG-002** — Fixed `Color::parse` misdetecting 6-char names like "purple" as hex; added `is_ascii_hexdigit()` guard
- **BUG-003** — `Progress::update()` now clamps completed to `[0.0, total]` to prevent NaN/Inf
- **BUG-005** — Fixed `ProgressBar::render()` integer underflow with `saturating_sub(1)`
- **BUG-006** — `Live::get_renderable()` and `renderable()` return `Option<DynRenderable>` instead of panicking
- **BUG-007** — Added `MAX_MARKUP_DEPTH = 100` guard to markup parser preventing stack overflow from deep nesting
- **BUG-008** — `Console::end_capture()` returns `Result<Capture, CaptureError>` instead of panicking
- **BUG-009** — Fixed `rgb_to_8bit` pure black `(0,0,0)` mapping to index 0 (standard black) instead of 16 (grey0)
- **BUG-010** — `Style::from_str()` fixed "not bold" with space handling; rewrite with `not` prefix token support

### Fixed (code quality)

- **VULN-004** — `ThemeContext` hardened with `PhantomData<*const ()>` for explicit `!Send + !Sync`
- **VULN-007** — HTML/SVG export safe from template injection; all values escaped, `{code}` replaced first
- **IMP-002** — Removed `once_cell` dependency; migrated to `std::sync::LazyLock` (Rust 1.80+)
- **IMP-005** — Added 11 zero-allocation ANSI constants to `control` module; replaced all hardcoded `\x1b[...` strings in `console.rs` and `live.rs`
- `clear_live()` redundant if/else branches removed
- Doctest fixes: `ignore` → `no_run` for 5 doc-tests; cfg-gated markdown example for `--no-default-features`
- Style negation regression tests added: "not bold", `!italic`, `nounderline`, `on red`

### Changed

- 51 source modules (was 48), ~25,569 lines (was ~21,400), 742+ tests (was 778)
- 0 Clippy warnings, 0 fmt issues, 0 doc warnings
- CI fully green: Build (×3 OS) · Test (all + no-default) · Lint (fmt + clippy) · Docs · Security Audit
- `Full_audit.md` — comprehensive 4-part audit report included in repo
- `RAW_URLS_AND_AI_PROMPTS.md` — 6 AI-ready analysis prompts with all source file URLs

## 0.4.1 (2026-06-03)

### Fixed
- **ThemeContext lifetime safety** — added `PhantomData<&'a mut Console>` to `ThemeContext` to properly track borrow lifetimes and prevent use-after-free with the internal raw pointer
- **Syntax theme fallback** — `Syntax` render no longer panics when the requested theme is missing; falls back to `base16-ocean.dark` instead of indexing into an absent key
- **Layout divide-by-zero** — `Layout::render()` skips empty splits (`total_size == 0` or no children) instead of dividing by zero
- **Text::stylize out-of-bounds** — `end` parameter is now clamped to `self.plain.len()` to prevent span index out-of-bounds

## 0.4.0 (2026-06-02)

### Fixed
- **Box styles match Python Rich exactly** — MINIMAL, MINIMAL_HEAVY, MINIMAL_DOUBLE_HEAD, SIMPLE_HEAD, SQUARE_DOUBLE_HEAD, ASCII, ASCII2, ASCII_DOUBLE_HEAD all corrected to match Python Rich 14.x box definitions
- **Rowspan vertical bar bleed** — consecutive rowspan columns now merged into one span; no stray `│` inside spanned cells
- **Colspan bottom border** — `compute_span_widths()` respects cell colspan so dividers only appear at real column boundaries
- **Colspan span width** — includes internal separator widths so all rows have consistent total width regardless of colspan
- **ASCII mode ANSI leak** — `\x1b[...m` sequences stripped when `ascii_only=true`
- **Terminal wrapping** — `max_width = terminal_width - 1` prevents edge-of-screen line wrapping
- **`cell_length()` ANSI handling** — ANSI escape sequences excluded from visible width measurement, fixing right border alignment in styled Panels
- **Panel/Table left border** — uses `mid_left` instead of `mid_vertical` for asymmetric box styles (HEAVY_EDGE, DOUBLE_EDGE)
- **Panel border ANSI batching** — repeated horizontal chars batched under single ANSI wrap instead of per-char wrapping
- **Table border ANSI batching** — same batching applied to table horizontal repeats
- **Terminal resize** — `Console::println()` re-detects terminal size on each call so output adapts to window resizes
- **Doc examples** — fixed `add_row_str`, `create_writer`, `export_svg` signatures in lib.rs, table.rs, live.rs

### Added
- `BoxStyle::has_visible_edges()` — returns true if box has non-space corners
- **Edge-less Panel rendering** — Panels with edge-less box styles (SIMPLE, MINIMAL, MARKDOWN, etc.) render content without invisible borders
- `Table::compute_span_widths()` — computes effective column widths respecting colspan
- `Table::compute_bottom_widths()` — uses last row's cell layout for bottom border
- `Console::refresh_size()` — re-detects terminal dimensions before each render
- 150 exhaustive box/table tests (`tests/box_table_exhaustive.rs`) covering all 18 box styles × every feature
- Visual demo (`examples/view_all.rs`) rendering all box styles, panels, tables, and features

### Changed
- `Table::render_cell_line_with_rowspan` — left edge uses `mid_left`
- `Panel::render_pad_line` — left edge uses `mid_left`
- Version bumped to 0.4.0

## 0.3.0 (2025-??-??)
- Initial public release
