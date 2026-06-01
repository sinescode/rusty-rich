export const meta = {
  name: 'compare-rich-vs-rust-rich',
  description: 'Deep comparison of Python Rich vs Rust-Rich to find logic/power gaps',
  phases: [
    { title: 'Map Python Rich', detail: 'Catalog all modules, features, capabilities of Python Rich' },
    { title: 'Map Rust-Rich', detail: 'Catalog all modules, features, capabilities of Rust-Rich port' },
    { title: 'Compare', detail: 'Side-by-side gap analysis' },
    { title: 'Synthesize', detail: 'Comprehensive gap report' },
  ],
}

phase('Map Python Rich')

const pyStructure = await agent(
  "Explore the Python Rich library at /root/tuiproject/rich. Catalog ALL source modules under rich/ directory, all public classes, functions, and key capabilities. Check: Layout, Table, Columns, Panel, Padding, Text, Markup, Syntax highlighting, Markdown, Console, export, capture, recording, Progress bars, Status, Live display, Trees, Box drawing/borders, Color system, Themes, JSON rendering, Logging handler, Prompts, File watching, Tracebacks, Inspect, Emoji, Segment/Strip, Measure, Highlighter, Styling, Spinners, Rule, Align. Check what the top-level __init__.py exports. Read key source files. Be VERY thorough.",
  { phase: 'Map Python Rich', agentType: 'Explore' }
);

phase('Map Rust-Rich')

const rsStructure = await agent(
  "Explore the Rust Rich library at /root/tuiproject/rust-rich. Catalog ALL source modules under src/, all public structs, enums, traits, impls, and key capabilities. Check: Layout, Table, Columns, Panel, Padding, Text, Markup, Syntax highlighting, Markdown, Console, export, capture, recording, Progress bars, Status, Live display, Trees, Box drawing/borders, Color system, Themes, JSON rendering, Logging handler, Prompts, File watching, Tracebacks, Inspect, Emoji, Segment/Strip, Measure, Highlighter, Styling, Spinners, Rule, Align, Cells. Also check Cargo.toml. Read key source files. Be VERY thorough.",
  { phase: 'Map Rust-Rich', agentType: 'Explore' }
);

phase('Compare')

const comparison = await agent(
  "Produce a structured gap analysis between Python Rich and its Rust port (rust-rich). For EACH module/feature area, state IMPLEMENTED (fully in Rust), PARTIAL (present but reduced), or MISSING (completely absent). Be SPECIFIC. Feature areas: Console, Text/Span, Tables, Syntax Highlighting, Markdown, Layout, Panel, Columns, Padding, Progress, Live display, Tree, Box/Borders, Color, Theme, JSON rendering, Logging handler, Rule, Align, Emoji, Spinner, Segment/Strip, Measure, Highlighter, Status, Cells, Styling, Traceback, Inspect, Prompts, File watching, Protocol/ABCs, Screen, Export formats, Constrained rendering, Renderables, Group, Ratio. For each gap assess SEVERITY (LOW/MEDIUM/HIGH) and whether LOGIC gap (behavior differs) or POWER gap (feature missing).",
  { phase: 'Compare' }
);

phase('Synthesize')

const report = await agent(
  "Synthesize all findings into a comprehensive gap report. Structure: Executive Summary, High-Severity Gaps table (Feature | Python | Rust | Impact), Medium-Severity Gaps table, Low-Severity Gaps table, Logic Gaps section (behavioral differences), Feature Completeness Scorecard (percentage per module), Recommendations for reaching parity.",
  { phase: 'Synthesize' }
);

return { pyStructure, rsStructure, comparison, report };
