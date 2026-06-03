# Changelog

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
