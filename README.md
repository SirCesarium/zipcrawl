# 📦 ZipCrawl

Explore and stream ZIP archives without even extracting them.

---

`zipcrawl` is a fast, developer-focused CLI for inspecting, searching and processing ZIP archives as if they were regular filesystems without ever extracting them.

## Features

- **Tree view**
  - Visualize archive structure with sizes and hierarchy
  - `zipcrawl <file(s).zip> tree --sizes`
  - Example: `zipcrawl archives/*.zip tree --sizes`

- **Flat listing**
  - List all files quickly
  - `zipcrawl <file(s).zip> list --sizes`
  - Example: `zipcrawl myArchive.zip list`

- **Search**
  - Find files by name or pattern
  - `zipcrawl <file(s).zip> find <query>`
  - Example: `zipcrawl *.jar find ".bat|.java|.toml"`

- **Content grep**
  - Search inside files without extraction
  - `zipcrawl <file(s).zip> grep <pattern>`
  - Example: `zipcrawl "logs-*.zip" grep "FATAL_ERROR"`

- **Stream file contents**
  - Pipe file contents into external tools
  - `zipcrawl <file(s).zip> cat <path> | <command>`
  - `zipcrawl <file(s).zip> x <path> <command>`
  - Example: `zipcrawl path/to/folder/*.zip x myScript.sh bash`

- **TUI mode**
  - Interactive navigation inside archives
  - Browse and preview archives like a file explorer, but inside a ZIP and without leaving the terminal.

## Why ZipCrawl?

Because extracting archives is _slow_, _messy_ and **unnecessary**

`zipcrawl` lets you:

- Inspect large archives instantly
- Search code inside `.zip`, `.jar`, `.mrpack`, etc.
- Keep your disk clean: No more folders cluttering your /tmp or Downloads
- Search for a specific file or string across dozens of ZIPs with a single command.

## Use cases

- Inspecting any ZIP archive
- Debugging build artifacts
- Reverse engineering archives
- Grepping code inside compressed files

## Security

`zipcrawl` is built with security in mind to handle untrusted archives safely:

- **Zip Bomb Protection:** Automatically detects and rejects archives with suspicious compression ratios or excessive uncompressed sizes.
- **Path Traversal Defense:** Strict validation of internal paths to prevent files from attempting to access or overwrite locations outside the extraction context (safe `../` handling).
- **Memory Efficient:** Processes data via streaming/seeking. It never loads the entire ZIP into RAM.

## Installation

### Direct Download (Recommended)

Grab the pre-built binary for your operating system from the [Latest Releases](https://github.com/SirCesarium/zipcrawl/releases/latest).

- **Windows:** Download `zipcrawl-windows-amd64.exe`.
- **macOS:** Download `zipcrawl-macos-arm64` (Apple Silicon) or `intel`.
- **Linux:** Download `zipcrawl-linux-amd64` (or `musl` for a static binary).

### From Source (Cargo)

By default, this installs the CLI, the TUI, and NerdFont support:

```bash
cargo install zipcrawl
```

If you don't use a NerdFont and want plain ASCII icons:

```bash
cargo install zipcrawl --no-default-features --features cli
```

## Command Reference

| Command | Description                  | Flags (Short/Long)                                                                                    | Options / Details                                                                                  |
| :------ | :--------------------------- | :---------------------------------------------------------------------------------------------------- | :------------------------------------------------------------------------------------------------- |
| `tree`  | Recursive tree visualization | `-d`, `--depth` <br> `-s`, `--sizes`                                                                  | Default depth: 4. Shows uncompressed sizes.                                                        |
| `list`  | Flat entry listing           | `-s`, `--sizes`                                                                                       | Optimized for quick scans of all entries.                                                          |
| `cat`   | Stream file content          | `-q`, `--quiet`                                                                                       | Supports **glob patterns**. Quiet hides headers.                                                   |
| `find`  | Search for entries           | `-g`, `--glob` <br> `-p`, `--path` <br> `-t`, `--entry-type`                                          | Switch between Regex (default) and Glob. <br> Types: `f` (file) or `d` (directory).                |
| `grep`  | Pattern match in files       | `-g`, `--glob` <br> `-p`, `--path`                                                                    | Filter by file extension or subdirectory <br> before searching content.                            |
| `diff`  | Compare archives             | `-b`, `--base` <br> `-m`, `--mode` <br> `-i`, `--include` <br> `-e`, `--exclude` <br> `-q`, `--quiet` | **Modes**: `default`, `structure`, `stats`, `full`. <br> Supports comma-separated include/exclude. |
| `x`     | Execute command              | `-q`, `--quiet`                                                                                       | Passes file content to `stdin` of the command.                                                     |
| `tui`   | Terminal UI                  | _None_                                                                                                | Interactive navigation and previews.                                                               |

## Quick Examples

| Task                     | Command                                                         |
| :----------------------- | :-------------------------------------------------------------- |
| **Audit changes**        | `zipcrawl new.zip diff --base old.zip --mode full`              |
| **Search in Rust files** | `zipcrawl app.zip grep "fn main" --glob "*.rs"`                 |
| **Find deep JSONs**      | `zipcrawl data.zip find "*.json" --glob --path "configs/"`      |
| **Check Zip Size**       | `zipcrawl archive.zip tree --sizes --depth 2`                   |
| **Format JSON via pipe** | `zipcrawl app.zip cat "manifest.json" --quiet \| jq .`          |
| **Check checksums**      | `zipcrawl bundle.zip x "*" sha256sum`                           |
| **Compare Structure**    | `zipcrawl v2.zip diff -b v1.zip -m structure -e "temp/*,*.log"` |

## 🦀 Use as a Library

`zipcrawl` isn't just a CLI, it's a modular Rust crate. You can integrate the `ZipManager` engine into your own projects to handle archives with the same safety and speed.

```toml
[dependencies]
zipcrawl = { version = "1", default-features = false }
```

```rust
use zipcrawl::ZipManager;
use std::path::Path;

let mut manager = ZipManager::new(Path::new("archive.zip"))?;
let entries = manager.entries()?;
```

## License

This project is licensed under [MIT License](./LICENSE).
