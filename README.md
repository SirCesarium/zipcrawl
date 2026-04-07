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
