# Installation

## Adding rusty-rich to Your Project

Add the following to your `Cargo.toml`:

```toml
[dependencies]
rusty-rich = "0.1"
```

If you use `cargo add`:

```shell
cargo add rusty-rich
```

---

## Minimum Supported Rust Version (MSRV)

rusty-rich targets **Rust 1.56** or later, which corresponds to the
`edition = "2021"` used by the crate. No particular MSRV policy is in place
yet; the crate tracks stable Rust releases and may raise the minimum
requirement in future minor bumps. If you are pinned to an older toolchain,
pin to a specific rusty-rich patch version.

To verify your Rust version:

```shell
rustc --version
```

---

## Feature Flags

rusty-rich does **not** currently expose any Cargo feature flags. All
functionality — tables, trees, panels, syntax highlighting, markdown,
JSON pretty-printing, progress bars, spinners, prompts, and every other
component — is compiled in by default.

If you have a use case for conditional compilation (e.g., excluding
syntax highlighting to reduce build times), please open an issue on the
[repository](https://github.com/sinescode/rusty-rich).

---

## Verifying Installation

Create a file named `src/main.rs` with the following content:

```rust
use rusty_rich::Console;

fn main() {
    let mut console = Console::new();
    console.print_str("[bold green]rusty-rich[/bold green] installed and working!\n");
}
```

Then run:

```shell
cargo run
```

If everything is set up correctly, you will see:

> **rusty-rich** installed and working!

in green text.

---

## Running the Demo

The crate ships with a full-featured demo that exercises every major
component. Run it with:

```shell
cargo run --example demo
```

You should see an interactive terminal showcase with tables, trees,
panels, syntax-highlighted code, markdown rendering, progress bars,
spinners, and a color palette.

---

## Next Steps

- [Quick Start](./quick-start.md) — build your first styled output.
- [Console](../core-concepts/console.md) — learn about the central
  rendering engine.
- [Style & Color](../core-concepts/style-and-color.md) — understand
  colors and text attributes.
