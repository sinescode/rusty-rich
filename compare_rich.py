"""
Python Rich equivalent of the rusty-rich view_all demo.
Usage: python3 compare_rich.py > /tmp/py_rich_output.txt 2>&1
"""
from rich.console import Console
from rich.table import Table, Column
from rich.panel import Panel
from rich.columns import Columns
from rich.rule import Rule
from rich.text import Text
from rich.style import Style
from rich.box import (
    ROUNDED, SQUARE, HEAVY, HEAVY_EDGE, HEAVY_HEAD,
    DOUBLE, DOUBLE_EDGE,
    SIMPLE, SIMPLE_HEAVY,
    MINIMAL, MINIMAL_HEAVY,
    ASCII, ASCII2,
    SQUARE_DOUBLE_HEAD, MINIMAL_DOUBLE_HEAD, SIMPLE_HEAD,
    ASCII_DOUBLE_HEAD, MARKDOWN,
)

console = Console(width=80, force_terminal=True, color_system="truecolor")

# =========================================================================
# ALL 18 BOX STYLES — TABLES
# =========================================================================
rule = Rule(" ALL 18 BOX STYLES — Table ", style=Style(bold=True, color="cyan"))
console.print(rule)

all_styles = [
    ("ROUNDED", ROUNDED),
    ("SQUARE", SQUARE),
    ("HEAVY", HEAVY),
    ("HEAVY_EDGE", HEAVY_EDGE),
    ("HEAVY_HEAD", HEAVY_HEAD),
    ("DOUBLE", DOUBLE),
    ("DOUBLE_EDGE", DOUBLE_EDGE),
    ("SIMPLE", SIMPLE),
    ("SIMPLE_HEAVY", SIMPLE_HEAVY),
    ("MINIMAL", MINIMAL),
    ("MINIMAL_HEAVY", MINIMAL_HEAVY),
    ("ASCII", ASCII),
    ("ASCII2", ASCII2),
    ("SQUARE_DOUBLE_HEAD", SQUARE_DOUBLE_HEAD),
    ("MINIMAL_DOUBLE_HEAD", MINIMAL_DOUBLE_HEAD),
    ("SIMPLE_HEAD", SIMPLE_HEAD),
    ("ASCII_DOUBLE_HEAD", ASCII_DOUBLE_HEAD),
    ("MARKDOWN", MARKDOWN),
]

for name, box_style in all_styles:
    table = Table(
        Column("Item", justify="left"),
        Column("Qty", justify="center"),
        title=f" {name} ",
        box=box_style,
        border_style=Style(dim=True),
    )
    table.add_row("Widget", "5")
    table.add_row("Gadget", "12")
    console.print(table)
    console.print()

# =========================================================================
# PANEL — Edged Box Styles
# =========================================================================
rule = Rule(" PANEL — Edged Box Styles ", style=Style(bold=True, color="green"))
console.print(rule)

edged = [
    ("ROUNDED", ROUNDED),
    ("SQUARE", SQUARE),
    ("HEAVY", HEAVY),
    ("HEAVY_EDGE", HEAVY_EDGE),
    ("HEAVY_HEAD", HEAVY_HEAD),
    ("DOUBLE", DOUBLE),
    ("DOUBLE_EDGE", DOUBLE_EDGE),
    ("ASCII", ASCII),
    ("ASCII2", ASCII2),
    ("SQUARE_DOUBLE_HEAD", SQUARE_DOUBLE_HEAD),
    ("ASCII_DOUBLE_HEAD", ASCII_DOUBLE_HEAD),
]

for chunk in [edged[i:i+3] for i in range(0, len(edged), 3)]:
    cols = Columns(equal=True)
    for name, box_style in chunk:
        panel = Panel(
            name,
            title=f" {name} ",
            box=box_style,
            padding=(1, 2, 1, 2),
        )
        cols.add_renderable(panel)
    console.print(cols)
    console.print()

# =========================================================================
# PANEL — Title Alignments
# =========================================================================
rule = Rule(" PANEL — Title Alignments ", style=Style(bold=True, color="green"))
console.print(rule)

for align_name, align in [("left", "left"), ("center", "center"), ("right", "right")]:
    panel = Panel(
        f"Title aligned: {align_name}",
        title=f" {align_name} Title ",
        title_align=align,
        box=SQUARE,
        border_style=Style(color="bright_blue"),
        padding=(1, 2, 1, 2),
    )
    console.print(panel)
    console.print()

# =========================================================================
# PANEL — Subtitle Alignments
# =========================================================================
rule = Rule(" PANEL — Subtitle Alignments ", style=Style(bold=True, color="green"))
console.print(rule)

for align_name, align in [("left", "left"), ("center", "center"), ("right", "right")]:
    panel = Panel(
        f"Subtitle aligned: {align_name}",
        subtitle=f" {align_name} Subtitle ",
        subtitle_align=align,
        box=HEAVY_HEAD,
        border_style=Style(color="bright_magenta"),
        padding=(1, 2, 1, 2),
    )
    console.print(panel)
    console.print()

# =========================================================================
# TABLE — Colspan
# =========================================================================
rule = Rule(" TABLE — Colspan ", style=Style(bold=True, color="yellow"))
console.print(rule)

t1 = Table(
    Column("Product"),
    Column("Details"),
    Column("Price"),
    title=" Colspan Example ",
    box=HEAVY_HEAD,
)
t1.add_row("Laptop", "16GB RAM, 512GB SSD", "$999")
# Rich: use a single-cell row that fills the full width
t1.add_row("FREE SHIPPING ON ALL ORDERS!", end_section=True)
console.print(t1)
console.print()

# =========================================================================
# TABLE — Rowspan
# =========================================================================
rule = Rule(" TABLE — Rowspan ", style=Style(bold=True, color="yellow"))
console.print(rule)

t2 = Table(
    Column("Category"),
    Column("Item"),
    Column("Price"),
    title=" Rowspan Example ",
    box=SQUARE,
)
# Python Rich doesn't have direct rowspan, so we simulate
# Actually it also doesn't have colspan built-in for add_row
# We just show the basic structure
t2.add_row("Electronics", "Laptop", "$999")
t2.add_row("", "Phone", "$699")
t2.add_row("Clothing", "Jacket", "$89")
t2.add_row("", "Shoes", "$59")
console.print(t2)
console.print()

# =========================================================================
# TABLE — Sections
# =========================================================================
rule = Rule(" TABLE — Sections ", style=Style(bold=True, color="yellow"))
console.print(rule)

t3 = Table(
    Column("Phase"),
    Column("Status"),
    title=" Project Phases ",
    box=HEAVY_EDGE,
    border_style=Style(color="bright_cyan"),
)
t3.add_row("Planning", "✓ Done")
t3.add_row("Design", "✓ Done")
t3.add_section()
t3.add_row("Development", "⏳ In Progress")
t3.add_row("Code Review", "… Pending")
t3.add_section()
t3.add_row("Testing", "… Pending")
t3.add_row("Deploy", "… Pending")
console.print(t3)
console.print()

# =========================================================================
# TABLE — Show Lines
# =========================================================================
rule = Rule(" TABLE — Show Lines ", style=Style(bold=True, color="yellow"))
console.print(rule)

t4 = Table(
    Column("Name"),
    Column("Score"),
    Column("Rank"),
    title=" Leaderboard ",
    box=SIMPLE,
    show_lines=True,
    border_style=Style(color="bright_green"),
)
t4.add_row("Alice", "95", "1st")
t4.add_row("Bob", "87", "2nd")
t4.add_row("Carol", "82", "3rd")
console.print(t4)
console.print()

# =========================================================================
# TABLE — Footer
# =========================================================================
rule = Rule(" TABLE — Footer ", style=Style(bold=True, color="yellow"))
console.print(rule)

t5 = Table(
    Column("Item", footer="TOTAL"),
    Column("Qty", justify="center", footer="7"),
    Column("Price", justify="right", footer="$249"),
    title=" Order Summary ",
    box=HEAVY_HEAD,
    show_footer=True,
)
t5.add_row("Widget", "3", "$99")
t5.add_row("Gadget", "4", "$150")
console.print(t5)
console.print()

# =========================================================================
# TABLE — Leading
# =========================================================================
rule = Rule(" TABLE — Leading = 1 ", style=Style(bold=True, color="yellow"))
console.print(rule)

t6 = Table(
    Column("Step"),
    Column("Action"),
    title=" Deployment Steps ",
    box=ROUNDED,
    leading=1,
)
t6.add_row("1", "Connect to DB")
t6.add_row("2", "Run migrations")
t6.add_row("3", "Start server")
console.print(t6)
console.print()

# =========================================================================
# TABLE — ASCII Only
# =========================================================================
rule = Rule(" TABLE — ASCII Only ", style=Style(bold=True, color="yellow"))
console.print(rule)

t7 = Table(
    Column("Name"),
    Column("Value"),
    title=" Config ",
    box=ASCII_DOUBLE_HEAD,
)
t7.add_row("Color", "Red")
t7.add_row("Size", "Large")
console.print(t7, no_wrap=True)
console.print()

print("=" * 60)
print("  Python Rich rendering complete!")
print("=" * 60)
