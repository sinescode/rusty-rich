# Public Types Reference

This document lists every public struct, enum, trait, and type alias in `rusty_rich`,
organized by module. Each entry includes a brief description and key fields or methods.

---

## Core Modules

### `cells` — Unicode cell-width helpers

| Type | Kind | Description |
|------|------|-------------|
| `cell_len(text: &str) -> usize` | free fn | Total Unicode display width of `text` (handles CJK, emoji). |
| `get_character_cell_size(ch: char) -> usize` | free fn | Cell width of a single character (0, 1, or 2). |
| `set_cell_size(text: &str, target: usize) -> String` | free fn | Pad or crop `text` to exactly `target` cells. |
| `chop_cells(text: &str, width: usize) -> Vec<String>` | free fn | Split `text` into lines each fitting within `width` cells. |
| `split_text(text: &str, offset: usize) -> (String, String)` | free fn | Split `text` at a cell offset. |

---

### `color` — Color types

#### `ColorTriplet`
- **Description**: An RGB color triplet.
- **Kind**: `struct` (`Debug, Clone, Copy, PartialEq, Eq, Hash, Default`)
- **Fields**: `red: u8`, `green: u8`, `blue: u8`
- **Methods**:
  - `const fn new(red, green, blue) -> Self`
- **Display**: hex format `#rrggbb`.

#### `ColorSystem`
- **Description**: What the terminal supports.
- **Kind**: `enum` (`Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash`)
- **Variants**:
  - `Standard` — 3-bit / 4-bit ANSI (8/16 colors)
  - `EightBit` — 8-bit (256 colors)
  - `TrueColor` — 24-bit true color

#### `ColorType`
- **Description**: How the color value is stored internally.
- **Kind**: `enum` (`Debug, Clone, Copy, PartialEq, Eq, Hash`)
- **Variants**: `Default`, `Standard`, `EightBit`, `TrueColor`

#### `Color`
- **Description**: A terminal color — default (inherit), standard ANSI name, 8-bit palette index, or 24-bit RGB triple.
- **Kind**: `struct` (`Debug, Clone, Copy, PartialEq, Eq, Hash`)
- **Key methods**:
  - `const fn default() -> Self`
  - `fn from_ansi_name(name: &str) -> Option<Self>`
  - `fn from_8bit(n: u8) -> Self`
  - `fn from_rgb(r, g, b) -> Self`
  - `fn from_hex(hex: &str) -> Result<Self, ColorParseError>`
  - `fn parse(s: &str) -> Result<Self, ColorParseError>` — parse from name/hex/`color<N>`/`default`
  - `fn is_default() -> bool`
  - `fn get_truecolor(theme) -> (u8, u8, u8)`
  - `fn downgrade(system: ColorSystem) -> Self`
  - `fn name_to_index(name: &str) -> Option<u8>` (static)

#### `ColorParseError`
- **Description**: Errors from `Color::parse`.
- **Kind**: `enum` (`Debug, Clone`)
- **Variants**: `UnknownName(String)`, `InvalidHex(String)`

#### `TerminalTheme`
- **Description**: Terminal default colors for blending/downgrade.
- **Kind**: `struct` (`Debug, Clone, Copy`)
- **Fields**: `foreground_color: (u8, u8, u8)`, `background_color: (u8, u8, u8)`

#### `rgb_to_8bit(r, g, b) -> u8`
- **Free fn**: Convert RGB to nearest 8-bit palette index.

#### `rgb_to_standard(r, g, b) -> u8`
- **Free fn**: Convert RGB to nearest standard (16) ANSI color.

#### `blend_rgb(color1, color2, cross_fade) -> (u8, u8, u8)`
- **Free fn**: Blend two RGB colors.

#### `blend_colors(color1, color2, cross_fade, theme) -> Color`
- **Free fn**: Blend two Colors.

#### `STANDARD_PALETTE`, `STANDARD_COLOR_NAMES`, `EIGHT_BIT_PALETTE`
- **Statics**: Pre-computed ANSI palettes.

---

### `style` — Text styles and attributes

#### `Attributes`
- **Description**: Bit flags for text attributes.
- **Kind**: `struct` (`Debug, Clone, Copy, PartialEq, Eq, Hash`)
- **Constants**: `BOLD`, `DIM`, `ITALIC`, `UNDERLINE`, `BLINK`, `REVERSE`, `STRIKE`, `UNDERLINE2`, `FRAME`, `ENCIRCLE`, `OVERLINE`, `BLINK2`, `CONCEAL`
- **Methods**:
  - `const fn empty() -> Self`
  - `fn set(bit, value: bool)`
  - `fn get(bit) -> bool`
  - `const fn bits() -> u32`

#### `Style`
- **Description**: A terminal style — foreground/background color, text attributes, optional hyperlink. Three-state attribute system (set true, set false, or not set / inherit).
- **Kind**: `struct` (`Debug, Clone, PartialEq, Eq, Hash`)
- **Key methods**:
  - `fn null() -> Self` — empty/inherit style
  - `fn new() -> Self`
  - Builder methods: `.color(…)`, `.bgcolor(…)`, `.bold(bool)`, `.dim(bool)`, `.italic(bool)`, `.underline(bool)`, `.blink(bool)`, `.reverse(bool)`, `.strike(bool)`, `.blink2(bool)`, `.conceal(bool)`, `.underline2(bool)`, `.frame(bool)`, `.encircle(bool)`, `.overline(bool)`, `.link(url)`
  - `fn from_str(definition: &str) -> Self` — e.g. `"bold red on blue"`
  - `fn without_color() -> Self`
  - `fn background_style() -> Self`
  - `fn transparent_background() -> bool`
  - `fn is_null() -> bool`
  - `fn is_plain() -> bool`
  - `fn get_bold() -> Option<bool>` (and similar for each attribute)
  - `fn combine(&self, other: &Style) -> Style` — merge styles
  - `fn to_ansi() -> String` — render as ANSI SGR sequence
  - `fn reset_ansi() -> &'static str`

#### `StyleType`
- **Kind**: type alias `= Style`

#### `StyleStack`
- **Description**: A stack of styles for nested markup rendering.
- **Kind**: `struct` (`Debug, Clone`)
- **Methods**: `new(default)`, `current()`, `push(style)`, `pop()`, `len()`, `is_empty()`

---

### `segment` — Styled text units

#### `ControlType`
- **Description**: Non-printable control codes.
- **Kind**: `enum` (`Debug, Clone, Copy, PartialEq, Eq, Hash`)
- **Variants**: `Bell`, `CarriageReturn`, `Home`, `Clear`, `ShowCursor`, `HideCursor`, `EnableAltScreen`, `DisableAltScreen`, `CursorUp`, `CursorDown`, `CursorForward`, `CursorBackward`, `CursorMoveToColumn`, `CursorMoveTo`, `EraseInLine`, `SetWindowTitle`

#### `ControlCode`
- **Description**: A control code with optional parameters.
- **Kind**: `enum` (`Debug, Clone, PartialEq, Eq, Hash`)
- **Variants**: `Simple(ControlType)`, `WithInt(ControlType, i32)`, `WithTwoInts(ControlType, i32, i32)`, `WithString(ControlType, String)`

#### `Segment`
- **Description**: The smallest unit of output — styled text or a control code.
- **Kind**: `struct` (`Debug, Clone, PartialEq`)
- **Fields**: `text: String`, `style: Option<Style>`, `control: Option<ControlCode>`
- **Key methods**:
  - `fn new(text) -> Self`
  - `fn styled(text, style) -> Self`
  - `fn control(code) -> Self`
  - `fn line() -> Self`
  - `fn cell_length() -> usize`
  - `fn is_empty() -> bool`
  - `fn split(offset) -> (Segment, Option<Segment>)`
  - `fn to_ansi() -> String`

#### `Segments`
- **Description**: A collection of `Segment`s.
- **Kind**: `struct` (`Debug, Clone, Default`)
- **Fields**: `segments: Vec<Segment>`
- **Methods**: `new()`, `push(seg)`, `extend(…)`, `to_ansi()`, `cell_len()`

#### `line() -> Segment`
- **Free fn**: Create a newline segment.

#### `space(count) -> Segment`
- **Free fn**: Create a space segment.

---

### `text` — Styled text with spans

#### `Span`
- **Description**: A marked-up region in some text.
- **Kind**: `struct` (`Debug, Clone, PartialEq`)
- **Fields**: `start: usize`, `end: usize`, `style: Style`
- **Methods**: `new(start, end, style)`, `is_empty()`, `split(offset)`, `move_by(offset)`, `right_crop(offset)`

#### `Text`
- **Description**: A renderable piece of text with optional style spans.
- **Kind**: `struct` (`Debug, Clone`)
- **Fields**: `plain: String`, `spans: Vec<Span>`, `style: Style`, `justify: JustifyMethod`, `end: String`, `overflow: OverflowMethod`, `no_wrap: bool`
- **Key methods**:
  - `fn new(plain) -> Self`
  - `fn styled(plain, style) -> Self`
  - Builder: `.style(…)`, `.justify(…)`, `.end(…)`, `.overflow(…)`
  - `fn append(&mut self, text, style)`
  - `fn append_styled(&mut self, text, style)`
  - `fn cell_len() -> usize`
  - `fn style_at(position) -> Style`
  - `fn truncate(max_width, overflow)`
  - `fn expand_tabs()`
  - `fn split_lines() -> Vec<Text>`
  - `fn render() -> String`
  - `fn pad(count, char)`
  - `fn pad_left(count, char)`
  - `fn pad_right(count, char)`
  - `fn align(method, width)`
  - `fn stylize(style, start, end)`
  - `fn highlight_regex(pattern, style) -> usize`
  - `fn wrap(width) -> Vec<Text>`

#### `TextType`
- **Description**: Either a plain `&str` or a `Text`.
- **Kind**: `enum` (`Debug, Clone`)
- **Variants**: `Plain(String)`, `Rich(Text)`
- **Methods**: `render() -> String`

#### Type aliases: `JustifyMethod = AlignMethod`, `OverflowMethod = crate::console::OverflowMethod`

---

### `theme` — Named style themes

#### `Theme`
- **Description**: A set of named `Style` values with optional inheritance.
- **Kind**: `struct` (`Debug, Clone`)
- **Fields**: `styles: HashMap<String, Style>`, `inherit: Option<Box<Theme>>`
- **Methods**: `new()`, `with_inherit(theme)`, `get(name)`, `set(name, style)`, `merge(other)`

#### `ThemeStack`
- **Description**: A stack of themes, checked top-down.
- **Kind**: `struct` (`Debug, Clone`)
- **Methods**: `new()`, `push(theme)`, `pop()`, `get(name)`

#### `default_theme() -> Theme`
- **Free fn**: Returns the default Rich-like theme with styles for repr, table, tree, progress, markdown, JSON, and traceback.

#### `pub mod names`
- **Module**: Constants for theme style keys (e.g. `REPR_NUMBER`, `TABLE_HEADER`, `TRACEBACK_BORDER`, `JSON_KEY`, `MARKDOWN_H1`, etc.).

---

### `measure` — Width measurement

#### `Measurement`
- **Description**: Min/max width range for a renderable.
- **Kind**: `struct` (`Debug, Clone, Copy, PartialEq, Eq`)
- **Fields**: `minimum: usize`, `maximum: usize`
- **Methods**: `new(min, max)`, `with_maximum(max)`, `with_minimum(min)`, `shrink(amount)`, `grow(amount)`, `fixed(width)`

#### `Measurable`
- **Kind**: `trait`
- **Method**: `fn measure(&self, options: &ConsoleOptions) -> Measurement`

#### `measure_renderables(items, options) -> Measurement`
- **Free fn**: Aggregate measurements from a collection.

---

### `align` — Alignment

#### `AlignMethod`
- **Description**: Horizontal alignment method.
- **Kind**: `enum` (`Debug, Clone, Copy, PartialEq, Eq, Hash`)
- **Variants**: `Left`, `Center`, `Right`, `Full`
- **Methods**: `fn align_text(&self, text, width) -> String`, `fn from_str(s) -> Self`

#### `VerticalAlignMethod`
- **Description**: Vertical alignment method.
- **Kind**: `enum` (`Debug, Clone, Copy, PartialEq, Eq, Hash`)
- **Variants**: `Top`, `Middle`, `Bottom`

#### `Align<T: Renderable>`
- **Description**: Wraps a renderable with horizontal and/or vertical alignment.
- **Kind**: `struct` (`Debug, Clone`)
- **Fields**: `renderable: T`, `align: AlignMethod`, `vertical: VerticalAlignMethod`, `width: Option<usize>`, `height: Option<usize>`
- **Methods**: `new(r)`, `.align(m)`, `.vertical(m)`, `center(r)`, `middle(r)`
- Implements `Renderable`.

---

### `markup` — Console markup parser

#### `Tag`
- **Description**: A parsed markup tag (`[bold]`, `[red]`, `[/]`, etc.).
- **Kind**: `struct` (`Debug, Clone, PartialEq`)
- **Fields**: `name: String`, `parameters: Option<String>`
- **Methods**: `new(name)`, `with_params(name, params)`, `is_closing()`, `closing_name()`, `markup()`

#### `render(markup: &str) -> Text`
- **Free fn**: Parse console markup and return styled `Text`.

#### `escape(markup: &str) -> String`
- **Free fn**: Escape text so it won't be interpreted as markup.

---

### `ratio` — Space distribution

| Function | Description |
|----------|-------------|
| `ratio_distribute(total, ratios, minimums) -> Vec<usize>` | Distribute space proportionally. |
| `ratio_resolve(total, fixed_sizes, ratios, minimums) -> Vec<usize>` | Resolve flexible layout sizes with fixed+ratio items. |
| `ratio_reduce(total, ratios, maximums, values) -> Vec<usize>` | Reduce values proportionally to fit `total`. |

---

## Console

### `console` — Central rendering engine

#### `ConsoleDimensions`
- **Description**: Terminal cell size.
- **Kind**: `struct` (`Debug, Clone, Copy, PartialEq, Eq`)
- **Fields**: `width: usize`, `height: usize`
- **Methods**: `detect() -> Self`

#### `OverflowMethod`
- **Description**: How to handle text overflow.
- **Kind**: `enum` (`Debug, Clone, Copy, PartialEq, Eq, Hash`)
- **Variants**: `Fold` (wrap), `Crop` (clip), `Ellipsis` (clip with "..."), `Ignore` (no clip)

#### `ConsoleOptions`
- **Description**: Options passed to renderables during rendering.
- **Kind**: `struct` (`Debug, Clone`)
- **Fields**: `size`, `is_terminal`, `encoding`, `min_width`, `max_width`, `max_height`, `justify`, `overflow`, `no_wrap`, `ascii_only`, `markup`, `highlight`, `height`, `legacy_windows`
- **Methods**: `update_width(max_width)`, `update_height(height)`, `shrink_width(amount)`

#### `RenderItem`
- **Description**: A single item in a render result — either a `Segment` or a nested `DynRenderable`.
- **Kind**: `enum` (`Debug, Clone`)
- **Variants**: `Segment(Segment)`, `Nested(DynRenderable)`

#### `RenderResult`
- **Description**: Result of rendering a renderable — flat line-oriented segments plus optional nested items for recursive flattening.
- **Kind**: `struct` (`Debug, Clone`)
- **Fields**: `lines: Vec<Vec<Segment>>`, `items: Vec<RenderItem>`
- **Methods**:
  - `new()`, `from_text(str)`, `from_segments(…)`, `from_lines(…)`, `from_items(…)`
  - `push_item(…)`, `push_renderable(…)`
  - `flatten(options) -> Vec<Segment>`
  - `to_ansi() -> String`

#### `Renderable`
- **Description**: Trait for anything that can be rendered to the console.
- **Kind**: `trait`
- **Required method**: `fn render(&self, options: &ConsoleOptions) -> RenderResult`
- **Optional method**: `fn measure(&self, options) -> Option<Measurement>`

Blanket impls for `String`, `&str`, and `Text`.

#### `DynRenderable`
- **Description**: A `Clone`+`Debug` wrapper for trait-object renderables.
- **Kind**: `struct` (`Debug, Clone`)
- **Method**: `fn new(r: impl Renderable + Send + Sync + 'static) -> Self`
- Implements `Renderable`.

#### `Group`
- **Description**: A group of renderables rendered one after another.
- **Kind**: `struct` (`Debug, Clone`)
- **Fields**: `children: Vec<DynRenderable>`
- **Methods**: `new()`, `add(r)`
- Implements `Renderable`.

#### `Console`
- **Description**: The main console for rendering rich output.
- **Kind**: `struct` (no public fields exposed except `file`, `color_system`, `theme`, `options`, `quiet`, `soft_wrap`)
- **Key methods**:
  - `new()`, `with_file(file)`
  - `set_width(w)`, `set_height(h)`, `width()`, `height()`
  - `render_lines(renderable, options, style, pad)`
  - `get_style(name, default)`
  - `render_str(text, style)`
  - `print(objects, sep, end)`, `println(r)`, `print_str(text)`, `print_json(data)`
  - `clear()`, `show_cursor()`, `hide_cursor()`, `set_window_title(title)`
  - `color_ansi(color) -> String`
  - `render(renderable, options) -> Vec<Segment>`
  - `measure(renderable, options) -> Measurement`
  - `rule(title, characters, style, align)`
  - `bell()`, `line(count)`, `log(objects)`
  - `push_theme(theme)`, `pop_theme()`
  - `set_quiet(bool)`, `quiet(bool) -> Self`, `set_soft_wrap(bool)`, `soft_wrap(bool) -> Self`
  - `input(prompt, password) -> String`
  - `screen() -> ScreenContext`, `set_alt_screen(bool)`
  - `is_terminal()`, `set_size(w, h)`
  - `on_broken_pipe()`

#### `get_console() -> MutexGuard<Console>`
- **Free fn**: Get the global Console instance.

#### `print_objects(objects)`, `print_str(text)`, `print_json_val(data)`
- **Free fns**: Convenience wrappers using the global Console.

---

### `box_drawing` — Box style definitions

#### `BoxStyle`
- **Description**: Characters for drawing a bordered box (28 characters: 8 rows x 4 columns).
- **Kind**: `struct` (`Debug, Clone, PartialEq, Eq`)
- **Fields**: `top_left`, `top`, `top_divider`, `top_right`, `head_left`, `head_horizontal`, `head_vertical`, `head_right`, `head_row_*` (4 fields), `mid_*` (4 fields), `row_*` (4 fields), `foot_row_*` (4 fields), `foot_*` (4 fields), `bottom_left`, `bottom`, `bottom_divider`, `bottom_right`, `ascii: bool`
- **Methods**: `from_str(box_str, ascii)`, `to_string()`

#### Static box constants: `BOX_ROUNDED`, `BOX_SQUARE`, `BOX_HEAVY`, `BOX_HEAVY_EDGE`, `BOX_HEAVY_HEAD`, `BOX_DOUBLE`, `BOX_DOUBLE_EDGE`, `BOX_SIMPLE`, `BOX_SIMPLE_HEAVY`, `BOX_MINIMAL`, `BOX_MINIMAL_HEAVY`, `BOX_ASCII`, `BOX_ASCII2`, `BOX_SQUARE_DOUBLE_HEAD`, `BOX_MINIMAL_DOUBLE_HEAD`, `BOX_SIMPLE_HEAD`, `BOX_ASCII_DOUBLE_HEAD`, `BOX_MARKDOWN`

#### `get_safe_box(box_style, ascii_only) -> BoxStyle`
- **Free fn**: Return ASCII-safe version if needed.

---

## Renderable Components

### `panel` — Bordered container

#### `Panel`
- **Description**: Draws a border around its contents.
- **Kind**: `struct` (`Clone`, `Debug`)
- **Fields**: `renderable`, `box_style`, `title`, `title_align: AlignMethod`, `subtitle`, `subtitle_align`, `expand`, `style`, `border_style`, `width`, `height`, `padding: (top, right, bottom, left)`, `highlight`
- **Builder methods**:
  - `new(r)`, `.box_style(bs)`, `.title(t)`, `.subtitle(t)`, `.border_style(s)`, `.style(s)`
  - `.width(w)`, `.height(h)`, `.padding(t, r, b, l)`, `.fit()`, `.title_align(align)`
- Implements `Renderable`.

---

### `table` — Tabular data

#### `Cell`
- **Description**: A single cell in a table row.
- **Kind**: `struct` (`Debug, Clone`)
- **Fields**: `content: String`, `style: Option<Style>`, `colspan: usize`, `rowspan: usize`
- **Methods**: `new(content)`, `.style(s)`, `.colspan(n)`, `.rowspan(n)`
- `From<String>` and `From<&str>`.

#### `Column`
- **Description**: Defines a column within a Table.
- **Kind**: `struct` (`Debug, Clone`)
- **Fields**: `header`, `footer`, `header_style`, `footer_style`, `style`, `justify: AlignMethod`, `vertical: VerticalAlignMethod`, `overflow: OverflowMethod`, `width`, `min_width`, `max_width`, `ratio`, `colspan`
- **Builder methods**: `new(header)`, `.justify(j)`, `.width(w)`, `.min_width(w)`, `.max_width(w)`, `.style(s)`, `.header_style(s)`, `.ratio(r)`, `.overflow(o)`

#### `Table`
- **Description**: A renderable for tabular data.
- **Kind**: `struct` (`Debug, Clone`)
- **Key fields**: `title`, `caption`, `box_style`, `show_header`, `show_footer`, `show_edge`, `show_lines`, `padding`, `collapse_padding`, `style`, `border_style`, `title_style`, `caption_style`, `title_justify`, `caption_justify`, `highlight`, `width`, `row_styles`, `leading`, `section_rows`
- **Methods**:
  - `new()`, `add_column(col)`, `add_row(cells)`, `add_row_str(strings)`
  - Builder: `.column(c)`, `.row(cells)`, `.row_str(strings)`, `.title(t)`, `.caption(t)`, `.box_style(bs)`, `.border_style(s)`, `.hide_header()`, `.show_lines()`, `.leading(n)`
  - `fn grid() -> Self` — grid table without outer border/header/footer
  - `fn add_section()`, `fn row_count()`
- Implements `Renderable`.

---

### `tree` — Hierarchical tree

#### `TreeGuides`
- **Description**: Characters for tree guide lines.
- **Kind**: `struct` (`Debug, Clone`)
- **Fields**: `space`, `continue_line`, `fork`, `end` (all `&'static str`)

**Constants**: `TREE_GUIDES` (Unicode), `ASCII_GUIDES` (ASCII).

#### `Tree`
- **Description**: A renderable tree node with children.
- **Kind**: `struct` (`Debug, Clone`)
- **Fields**: `label: String`, `style`, `guide_style`, `expanded`, `highlight`, `hide_root`, `children: Vec<Tree>`
- **Methods**:
  - `new(label)`
  - `fn add(&mut self, label) -> &mut Tree` — add child, returns mutable reference
  - `.style(s)`, `.guide_style(s)`, `.hide_root()`
- Implements `Renderable`.

---

### `rule` — Horizontal divider

#### `Rule`
- **Description**: A horizontal rule (divider) with optional centered title.
- **Kind**: `struct` (`Debug, Clone`)
- **Fields**: `title: String`, `characters: String`, `style`, `end: String`, `align: AlignMethod`
- **Builder methods**: `new()`, `.title(t)`, `.characters(chars)`, `.style(s)`, `.align(a)`
- Implements `Renderable`.

---

### `padding` — Space around content

#### `PaddingDimensions`
- **Description**: CSS-style padding (1, 2, or 4 values).
- **Kind**: `struct` (`Debug, Clone, Copy`)
- **Fields**: `top`, `right`, `bottom`, `left`
- **Methods**: `all(pad)`, `symmetric(vert, horiz)`, `new(top, right, bottom, left)`

#### `Padding`
- **Description**: Wraps a renderable with padding.
- **Kind**: `struct` (`Clone`, `Debug`)
- **Fields**: `renderable`, `pad: PaddingDimensions`, `style`, `expand`
- **Builder methods**:
  - `new(r)`, `.pad(t, r, b, l)`, `.pad_all(pad)`, `.indent(level)`, `.style(s)`
- Implements `Renderable`.

---

### `columns` — Side-by-side layout

#### `Columns`
- **Description**: Renders a set of renderables in side-by-side columns.
- **Kind**: `struct` (`Clone`, `Debug`)
- **Fields**: `renderables: Vec<DynRenderable>`, `equal: bool`, `expand: bool`, `padding: usize`, `width: Option<usize>`
- **Methods**: `new()`, `add(r)`, `.padding(p)`, `.equal()`, `.expand()`
- Implements `Renderable`.

---

### `layout` — Split-pane layout

#### `Region`
- **Description**: A region on screen.
- **Kind**: `struct` (`Debug, Clone, Copy, PartialEq, Eq`)
- **Fields**: `x: usize`, `y: usize`, `width: usize`, `height: usize`

#### `Direction`
- **Kind**: `enum` (`Debug, Clone, Copy, PartialEq, Eq`)
- **Variants**: `Horizontal`, `Vertical`

#### `LayoutNode`
- **Description**: A layout node — split container or leaf.
- **Kind**: `enum` (`Debug, Clone`)
- **Variants**:
  - `Split { direction, sizes, children }`
  - `Leaf { name, renderable, size }`
- **Methods**: `split(direction, children)`, `.sizes(sizes)`

#### `Layout`
- **Description**: The layout compute engine.
- **Kind**: `struct` (`Debug, Clone`)
- **Fields**: `root: LayoutNode`, `visible: bool`, `minimum_size: usize`
- **Methods**: `new(root)`, `compute(total_width, total_height) -> Vec<(String, Region)>`

---

## Dynamic / Animated Components

### `progress` — Multi-task progress bars

#### `ProgressBar`
- **Description**: A single progress bar.
- **Kind**: `struct` (`Debug, Clone`)
- **Fields**: `total: Option<f64>`, `completed: f64`, `width`, `complete_char`, `remaining_char`, `pulse`, `complete_style`, `remaining_style`, `pulse_style`
- **Methods**: `new()`, `.total(f)`, `.completed(f)`, `.width(w)`, `.complete_style(s)`, `.remaining_style(s)`, `percentage() -> f64`, `render(width) -> String`

#### `Task`
- **Description**: A tracked task within a Progress display.
- **Kind**: `struct` (`Debug, Clone`)
- **Fields**: `id: usize`, `description: String`, `total: Option<f64>`, `completed: f64`, `visible: bool`, `start_time: Instant`, `fields: HashMap<String, String>`
- **Methods**: `new(id, desc, total)`, `progress() -> f64`, `elapsed() -> Duration`, `time_remaining() -> Option<Duration>`, `is_finished() -> bool`

#### `Progress`
- **Description**: A multi-task progress display.
- **Kind**: `struct` (`Debug`)
- **Fields**: `tasks: Vec<Task>`, `auto_refresh`, `refresh_per_second`, `transient`, `columns`
- **Methods**:
  - `new()`, `.with_columns(columns)`
  - `add_task(description, total) -> usize`
  - `advance(task_id, delta)`, `update(task_id, completed)`, `remove_task(task_id)`
  - `advance_bytes(task_id, bytes)`
  - `render(width) -> String`
  - `track(iter, description, total) -> TrackIterator`
  - `open(path, description) -> io::Result<ProgressFile>`
  - `wrap_file(file, total, description) -> ProgressFile`

#### `TrackIterator<I: Iterator>`
- **Description**: An iterator wrapper that tracks progress.
- **Kind**: `struct`
- **Fields**: `progress_id: usize`
- **Methods**: `count() -> usize`, `total() -> f64`
- Implements `Iterator`.

#### `ProgressFile`
- **Description**: A file wrapper that tracks read progress.
- **Kind**: `struct` (`Debug`)
- **Methods**: `new(file, task_id, total)`, `bytes_read()`, `total()`, `task_id()`, `sync(progress)`, `inner()`, `inner_mut()`, `into_inner()`
- Implements `Read`.

---

### `progress_columns` — Progress column types

#### `ProgressColumn` (trait)
- **Method**: `fn render(&self, task, width, elapsed) -> String`

#### Concrete columns:

| Type | Description |
|------|-------------|
| `TextColumn` | Displays a formatted text field from `task.fields`. Methods: `new(key)`, `.format(fmt)`, `.style(s)` |
| `BarColumn` | Renders the progress bar itself. Methods: `new()`, `.complete_style(s)`, `.finished_style(s)`, `.width(w)` |
| `SpinnerColumn` | Shows spinner while running, checkmark when finished. Methods: `new()`, `.style(s)`, `.finished_style(s)` |
| `TimeElapsedColumn` | Shows elapsed time. Methods: `new()` |
| `TimeRemainingColumn` | Shows estimated time remaining. Methods: `new()`, `.elapsed_when_finished(bool)` |
| `TaskProgressColumn` | Shows percentage as text. Methods: `new()`, `.style(s)` |
| `MofNCompleteColumn` | Shows "completed/total". Methods: `new()`, `.style(s)`, `.separator(s)` |
| `FileSizeColumn` | Shows completed file size formatted. Methods: `new()`, `.style(s)` |
| `TotalFileSizeColumn` | Shows total file size formatted. Methods: `new()`, `.style(s)` |
| `DownloadColumn` | Shows "completed/total" with file size formatting. Methods: `new()`, `.style(s)`, `.separator(s)` |
| `TransferSpeedColumn` | Shows transfer speed. Methods: `new()`, `.style(s)` |

#### Free functions: `format_size(bytes) -> String`, `format_speed(bytes_per_sec) -> String`

---

### `spinner` — Animated spinners

#### `SpinnerFrames`
- **Description**: Predefined spinner animation frames.
- **Kind**: `struct` (`Debug, Clone`)
- **Fields**: `frames: &'static [&'static str]`, `interval: f64`

#### `Spinner`
- **Description**: An animated spinner renderable.
- **Kind**: `struct` (`Debug, Clone`)
- **Fields**: `frames`, `interval`, `text: String`, `style`
- **Methods**: `new(spinner)`, `.text(t)`, `.style(s)`, `frame_at(elapsed) -> &str`, `render(elapsed) -> String`

#### `get_spinner(name: &str) -> Option<&'static SpinnerFrames>`
- **Free fn**: Look up a spinner by name (case-insensitive).

#### `DEFAULT_SPINNER` and spinner constants:
`SPINNER_DOTS`, `SPINNER_LINE`, `SPINNER_ARC`, `SPINNER_ARROW`, `SPINNER_ARROW2`, `SPINNER_ARROW3`, `SPINNER_BOUNCING_BAR`, `SPINNER_BOUNCING_BALL`, `SPINNER_CHRISTMAS`, `SPINNER_CIRCLE`, `SPINNER_CLOCK`, `SPINNER_DOTS2`–`SPINNER_DOTS11`, `SPINNER_EARTH`, `SPINNER_GRENADE`, `SPINNER_GROW_HORIZONTAL`, `SPINNER_GROW_VERTICAL`, `SPINNER_HAMBURGER`, `SPINNER_HEARTS`, `SPINNER_MONKEY`, `SPINNER_MOON`, `SPINNER_NOISE`, `SPINNER_PONG`, `SPINNER_RUNNER`, `SPINNER_SHARK`, `SPINNER_SIMPLE_DOTS`, `SPINNER_SMILEY`, `SPINNER_TOGGLE`, `SPINNER_TRIANGLE`, `SPINNER_VERTICAL_BARS`

`SPINNERS` — all spinners as a `&[(&str, &SpinnerFrames)]` for runtime lookup.

---

### `status` — Status message with spinner

#### `Status`
- **Description**: A status message rendered with an animated spinner.
- **Kind**: `struct` (`Debug`)
- **Fields**: `spinner`, `status: String`, `started: Option<Instant>`
- **Methods**: `new(status)`, `.spinner(s)`, `start()`, `update(status)`, `stop()`, `refresh()`

---

### `live` — Auto-updating display

#### `Live`
- **Description**: Manages a live-updating region of the terminal.
- **Kind**: `struct` (`Debug`)
- **Methods**:
  - `new(renderable)`, `.screen()`, `.no_auto_refresh()`, `.refresh_per_second(rate)`, `.transient()`
  - `start()`, `stop()`, `update(renderable)`, `refresh()`
- `Drop` implementation calls `stop()`.

---

### `screen` — Full-screen rendering

#### `Screen`
- **Description**: Fills the terminal screen, cropping/padding content to fit.
- **Kind**: `struct` (`Debug`)
- **Fields**: `renderable: DynRenderable`, `style`, `application_mode`
- **Methods**: `new(r)`, `.style(s)`, `.application_mode(bool)`, `update(update)`
- Implements `Renderable`.

#### `ScreenUpdate`
- **Description**: An update to a screen display.
- **Kind**: `struct` (`Debug`)
- **Methods**: `new(r)`
- `From<R: Renderable>` impl.

#### `ScreenContext`
- **Description**: Enters alternate screen buffer, auto-exits on drop.
- **Kind**: `struct` (`Debug`)
- **Methods**: `new()`, `.style(s)`, `enter()`, `exit()`, `update(update)`, `is_active()`
- `Drop` calls `exit()`.

---

### `prompt` — Interactive prompts

#### `PromptError`
- **Description**: Errors during prompting.
- **Kind**: `enum` (`Debug`)
- **Variants**: `InvalidResponse(String)`, `IOError(io::Error)`, `Cancelled`

#### `PromptBase`
- **Description**: Base configuration for all prompt types.
- **Kind**: `struct` (no public fields directly exposed in source but pub accessible)
- **Fields**: `prompt: String`, `console: Option<Console>`, `password: bool`, `choices: Option<Vec<String>>`, `case_sensitive: bool`, `show_default: bool`, `show_choices: bool`
- **Methods**: `new(prompt)`, `.console(c)`, `.password(yes)`, `.choices(c)`, `.case_sensitive(yes)`, `.show_default(yes)`, `.show_choices(yes)`, `render_default(default)`, `make_prompt()`, `check_choice(value)`

#### `Prompt`
- **Description**: Prompt the user for a string.
- **Kind**: `struct`
- **Methods**: `new(prompt)`, `.console(c)`, `.password(yes)`, `.choices(c)`, `.case_sensitive(yes)`, `.show_choices(yes)`, `.show_default(yes)`, `render() -> String`, `ask() -> Result<String, PromptError>`, `ask_with(prompt) -> Result<String, PromptError>` (static)

#### `IntPrompt`
- **Description**: Prompt for an integer.
- **Kind**: `struct`
- **Methods**: `new(prompt)`, `.console(c)`, `.password(yes)`, `.choices(c)`, `.case_sensitive(yes)`, `ask() -> Result<i64, PromptError>`, `ask_with(prompt) -> Result<i64, PromptError>` (static)

#### `FloatPrompt`
- **Description**: Prompt for a float.
- **Kind**: `struct`
- **Methods**: `new(prompt)`, `.console(c)`, `.password(yes)`, `.choices(c)`, `.case_sensitive(yes)`, `ask() -> Result<f64, PromptError>`, `ask_with(prompt) -> Result<f64, PromptError>` (static)

#### `Confirm`
- **Description**: Yes/no prompt.
- **Kind**: `struct`
- **Fields**: `default: bool`
- **Methods**: `new(prompt, default)`, `.console(c)`, `ask() -> Result<bool, PromptError>`, `ask_with(prompt, default) -> Result<bool, PromptError>` (static)

#### `Select<T>`
- **Description**: Select from a numbered list.
- **Kind**: `struct`
- **Methods**: `new(prompt)`, `.console(c)`, `.choice(label, value)`, `render() -> String`, `ask() -> Result<T, PromptError>`
- `T: Display` required for `render()`, `T: Display + Clone` required for `ask()`.

---

## Content Rendering

### `syntax` — Syntax highlighting (powered by syntect)

#### `Syntax`
- **Description**: A syntax-highlighted source code renderable.
- **Kind**: `struct` (`Debug, Clone`)
- **Fields**: `code: String`, `language: String`, `theme: String`, `start_line: usize`, `line_numbers: bool`, `highlight: bool`, `background_color`, `tab_size: usize`
- **Builder methods**: `new(code, lang)`, `.theme(t)`, `.line_numbers()`, `.start_line(n)`, `.background(color)`
- Implements `Renderable`.

---

### `markdown` — Markdown rendering (powered by pulldown-cmark)

#### `MarkdownRender`
- **Description**: Renders markdown with Rich formatting (headings with rules, code blocks, lists, blockquotes, links).
- **Kind**: `struct` (`Debug, Clone`)
- **Methods**: `.width(w)`
- Implements `Renderable`.

#### `render_markdown(md: &str) -> MarkdownRender`
- **Free fn**: Create a `MarkdownRender`.

---

### `json` — JSON pretty printing

#### `JsonRender`
- **Description**: Renders JSON with syntax highlighting.
- **Kind**: `struct` (`Debug, Clone`)
- **Methods**: `.indent(n)`, `.theme(t)`
- Implements `Renderable`.

#### `render_json(value: &Value) -> JsonRender`
- **Free fn**: Create a `JsonRender` from a `serde_json::Value`.

---

### `logging` — Logging integration

#### `RichHandler`
- **Description**: Renders `log` crate records with Rich formatting.
- **Kind**: `struct`
- **Fields**: `console: Console`, `show_time: bool`, `show_level: bool`, `show_path: bool`, `enable_link_path: bool`, `markup: bool`, `highlighter: ReprHighlighter`
- **Methods**: `new()`, `render(level, message, module_path, file, line) -> String`, `emit(record)`

#### `install() -> Result<(), SetLoggerError>`
- **Free fn**: Install a Rich logger (placeholder).

#### `style_level(level: log::Level) -> Style`
- **Free fn**: Get the style for a log level.

---

### `traceback` — Exception traceback rendering

#### `Frame`
- **Description**: A single frame in a traceback.
- **Kind**: `struct` (`Debug, Clone`)
- **Fields**: `filename: String`, `lineno: usize`, `name: String`, `line: Option<String>`, `locals: Option<HashMap<String, String>>`, `last_instruction: Option<String>`
- **Methods**: `new(filename, lineno, name)`, `.line(l)`, `.locals(map)`

#### `Stack`
- **Description**: A stack of frames (one exception level).
- **Kind**: `struct` (`Debug, Clone`)
- **Fields**: `exc_type`, `exc_value`, `syntax_error`, `is_cause`, `frames: Vec<Frame>`, `notes: Vec<String>`, `is_group`, `exceptions: Vec<Stack>`
- **Methods**: `new()`, `.exc_type(t)`, `.exc_value(v)`, `.add_frame(frame)`

#### `Trace`
- **Description**: Full trace data (one or more stacks for chained exceptions).
- **Kind**: `struct` (`Debug, Clone`)
- **Fields**: `stacks: Vec<Stack>`
- **Methods**: `new()`, `from_stack(stack)`

#### `Traceback`
- **Description**: Renders a traceback with styled box-drawn borders, source code context, and locals tables.
- **Kind**: `struct` (`Debug, Clone`)
- **Builder methods**: `new(trace)`, `from_exception(type, value, frames)`, `.width(w)`, `.code_width(w)`, `.extra_lines(n)`, `.theme(t)`, `.word_wrap(bool)`, `.show_locals(bool)`, `.indent_guides(bool)`, `.locals_max_length(n)`, `.locals_max_string(n)`, `.locals_max_depth(n)`, `.locals_hide_dunder(bool)`, `.locals_hide_sunder(bool)`, `.suppress(vec)`, `.max_frames(n)`
- Implements `Renderable`.

#### `install()`
- **Free fn**: Install a panic hook that renders Rich-formatted tracebacks to stderr.

---

### `highlighter` — Text highlighting

#### `Highlighter` (trait)
- **Method**: `fn highlight(&self, text: &Text) -> Text`

#### `NullHighlighter`
- **Description**: A highlighter that does nothing.
- Implements `Highlighter`.

#### `RegexHighlighter`
- **Description**: Applies a list of (regex, style) rules.
- **Methods**: `new()`, `add_rule(pattern, style) -> Result<(), regex::Error>`
- Implements `Highlighter`.

#### `ReprHighlighter`
- **Description**: Highlights numbers, URLs, paths, quoted strings (Python repr-like).
- **Methods**: `new()`, `highlight_str(text) -> Text`
