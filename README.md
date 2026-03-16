# 📦 ZipCrawl

**ZipCrawl** is a high-performance ZIP explorer and content inspector for the terminal, written in Rust. It allows you to visualize, search, and peek into compressed archives without the overhead of manual extraction.

## Features

* **Tree Visualization**: Get a recursive view of the archive's structure with customizable depth.
* **Instant Inspection**: Read file contents directly to `stdout`.
* **Regex Search**: Find files within archives using powerful regular expressions.
* **Integrated with Ripgrep**: Search through file contents using `ripgrep` performance directly on the compressed data.
* **Nerd Font Support**: Clean UI with folder and file icons.

## Installation

```bash
# Clone the repository
git clone https://github.com/SirCesarium/zipcrawl
cd zipcrawl

# Build and install
cargo install --path .

```

## Usage

### Visualize structure

```bash
zipcrawl my_archive.zip tree --depth 3

```

### Search for files

```bash
zipcrawl data.zip find ".*\.json$"

```

### Grep inside the ZIP

```bash
zipcrawl modpack.mrpack grep "fabric-loader"

```

### Pipe to other tools

Integrates perfectly with tools like [`type-forge`](https://github.com/SirCesarium/type-forge) for schema generation:

```bash
zipcrawl bundle.zip cat config.json | type-forge --lang rust

```

## Commands

| Command | Description | Options |
| --- | --- | --- |
| `tree` | Show archive structure | `--depth <N>` (Default: 2) |
| `cat` | Output file content to stdout | `<FILE_PATH>` |
| `list` | List files in archive | |
| `find` | Search files by name (Regex) | `<REGEX>` |
| `grep` | Search inside file contents | `<PATTERN>` |

## Requirements

* [ripgrep](https://github.com/BurntSushi/ripgrep) (required for the `grep` command).
* A [Nerd Font](https://www.nerdfonts.com/) for proper icon rendering.
