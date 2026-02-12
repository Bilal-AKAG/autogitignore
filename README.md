# autogitignore

[![crates.io](https://img.shields.io/crates/v/autogitignore?style=for-the-badge&logo=rust)](https://crates.io/crates/autogitignore)
[![downloads](https://img.shields.io/crates/d/autogitignore?style=for-the-badge&logo=rust)](https://crates.io/crates/autogitignore)
[![release workflow](https://img.shields.io/github/actions/workflow/status/Bilal-AKAG/autogitignore/release.yml?style=for-the-badge&label=release)](https://github.com/Bilal-AKAG/autogitignore/actions/workflows/release.yml)
[![license](https://img.shields.io/crates/l/autogitignore?style=for-the-badge)](https://github.com/Bilal-AKAG/autogitignore/blob/main/LICENSE)

Search, preview, and generate `.gitignore` files from gitignore.io (Toptal) with a fast, focused TUI.

![autogitignore TUI screenshot](assets/TUI.png)

## Highlights

- Fuzzy search across templates
- Highlighted or combined preview modes
- Multi-template selection
- Offline cache after first sync
- Safe write with `.gitignore.bak` backup
- Optional output directory support

## Quick Start

Run in the current folder:

```bash
autogitignore
```

Write into a different folder:

```bash
autogitignore --dir /path/to/project
```

Run from source:

```bash
cargo run -- --dir /path/to/project
```

## Install

### From crates.io

```bash
cargo install autogitignore
```

### From GitHub Releases (prebuilt binaries)

macOS/Linux (latest):

```bash
curl -fsSL https://raw.githubusercontent.com/Bilal-AKAG/autogitignore/main/scripts/install.sh | sh
```

macOS/Linux (specific version, example `v0.1.3`):

```bash
curl -fsSL https://raw.githubusercontent.com/Bilal-AKAG/autogitignore/main/scripts/install.sh | sh -s -- v0.1.3
```

Windows PowerShell (latest):

```powershell
iwr https://raw.githubusercontent.com/Bilal-AKAG/autogitignore/main/scripts/install.ps1 -UseBasicParsing | iex
```

Windows PowerShell (specific version, example `v0.1.3`):

```powershell
& ([scriptblock]::Create((iwr https://raw.githubusercontent.com/Bilal-AKAG/autogitignore/main/scripts/install.ps1 -UseBasicParsing).Content)) -Version v0.1.3
```

By default, scripts install to `$HOME/.local/bin` (`%USERPROFILE%\\.local\\bin` on Windows). Set `BINDIR` to change it.

Then:

```bash
autogitignore
```

## Configuration

CLI options:

- `-d`, `--dir <path>`: Write the `.gitignore` file into a specific directory (defaults to the current working directory).

Cache behavior:

- Templates are cached locally after the first sync.
- The cache location is determined by your OS using the `directories` crate (app cache directory).

## Controls

| Key | Action |
| --- | --- |
| `i` or `/` | Enter search mode |
| `Esc` | Exit search or close modal |
| `Space` | Toggle selection |
| `P` | Toggle preview mode (Highlighted/Combined) |
| `Alt+J` / `Alt+K` | Scroll preview |
| `Ctrl+S` | Save |
| `Enter` | Save and quit |
| `Q` | Quit |

## Project Layout

- `src/api.rs` API client and cache
- `src/app.rs` App state and business logic
- `src/ui.rs` Ratatui rendering
- `src/gitignore.rs` File writing logic
- `src/main.rs` Event loop and input handling

## Notes

- The cache is stored using `directories` in the user cache directory.
- Linux builds using `native-tls` may require OpenSSL.

## License

MIT
