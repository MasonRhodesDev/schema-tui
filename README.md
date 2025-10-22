# schema-tui

A generic, schema-driven terminal user interface for configuration management.

Built by [Mason Rhodes](https://github.com/MasonRhodesDev).

## Features

- 📝 **Schema-Driven**: Define your config UI with JSON schema files
- 🎨 **Rich Widgets**: Dropdowns, multi-select, toggles, text inputs, and more
- 🔄 **Dynamic Options**: Populate dropdowns from scripts, functions, or files at runtime
- 🎯 **Type-Safe**: Strongly typed Rust API with validation
- 🎨 **Themeable**: Customizable colors and styles
- 📦 **Zero Config**: Works out of the box with sensible defaults

## Quick Start

```rust
use schema_tui::SchemaTUI;

fn main() -> anyhow::Result<()> {
    let mut tui = SchemaTUI::from_files("schema.json", "config.toml")?;
    tui.run()?;
    Ok(())
}
```

## Use Cases

- Application configuration editors
- CLI tool settings management
- System administration tools
- Developer environment configuration
- Any application that needs a config TUI

## Status

**Work in Progress** - Initial development.

## License

MIT License - Free to use for any purpose with attribution to Mason Rhodes.
