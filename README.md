# autogitignore

A terminal UI (TUI) for searching, previewing, and generating `.gitignore` files using templates from gitignore.io (Toptal).

## Why This Project Exists

This repository is a **refactored version** of an earlier prototype. It was reorganized and cleaned up as a learning project to explore:

- Rust application structure
- async I/O with Tokio
- caching and HTTP APIs
- TUI development with Ratatui

The goal is to be readable, intentional, and a practical learning reference.

## Features

- Fuzzy search for templates
- Highlighted vs. combined preview modes
- Selection of multiple templates
- Offline cache after first sync
- Safe write with `.gitignore.bak` backup
- Optional output directory support

## Usage

Run in the current folder:

```bash
autogitignore
```

Write into a different folder:

```bash
autogitignore --dir /path/to/project
```

Or run from source:

```bash
cargo run -- --dir /path/to/project
```

## Controls

- `i` or `/` Enter search mode
- `Esc` Exit search / close modal
- `Space` Toggle selection
- `P` Toggle preview mode (Highlighted/Combined)
- `Alt+J / Alt+K` Scroll preview
- `Ctrl+S` Save
- `Enter` Save and quit
- `Q` Quit

## Project Layout

- `src/api.rs` API client and cache
- `src/app.rs` App state and business logic
- `src/ui.rs` Ratatui rendering
- `src/gitignore.rs` File writing logic
- `src/main.rs` Event loop and input handling

## Notes

- The cache is stored using `directories` in the user cache directory.
- Linux builds using `native-tls` may require OpenSSL.

## Learning Goals

If you are learning Rust or Ratatui, this repo can be used as a small, complete example of:

- State-driven UI rendering
- Async background tasks in a TUI
- Clean separation of UI, state, and IO

## License

MIT
